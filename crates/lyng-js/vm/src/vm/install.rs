use super::{
    bytecode_index, code_index, feedback::FeedbackVector, Agent, AllocationLifetime, AtomId,
    BytecodeFunction, BytecodeFunctionId, CodeRef, CompiledAtom, ConstantValue, InstalledCode,
    RealmRef, TieringState, Value, Vm, VmError, VmResult,
};
use lyng_js_bytecode::{decode_instruction_bytes, CallRange, Instruction, Opcode, WideAbxOperands};
use lyng_js_env::{
    EnvironmentBindingLayout, EnvironmentLayout, EnvironmentLayoutKind, EnvironmentSlotFlags,
};
use lyng_js_gc::{CodeHandleStoreTarget, RuntimeCodeRecord, ValueStoreTarget};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct InstalledFunction {
    pub(super) function: BytecodeFunction,
    pub(super) child_codes: Vec<CodeRef>,
    canonical_atoms: Arc<[Option<AtomId>]>,
    direct_eval_lexical_sites: Vec<lyng_js_bytecode::DirectEvalLexicalSite>,
    loop_iteration_sites: Vec<lyng_js_bytecode::LoopIterationEnvironmentSite>,
    feedback_sites_by_slot: Vec<Option<lyng_js_bytecode::FeedbackSiteDescriptor>>,
}

impl InstalledFunction {
    #[inline]
    pub(super) fn new(
        function: BytecodeFunction,
        child_codes: Vec<CodeRef>,
        canonical_atoms: Arc<[Option<AtomId>]>,
    ) -> Self {
        let direct_eval_lexical_sites = function
            .direct_eval_lexical_sites()
            .iter()
            .map(|site| canonical_direct_eval_site(site, &canonical_atoms))
            .collect::<Vec<_>>();
        let loop_iteration_sites = function.loop_iteration_environment_sites().to_vec();
        let mut feedback_sites_by_slot =
            vec![None; usize::try_from(function.feedback_slot_count()).unwrap_or(usize::MAX)];
        for descriptor in function.feedback_sites() {
            if let Some(slot) = feedback_sites_by_slot.get_mut(
                usize::try_from(descriptor.slot().get().saturating_sub(1))
                    .expect("feedback slot should fit usize"),
            ) {
                *slot = Some(*descriptor);
            }
        }
        Self {
            function,
            child_codes,
            canonical_atoms,
            direct_eval_lexical_sites,
            loop_iteration_sites,
            feedback_sites_by_slot,
        }
    }

    #[inline]
    pub(super) fn instruction_at(&self, instruction_offset: u32) -> Option<Instruction> {
        decode_instruction_bytes(
            self.function
                .instruction_bytes()
                .get(usize::try_from(instruction_offset).ok()?..)?,
        )
        .ok()
    }

    pub(super) fn instruction_before(&self, instruction_offset: u32) -> Option<(u32, Instruction)> {
        let mut previous = None;
        for offset in self.function.instructions().byte_offsets() {
            let offset = u32::try_from(offset).ok()?;
            if offset >= instruction_offset {
                break;
            }
            previous = self
                .instruction_at(offset)
                .map(|instruction| (offset, instruction));
        }
        previous
    }

    pub(in crate::vm) fn feedback_descriptor_for_slot(
        &self,
        slot: lyng_js_types::FeedbackSlotId,
    ) -> Option<lyng_js_bytecode::FeedbackSiteDescriptor> {
        self.feedback_sites_by_slot
            .get(usize::try_from(slot.get().saturating_sub(1)).ok()?)
            .copied()
            .flatten()
    }

    #[inline]
    pub(super) fn loop_iteration_environment_site(
        &self,
        instruction_offset: u32,
    ) -> Option<&lyng_js_bytecode::LoopIterationEnvironmentSite> {
        self.loop_iteration_sites
            .binary_search_by_key(
                &instruction_offset,
                lyng_js_bytecode::LoopIterationEnvironmentSite::instruction_offset,
            )
            .ok()
            .and_then(|index| self.loop_iteration_sites.get(index))
    }

    #[inline]
    pub(super) fn direct_eval_lexical_site(
        &self,
        instruction_offset: u32,
    ) -> Option<&lyng_js_bytecode::DirectEvalLexicalSite> {
        self.direct_eval_lexical_sites
            .binary_search_by_key(
                &instruction_offset,
                lyng_js_bytecode::DirectEvalLexicalSite::instruction_offset,
            )
            .ok()
            .and_then(|index| self.direct_eval_lexical_sites.get(index))
    }

    #[inline]
    pub(super) fn feedback_slot_descriptors(
        &self,
    ) -> &[Option<lyng_js_bytecode::FeedbackSiteDescriptor>] {
        &self.feedback_sites_by_slot
    }

    #[inline]
    pub(super) fn source_map_entry(
        &self,
        instruction_offset: u32,
    ) -> Option<lyng_js_bytecode::SourceMapEntry> {
        self.function.source_map_entry_at(instruction_offset)
    }

    #[inline]
    pub(super) fn safepoint(
        &self,
        instruction_offset: u32,
    ) -> Option<lyng_js_bytecode::SafepointDescriptor> {
        self.function.safepoint_at(instruction_offset)
    }

    #[inline]
    pub(super) fn safepoint_by_id(
        &self,
        safepoint_id: u32,
    ) -> Option<lyng_js_bytecode::SafepointDescriptor> {
        self.function.safepoint_by_id(safepoint_id)
    }

    #[inline]
    pub(super) fn deopt_snapshot(
        &self,
        safepoint_id: u32,
    ) -> Option<&lyng_js_bytecode::DeoptSnapshot> {
        self.function.deopt_snapshot_for_safepoint(safepoint_id)
    }

    #[inline]
    pub(super) fn canonical_atom(&self, atom: AtomId) -> AtomId {
        self.canonical_atoms
            .get(usize::try_from(atom.raw()).unwrap_or(usize::MAX))
            .copied()
            .flatten()
            .unwrap_or(atom)
    }

    #[cfg(test)]
    const fn cold_metadata_index_footprint(&self) -> usize {
        self.direct_eval_lexical_sites.len() + self.loop_iteration_sites.len()
    }
}

fn canonical_direct_eval_site(
    site: &lyng_js_bytecode::DirectEvalLexicalSite,
    canonical_atoms: &[Option<AtomId>],
) -> lyng_js_bytecode::DirectEvalLexicalSite {
    let canonical_scopes = site
        .scopes()
        .iter()
        .map(|scope| {
            let canonical_scope = lyng_js_bytecode::DirectEvalLexicalScope::new(
                scope.source_base(),
                scope
                    .bindings()
                    .iter()
                    .copied()
                    .map(|binding| {
                        lyng_js_bytecode::BytecodeEnvironmentBinding::new(
                            binding
                                .name()
                                .map(|atom| canonical_atom(canonical_atoms, atom)),
                            binding.flags(),
                        )
                    })
                    .collect::<Vec<_>>(),
            );
            if let Some(name) = scope.annex_b_catch_name() {
                canonical_scope.with_annex_b_catch_name(canonical_atom(canonical_atoms, name))
            } else {
                canonical_scope
            }
        })
        .collect::<Vec<_>>();
    let canonical_annex_b_catch_names = site
        .annex_b_catch_names()
        .iter()
        .copied()
        .map(|name| canonical_atom(canonical_atoms, name))
        .collect::<Vec<_>>();
    let canonical_parameter_names = site
        .parameter_names()
        .iter()
        .copied()
        .map(|name| canonical_atom(canonical_atoms, name))
        .collect::<Vec<_>>();
    lyng_js_bytecode::DirectEvalLexicalSite::new(
        site.instruction_offset(),
        canonical_scopes,
        site.flags(),
        canonical_annex_b_catch_names,
        canonical_parameter_names,
    )
}

fn canonical_atom(canonical_atoms: &[Option<AtomId>], atom: AtomId) -> AtomId {
    canonical_atoms
        .get(usize::try_from(atom.raw()).unwrap_or(usize::MAX))
        .copied()
        .flatten()
        .unwrap_or(atom)
}

fn validate_register_operands(code: CodeRef, function: &BytecodeFunction) -> VmResult<()> {
    let register_len = function
        .register_count()
        .checked_add(function.hidden_register_count())
        .ok_or(VmError::RegisterOutOfBounds {
            code,
            register: u16::MAX,
        })?;
    for instruction in function.instructions() {
        validate_instruction_registers(code, register_len, instruction)?;
    }
    Ok(())
}

#[expect(
    clippy::too_many_lines,
    reason = "the register validation table mirrors VM dispatch operand ownership"
)]
fn validate_instruction_registers(
    code: CodeRef,
    register_len: u16,
    instruction: Instruction,
) -> VmResult<()> {
    match instruction {
        Instruction::Abc { opcode, a, b, c } | Instruction::AbcSlot { opcode, a, b, c, .. } => match opcode {
            Opcode::Move
            | Opcode::Ldar
            | Opcode::Negate
            | Opcode::BitNot
            | Opcode::Increment
            | Opcode::Decrement
            | Opcode::SetFunctionName
            | Opcode::ToPropertyKey
            | Opcode::CreateForIn
            | Opcode::CreateIterator
            | Opcode::GetNamedProperty
            | Opcode::DefineNamedProperty
            | Opcode::SetNamedProperty
            | Opcode::AssignNamedProperty
            | Opcode::StrictAssignNamedProperty
            | Opcode::StoreDenseElement
            | Opcode::LoadDenseElement
            | Opcode::AddSmi
            | Opcode::SubSmi
            | Opcode::MulSmi
            | Opcode::DivSmi
            | Opcode::ModSmi
            | Opcode::BitAndSmi
            | Opcode::EqualZero
            | Opcode::TailCall
            | Opcode::Construct => validate_registers(code, register_len, [a, b]),
            Opcode::LdaUndefined
            | Opcode::LdaNull
            | Opcode::LdaTrue
            | Opcode::LdaFalse
            | Opcode::LdaZero
            | Opcode::LdaOne
            | Opcode::LdaSmi8
            | Opcode::LdaConst8 => validate_registers(code, register_len, [0]),
            Opcode::Star0
            | Opcode::Star1
            | Opcode::Star2
            | Opcode::Star3
            | Opcode::Star4
            | Opcode::Star5
            | Opcode::Star6
            | Opcode::Star7 => validate_registers(
                code,
                register_len,
                [
                    0,
                    opcode
                        .accumulator_store_index()
                        .expect("store-accumulator opcode should have an index"),
                ],
            ),
            Opcode::Add
            | Opcode::Sub
            | Opcode::Mul
            | Opcode::Div
            | Opcode::Mod
            | Opcode::Exp
            | Opcode::BitOr
            | Opcode::BitXor
            | Opcode::BitAnd
            | Opcode::ShiftLeft
            | Opcode::ShiftRight
            | Opcode::UnsignedShiftRight
            | Opcode::Equal
            | Opcode::StrictEqual
            | Opcode::LessThan
            | Opcode::LessEqual
            | Opcode::GreaterThan
            | Opcode::GreaterEqual
            | Opcode::In
            | Opcode::GetKeyedProperty
            | Opcode::SetKeyedProperty
            | Opcode::AssignKeyedProperty
            | Opcode::StrictAssignKeyedProperty
            | Opcode::DefineKeyedProperty
            | Opcode::DeleteProperty
            | Opcode::CopyDataProperties
            | Opcode::AdvanceForIn
            | Opcode::AdvanceIterator
            | Opcode::DelegateYield
            | Opcode::Call => validate_registers(code, register_len, [a, b, c]),
            Opcode::Call0 | Opcode::Call1 | Opcode::Call2 | Opcode::Call3 => {
                validate_registers(code, register_len, [a, b])?;
                validate_small_call_registers(
                    code,
                    register_len,
                    c,
                    opcode.small_call_arity().unwrap_or(0),
                )
            }
            _ => Ok(()),
        },
        Instruction::Abx { opcode, a, bx } | Instruction::AbxSlot { opcode, a, bx, .. } => {
            let operands = WideAbxOperands::new(a, bx);
            let a = operands.a();
            let bx = operands.bx();
            match opcode {
                Opcode::LoadUndefined
                | Opcode::LoadUninitializedLexical
                | Opcode::LoadNull
                | Opcode::LoadTrue
                | Opcode::LoadFalse
                | Opcode::LoadZero
                | Opcode::LoadOne
                | Opcode::LoadSmi
                | Opcode::LoadSmi8
                | Opcode::LoadConst
                | Opcode::LoadConst8
                | Opcode::LoadEnvSlot
                | Opcode::StoreEnvSlot
                | Opcode::AssignEnvSlot
                | Opcode::LoadGlobal
                | Opcode::StoreGlobal
                | Opcode::AssignGlobal
                | Opcode::DeleteGlobal
                | Opcode::LoadName
                | Opcode::ResolveName
                | Opcode::ResolveGlobal
                | Opcode::AssignName
                | Opcode::AssignVariableName
                | Opcode::DeleteName
                | Opcode::CaptureName
                | Opcode::LoadThis
                | Opcode::LoadCallee
                | Opcode::LoadNewTarget
                | Opcode::JumpIfTrue
                | Opcode::JumpIfFalse
                | Opcode::JumpIfTrue8
                | Opcode::JumpIfFalse8
                | Opcode::CreateObject
                | Opcode::CreateArray
                | Opcode::CheckObjectCoercible
                | Opcode::ThrowIfUninitialized
                | Opcode::CreateClosure
                | Opcode::CloseForIn
                | Opcode::CloseIterator => validate_registers(code, register_len, [a]),
                Opcode::LdaSmi8 | Opcode::LdaConst8 => validate_registers(code, register_len, [0]),
                Opcode::LoadLocal0
                | Opcode::LoadLocal1
                | Opcode::LoadLocal2
                | Opcode::LoadLocal3 => validate_registers(
                    code,
                    register_len,
                    [
                        a,
                        opcode
                            .local_load_index()
                            .expect("load-local opcode should have an index"),
                    ],
                ),
                Opcode::StoreLocal0
                | Opcode::StoreLocal1
                | Opcode::StoreLocal2
                | Opcode::StoreLocal3 => validate_registers(
                    code,
                    register_len,
                    [
                        opcode
                            .local_store_index()
                            .expect("store-local opcode should have an index"),
                        a,
                    ],
                ),
                Opcode::LoadCapturedName
                | Opcode::LoadCapturedNameThis
                | Opcode::AssignCapturedName => {
                    let reference = register_from_u32(code, bx)?;
                    validate_registers(code, register_len, [a, reference])
                }
                _ => Ok(()),
            }
        }
        Instruction::Ax { opcode, ax } => match opcode {
            Opcode::PushWithEnv
            | Opcode::TypeOf
            | Opcode::Return
            | Opcode::Yield
            | Opcode::Await
            | Opcode::Throw
            | Opcode::LoadException
            | Opcode::LoadResumeKind
            | Opcode::LoadResumeValue => {
                let register = register_from_i32(code, ax)?;
                validate_registers(code, register_len, [register])
            }
            _ => Ok(()),
        },
        Instruction::CallRange {
            opcode,
            a,
            b,
            c,
            range,
            ..
        } => match opcode {
            Opcode::Call => {
                validate_registers(code, register_len, [a, b, c])?;
                validate_call_range(code, register_len, range)
            }
            Opcode::TailCall | Opcode::Construct => {
                validate_registers(code, register_len, [a, b])?;
                validate_call_range(code, register_len, range)
            }
            _ => Ok(()),
        },
    }
}

fn validate_call_range(code: CodeRef, register_len: u16, range: CallRange) -> VmResult<()> {
    if range.argument_count() == 0 {
        return Ok(());
    }
    let Some(last_register) = range
        .argument_base()
        .checked_add(range.argument_count().saturating_sub(1))
    else {
        return Err(VmError::RegisterOutOfBounds {
            code,
            register: u16::MAX,
        });
    };
    validate_registers(code, register_len, [last_register])
}

fn validate_small_call_registers(
    code: CodeRef,
    register_len: u16,
    call_base: u16,
    argument_count: u8,
) -> VmResult<()> {
    let Some(last_register) = call_base.checked_add(u16::from(argument_count)) else {
        return Err(VmError::RegisterOutOfBounds {
            code,
            register: u16::MAX,
        });
    };
    validate_registers(code, register_len, [call_base, last_register])
}

fn validate_registers<const N: usize>(
    code: CodeRef,
    register_len: u16,
    registers: [u16; N],
) -> VmResult<()> {
    for register in registers {
        if register >= register_len {
            return Err(VmError::RegisterOutOfBounds { code, register });
        }
    }
    Ok(())
}

fn register_from_u32(code: CodeRef, register: u32) -> VmResult<u16> {
    u16::try_from(register).map_err(|_| VmError::RegisterOutOfBounds {
        code,
        register: u16::MAX,
    })
}

fn register_from_i32(code: CodeRef, register: i32) -> VmResult<u16> {
    u16::try_from(register).map_err(|_| VmError::RegisterOutOfBounds { code, register: 0 })
}

impl Vm {
    #[expect(
        clippy::too_many_lines,
        reason = "recursive installation keeps code allocation, canonical atoms, and child references in one pass"
    )]
    pub(super) fn install_functions(
        &mut self,
        agent: &mut Agent,
        realm: RealmRef,
        entry: BytecodeFunctionId,
        functions: &[BytecodeFunction],
        atom_texts: &[(AtomId, CompiledAtom)],
    ) -> VmResult<InstalledCode> {
        let Some(max_function_id) = functions.iter().map(|function| function.id().get()).max()
        else {
            return Err(VmError::MissingEntry(entry));
        };
        let mut codes_by_function = vec![
            None;
            usize::try_from(max_function_id)
                .expect("bytecode function id should fit into usize")
                + 1
        ];
        let mut installed_templates = vec![
            None;
            usize::try_from(max_function_id)
                .expect("bytecode function id should fit into usize")
                + 1
        ];
        let max_atom = atom_texts
            .iter()
            .map(|(atom, _)| atom.raw())
            .max()
            .unwrap_or(0);
        let mut canonical_atoms = vec![
            None;
            usize::try_from(max_atom)
                .expect("atom id should fit into usize")
                .saturating_add(1)
        ];
        let mut compiled_atoms_by_id = vec![None; canonical_atoms.len()];
        for (atom, text) in atom_texts {
            let runtime_atom = match text {
                CompiledAtom::Utf8(text) => {
                    let runtime_atom = self
                        .preferred_atoms_by_text
                        .get(text.as_ref())
                        .copied()
                        .unwrap_or_else(|| agent.atoms_mut().intern_collectible(text));
                    self.preferred_atoms_by_text
                        .entry(text.clone())
                        .or_insert(runtime_atom);
                    self.atom_texts
                        .entry(runtime_atom)
                        .or_insert_with(|| text.clone());
                    runtime_atom
                }
                CompiledAtom::Utf16(units) => agent.atoms_mut().intern_collectible_utf16(units),
            };
            let atom_index = usize::try_from(atom.raw()).expect("atom id should fit into usize");
            canonical_atoms[atom_index] = Some(runtime_atom);
            compiled_atoms_by_id[atom_index] = Some(text);
        }
        let canonical_atoms: Arc<[Option<AtomId>]> = canonical_atoms.into();

        for function in functions {
            let function_id = function.id();
            let installed_function = if function.needs_environment() {
                let bindings = if function.environment_bindings().is_empty() {
                    (0..usize::from(function.environment_slot_count()))
                        .map(|_| {
                            EnvironmentBindingLayout::new(None, EnvironmentSlotFlags::var_like())
                        })
                        .collect::<Vec<_>>()
                } else {
                    if function.environment_bindings().len()
                        != usize::from(function.environment_slot_count())
                    {
                        return Err(VmError::InvalidEnvironmentLayout(function.id()));
                    }
                    function
                        .environment_bindings()
                        .iter()
                        .copied()
                        .map(|binding| {
                            let name = binding.name().map(|atom| {
                                canonical_atoms
                                    .get(usize::try_from(atom.raw()).unwrap_or(usize::MAX))
                                    .copied()
                                    .flatten()
                                    .unwrap_or(atom)
                            });
                            EnvironmentBindingLayout::new(
                                name,
                                EnvironmentSlotFlags::new(
                                    binding.flags().is_mutable(),
                                    binding.flags().is_lexical(),
                                    binding.flags().needs_tdz(),
                                    binding.flags().is_dynamic(),
                                )
                                .with_scoped(binding.flags().is_scoped())
                                .with_sloppy_immutable_assign_silent(
                                    binding.flags().sloppy_immutable_assign_silent(),
                                ),
                            )
                        })
                        .collect::<Vec<_>>()
                };
                let layout_kind = match function.kind() {
                    lyng_js_bytecode::BytecodeFunctionKind::Module => EnvironmentLayoutKind::Module,
                    _ => EnvironmentLayoutKind::Function,
                };
                let layout = agent.alloc_environment_layout(EnvironmentLayout::new(
                    layout_kind,
                    bindings,
                    true,
                ));
                let layout = lyng_js_bytecode::EnvironmentLayoutRef::from_raw(layout.get());
                function
                    .clone()
                    .with_environment_layout(layout)
                    .with_safepoint_environment_layout(layout)
            } else {
                function.clone()
            };
            let constants = self.install_constants(
                agent,
                realm,
                installed_function.constants(),
                &compiled_atoms_by_id,
                canonical_atoms.as_ref(),
            );
            let code = agent.heap_mut().mutator().alloc_code(
                RuntimeCodeRecord::new(None, Some(realm), constants),
                AllocationLifetime::Default,
            );
            validate_register_operands(code, &installed_function)?;
            codes_by_function[bytecode_index(function_id)] = Some(code);
            installed_templates[bytecode_index(function_id)] = Some(installed_function);
        }

        for function in functions {
            let parent_code = codes_by_function[bytecode_index(function.id())]
                .expect("allocated code should exist for every installed function");
            let installed_function = installed_templates[bytecode_index(function.id())]
                .take()
                .expect("template should be installed alongside its runtime code");
            let child_codes = function
                .child_functions()
                .iter()
                .map(|child| {
                    codes_by_function
                        .get(bytecode_index(*child))
                        .and_then(|child| *child)
                        .ok_or(VmError::MissingEntry(*child))
                })
                .collect::<VmResult<Vec<_>>>()?;

            {
                let mut mutator = agent.heap_mut().mutator();
                for child_code in &child_codes {
                    assert!(mutator.mut_store_code_handle(
                        CodeHandleStoreTarget::CodeParent(*child_code),
                        Some(parent_code),
                    ));
                }
            }

            self.store_installed(
                parent_code,
                InstalledFunction::new(
                    installed_function,
                    child_codes,
                    Arc::clone(&canonical_atoms),
                ),
            );
        }

        let code = codes_by_function
            .get(bytecode_index(entry))
            .and_then(|code| *code)
            .ok_or(VmError::MissingEntry(entry))?;

        Ok(InstalledCode::new(code, entry))
    }

    pub(super) fn install_constants(
        &mut self,
        agent: &mut Agent,
        realm: RealmRef,
        constants: &[ConstantValue],
        compiled_atoms_by_id: &[Option<&CompiledAtom>],
        canonical_atoms: &[Option<AtomId>],
    ) -> Option<lyng_js_gc::CodeSlotsRef> {
        if constants.is_empty() {
            return None;
        }

        let slots = {
            let mut mutator = agent.heap_mut().mutator();
            mutator.alloc_code_slots(
                constants.len(),
                Value::empty_internal_slot(),
                AllocationLifetime::Default,
            )
        };

        for (index, constant) in constants.iter().copied().enumerate() {
            if let Some(value) = self.constant_value(
                agent,
                realm,
                constant,
                compiled_atoms_by_id,
                canonical_atoms,
            ) {
                assert!(agent.heap_mut().mutator().init_store_value(
                    ValueStoreTarget::CodeSlot(
                        slots,
                        u32::try_from(index).expect("constant slot index should fit into u32"),
                    ),
                    value,
                ));
            }
        }
        Some(slots)
    }

    pub(super) fn store_installed(&mut self, code: CodeRef, installed: InstalledFunction) {
        let index = code_index(code);
        if self.installed.len() <= index {
            self.installed.resize_with(index + 1, || None);
        }
        if self.feedback_vectors.len() <= index {
            self.feedback_vectors
                .resize_with(index + 1, FeedbackVector::default);
        }
        self.ensure_tiering_capacity(code);
        self.tiering[index] = Some(TieringState::default());
        self.installed[index] = Some(Arc::new(installed));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lyng_js_bytecode::{
        ArgumentsMode, DeoptFrameValue, DeoptSnapshot, DeoptValueSource, Instruction, Opcode,
        SafepointDescriptor, SafepointKind, SourceMapEntry,
    };
    use lyng_js_common::SourceId;

    #[test]
    fn sparse_cold_metadata_indexes_do_not_scale_with_instruction_count() {
        let instruction_count = 1_024;
        let metadata_offset = u32::try_from(instruction_count - 1).unwrap();
        let source_map = SourceMapEntry::new(SourceId::new(7), metadata_offset, 11, 13);
        let safepoint = SafepointDescriptor::new(1, metadata_offset, SafepointKind::Allocation, 3);
        let deopt = DeoptSnapshot::new(
            1,
            vec![DeoptValueSource::FrameValue(DeoptFrameValue::ThisValue)],
        );
        let function = BytecodeFunction::new(
            BytecodeFunctionId::from_raw(1).unwrap(),
            None,
            ArgumentsMode::None,
        )
        .with_instructions(vec![Instruction::ax(Opcode::Nop, 0); instruction_count])
        .with_source_map(vec![source_map])
        .with_safepoints(vec![safepoint])
        .with_deopt_snapshots(vec![deopt]);

        let installed = InstalledFunction::new(function, Vec::new(), Arc::from([]));

        assert_eq!(
            installed.source_map_entry(metadata_offset),
            Some(source_map)
        );
        assert_eq!(installed.safepoint(metadata_offset), Some(safepoint));
        assert_eq!(installed.safepoint_by_id(1), Some(safepoint));
        assert_eq!(
            installed.deopt_snapshot(1).map(DeoptSnapshot::values),
            Some([DeoptValueSource::FrameValue(DeoptFrameValue::ThisValue)].as_slice())
        );
        assert!(
            installed.cold_metadata_index_footprint() < instruction_count,
            "cold metadata indexes should be sparse, not instruction-length"
        );
    }
}
