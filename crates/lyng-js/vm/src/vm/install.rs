use super::*;
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
    source_maps_by_offset: Vec<Option<lyng_js_bytecode::SourceMapEntry>>,
    safepoints_by_offset: Vec<Option<lyng_js_bytecode::SafepointDescriptor>>,
    safepoints_by_id: Vec<Option<lyng_js_bytecode::SafepointDescriptor>>,
    deopt_by_safepoint_id: Vec<Option<lyng_js_bytecode::DeoptSnapshot>>,
}

impl InstalledFunction {
    #[inline]
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
        let mut direct_eval_lexical_sites_by_offset = vec![None; function.instructions().len()];
        for site in function.direct_eval_lexical_sites() {
            if let Some(slot) = direct_eval_lexical_sites_by_offset.get_mut(
                usize::try_from(site.instruction_offset())
                    .expect("instruction offset should fit usize"),
            ) {
                let canonical_scopes = site
                    .scopes()
                    .iter()
                    .map(|scope| {
                        lyng_js_bytecode::DirectEvalLexicalScope::new(
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
                        )
                    })
                    .collect::<Vec<_>>();
                *slot = Some(lyng_js_bytecode::DirectEvalLexicalSite::new(
                    site.instruction_offset(),
                    canonical_scopes,
                    site.flags(),
                ));
            }
        }
        let mut loop_iteration_sites_by_offset = vec![None; function.instructions().len()];
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
        let mut source_maps_by_offset = vec![None; function.instructions().len()];
        let mut safepoints_by_offset = vec![None; function.instructions().len()];
        let max_safepoint_id = function
            .safepoints()
            .iter()
            .map(|descriptor| descriptor.id())
            .max()
            .unwrap_or(0);
        let mut safepoints_by_id =
            vec![None; usize::try_from(max_safepoint_id).unwrap_or(usize::MAX)];
        let mut deopt_by_safepoint_id =
            vec![None; usize::try_from(max_safepoint_id).unwrap_or(usize::MAX)];
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
        for entry in function.source_map() {
            if let Some(slot) = source_maps_by_offset.get_mut(
                usize::try_from(entry.instruction_offset())
                    .expect("instruction offset should fit usize"),
            ) {
                *slot = Some(*entry);
            }
        }
        for descriptor in function.safepoints() {
            if let Some(slot) = safepoints_by_offset.get_mut(
                usize::try_from(descriptor.instruction_offset())
                    .expect("instruction offset should fit usize"),
            ) {
                *slot = Some(*descriptor);
            }
            if let Some(slot) = safepoints_by_id.get_mut(
                usize::try_from(descriptor.id().saturating_sub(1))
                    .expect("safepoint id should fit usize"),
            ) {
                *slot = Some(*descriptor);
            }
        }
        for snapshot in function.deopt_snapshots() {
            if let Some(slot) = deopt_by_safepoint_id.get_mut(
                usize::try_from(snapshot.safepoint_id().saturating_sub(1))
                    .expect("safepoint id should fit usize"),
            ) {
                *slot = Some(snapshot.clone());
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
            source_maps_by_offset,
            safepoints_by_offset,
            safepoints_by_id,
            deopt_by_safepoint_id,
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
        self.source_maps_by_offset
            .get(usize::try_from(instruction_offset).ok()?)
            .copied()
            .flatten()
    }

    #[inline]
    pub(super) fn safepoint(
        &self,
        instruction_offset: u32,
    ) -> Option<lyng_js_bytecode::SafepointDescriptor> {
        self.safepoints_by_offset
            .get(usize::try_from(instruction_offset).ok()?)
            .copied()
            .flatten()
    }

    #[inline]
    pub(super) fn safepoint_by_id(
        &self,
        safepoint_id: u32,
    ) -> Option<lyng_js_bytecode::SafepointDescriptor> {
        self.safepoints_by_id
            .get(usize::try_from(safepoint_id.saturating_sub(1)).ok()?)
            .copied()
            .flatten()
    }

    #[inline]
    pub(super) fn deopt_snapshot(
        &self,
        safepoint_id: u32,
    ) -> Option<&lyng_js_bytecode::DeoptSnapshot> {
        self.deopt_by_safepoint_id
            .get(usize::try_from(safepoint_id.saturating_sub(1)).ok()?)?
            .as_ref()
    }

    #[inline]
    pub(super) fn canonical_atom(&self, atom: AtomId) -> AtomId {
        self.canonical_atoms
            .get(usize::try_from(atom.raw()).unwrap_or(usize::MAX))
            .copied()
            .flatten()
            .unwrap_or(atom)
    }
}

impl Vm {
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
            canonical_atoms[usize::try_from(atom.raw()).expect("atom id should fit into usize")] =
                Some(runtime_atom);
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
                atom_texts,
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
                .clone()
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
        atom_texts: &[(AtomId, CompiledAtom)],
        canonical_atoms: &[Option<AtomId>],
    ) -> Option<lyng_js_gc::CodeSlotsRef> {
        if constants.is_empty() {
            return None;
        }

        let mut mutator = agent.heap_mut().mutator();
        let slots = mutator.alloc_code_slots(
            constants.len(),
            Value::empty_internal_slot(),
            AllocationLifetime::Default,
        );
        drop(mutator);

        for (index, constant) in constants.iter().copied().enumerate() {
            if let Some(value) =
                self.constant_value(agent, realm, constant, atom_texts, canonical_atoms)
            {
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
        self.installed[index] = Some(Arc::new(installed));
    }
}
