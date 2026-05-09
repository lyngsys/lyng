use super::{
    bytecode_index, code_index, Agent, AllocationLifetime, AtomId, BytecodeFunction,
    BytecodeFunctionId, CodeRef, CompiledAtom, ConstantValue, InstalledCode, RealmRef,
    TieringState, Value, Vm, VmError, VmResult,
};
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
    pub(super) wide_payloads: Vec<Option<u32>>,
    direct_eval_lexical_sites_by_offset: Vec<Option<lyng_js_bytecode::DirectEvalLexicalSite>>,
    loop_iteration_sites_by_offset: Vec<Option<lyng_js_bytecode::LoopIterationEnvironmentSite>>,
    feedback_sites_by_offset: Vec<Option<lyng_js_bytecode::FeedbackSiteDescriptor>>,
    feedback_sites_by_slot: Vec<Option<lyng_js_bytecode::FeedbackSiteDescriptor>>,
}

impl InstalledFunction {
    #[inline]
    #[expect(
        clippy::too_many_lines,
        reason = "installed bytecode side tables are built in one pass per metadata family for locality"
    )]
    pub(super) fn new(
        function: BytecodeFunction,
        child_codes: Vec<CodeRef>,
        canonical_atoms: Arc<[Option<AtomId>]>,
    ) -> Self {
        let mut wide_payloads = vec![None; function.instructions().len()];
        for operand in function.wide_operands() {
            if let Some(slot) = wide_payloads.get_mut(
                usize::try_from(operand.instruction_offset())
                    .expect("instruction offset should fit usize"),
            ) {
                *slot = Some(operand.payload());
            }
        }
        let mut direct_eval_lexical_sites_by_offset = vec![
            None;
            offset_table_len(
                function
                    .direct_eval_lexical_sites()
                    .iter()
                    .map(lyng_js_bytecode::DirectEvalLexicalSite::instruction_offset),
            )
        ];
        for site in function.direct_eval_lexical_sites() {
            if let Some(slot) = direct_eval_lexical_sites_by_offset.get_mut(
                usize::try_from(site.instruction_offset())
                    .expect("instruction offset should fit usize"),
            ) {
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
                                        binding.name().map(|atom| {
                                            canonical_atoms
                                                .get(
                                                    usize::try_from(atom.raw())
                                                        .unwrap_or(usize::MAX),
                                                )
                                                .copied()
                                                .flatten()
                                                .unwrap_or(atom)
                                        }),
                                        binding.flags(),
                                    )
                                })
                                .collect::<Vec<_>>(),
                        );
                        if let Some(name) = scope.annex_b_catch_name() {
                            canonical_scope.with_annex_b_catch_name(
                                canonical_atoms
                                    .get(usize::try_from(name.raw()).unwrap_or(usize::MAX))
                                    .copied()
                                    .flatten()
                                    .unwrap_or(name),
                            )
                        } else {
                            canonical_scope
                        }
                    })
                    .collect::<Vec<_>>();
                let canonical_annex_b_catch_names = site
                    .annex_b_catch_names()
                    .iter()
                    .copied()
                    .map(|name| {
                        canonical_atoms
                            .get(usize::try_from(name.raw()).unwrap_or(usize::MAX))
                            .copied()
                            .flatten()
                            .unwrap_or(name)
                    })
                    .collect::<Vec<_>>();
                let canonical_parameter_names = site
                    .parameter_names()
                    .iter()
                    .copied()
                    .map(|name| {
                        canonical_atoms
                            .get(usize::try_from(name.raw()).unwrap_or(usize::MAX))
                            .copied()
                            .flatten()
                            .unwrap_or(name)
                    })
                    .collect::<Vec<_>>();
                *slot = Some(lyng_js_bytecode::DirectEvalLexicalSite::new(
                    site.instruction_offset(),
                    canonical_scopes,
                    site.flags(),
                    canonical_annex_b_catch_names,
                    canonical_parameter_names,
                ));
            }
        }
        let mut loop_iteration_sites_by_offset = vec![
            None;
            offset_table_len(
                function
                    .loop_iteration_environment_sites()
                    .iter()
                    .map(lyng_js_bytecode::LoopIterationEnvironmentSite::instruction_offset),
            )
        ];
        for site in function.loop_iteration_environment_sites() {
            if let Some(slot) = loop_iteration_sites_by_offset.get_mut(
                usize::try_from(site.instruction_offset())
                    .expect("instruction offset should fit usize"),
            ) {
                *slot = Some(site.clone());
            }
        }
        let mut feedback_sites_by_offset = vec![None; function.instructions().len()];
        let mut feedback_sites_by_slot =
            vec![None; usize::try_from(function.feedback_slot_count()).unwrap_or(usize::MAX)];
        for descriptor in function.feedback_sites() {
            if let Some(slot) = feedback_sites_by_offset.get_mut(
                usize::try_from(descriptor.instruction_offset())
                    .expect("instruction offset should fit usize"),
            ) {
                *slot = Some(*descriptor);
            }
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
            wide_payloads,
            direct_eval_lexical_sites_by_offset,
            loop_iteration_sites_by_offset,
            feedback_sites_by_offset,
            feedback_sites_by_slot,
        }
    }

    #[inline]
    pub(super) fn wide_payload(&self, instruction_offset: u32) -> Option<u32> {
        self.wide_payloads
            .get(usize::try_from(instruction_offset).ok()?)
            .copied()
            .flatten()
    }

    #[inline]
    pub(super) fn feedback_descriptor(
        &self,
        instruction_offset: u32,
    ) -> Option<lyng_js_bytecode::FeedbackSiteDescriptor> {
        self.feedback_sites_by_offset
            .get(usize::try_from(instruction_offset).ok()?)
            .copied()
            .flatten()
    }

    #[inline]
    pub(super) fn loop_iteration_environment_site(
        &self,
        instruction_offset: u32,
    ) -> Option<&lyng_js_bytecode::LoopIterationEnvironmentSite> {
        self.loop_iteration_sites_by_offset
            .get(usize::try_from(instruction_offset).ok()?)?
            .as_ref()
    }

    #[inline]
    pub(super) fn direct_eval_lexical_site(
        &self,
        instruction_offset: u32,
    ) -> Option<&lyng_js_bytecode::DirectEvalLexicalSite> {
        self.direct_eval_lexical_sites_by_offset
            .get(usize::try_from(instruction_offset).ok()?)?
            .as_ref()
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
        self.direct_eval_lexical_sites_by_offset.len() + self.loop_iteration_sites_by_offset.len()
    }
}

#[inline]
fn offset_table_len(offsets: impl Iterator<Item = u32>) -> usize {
    offsets.max().map_or(0, |offset| {
        usize::try_from(offset)
            .unwrap_or(usize::MAX)
            .saturating_add(1)
    })
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
        if self.feedback_warmup.len() <= index {
            self.feedback_warmup.resize(index + 1, 0);
        }
        if self.feedback_vectors.len() <= index {
            self.feedback_vectors.resize_with(index + 1, || None);
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
