use super::{
    code_index, Agent, AllocationLifetime, AtomId, DirectEvalEnvironmentState, EnvironmentLayoutId,
    EnvironmentRef, FrameRecord, Vm, VmError, VmResult,
};
use lyng_js_bytecode::DirectEvalSiteFlags;
use lyng_js_env::{
    EnvironmentBindingLayout, EnvironmentLayout, EnvironmentLayoutKind, EnvironmentSlotFlags,
};

impl Vm {
    fn direct_eval_lexical_layout(
        &mut self,
        agent: &mut Agent,
        bindings: &[lyng_js_bytecode::BytecodeEnvironmentBinding],
    ) -> EnvironmentLayoutId {
        if let Some(layout) = self.direct_eval_lexical_layouts.get(bindings).copied() {
            return layout;
        }

        let layout = agent.alloc_environment_layout(EnvironmentLayout::new(
            EnvironmentLayoutKind::Declarative,
            bindings
                .iter()
                .copied()
                .map(|binding| {
                    EnvironmentBindingLayout::new(
                        binding.name(),
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
                .collect::<Vec<_>>(),
            true,
        ));
        self.direct_eval_lexical_layouts
            .insert(bindings.to_vec(), layout);
        layout
    }

    fn environment_outer(
        &self,
        agent: &Agent,
        environment: EnvironmentRef,
    ) -> Option<EnvironmentRef> {
        let record = agent.environment(environment)?;
        Some(match record {
            lyng_js_env::EnvironmentRecord::Declarative(record) => record.outer()?,
            lyng_js_env::EnvironmentRecord::Private(record) => record.outer()?,
            lyng_js_env::EnvironmentRecord::Function(record) => record.declarative().outer()?,
            lyng_js_env::EnvironmentRecord::Module(record) => record.outer()?,
            lyng_js_env::EnvironmentRecord::Global(record) => record.outer()?,
            lyng_js_env::EnvironmentRecord::Object(record) => record.outer()?,
        })
    }

    fn environment_matches_direct_eval_scope(
        &self,
        agent: &Agent,
        environment: EnvironmentRef,
        scope: &lyng_js_bytecode::DirectEvalLexicalScope,
    ) -> bool {
        let Some(record) = agent.environment(environment) else {
            return false;
        };
        let Some(layout_id) = (match record {
            lyng_js_env::EnvironmentRecord::Declarative(record) => Some(record.layout()),
            lyng_js_env::EnvironmentRecord::Function(record) => Some(record.declarative().layout()),
            lyng_js_env::EnvironmentRecord::Module(record) => Some(record.layout()),
            lyng_js_env::EnvironmentRecord::Global(record) => Some(record.layout()),
            lyng_js_env::EnvironmentRecord::Private(_)
            | lyng_js_env::EnvironmentRecord::Object(_) => None,
        }) else {
            return false;
        };
        let Some(layout) = agent.environment_layout(layout_id) else {
            return false;
        };
        scope.bindings().iter().enumerate().all(|(index, binding)| {
            let Some(slot) = scope
                .source_base()
                .checked_add(u32::try_from(index).unwrap_or(u32::MAX))
            else {
                return false;
            };
            let Some(layout_binding) = layout.binding(slot) else {
                return false;
            };
            layout_binding.name() == binding.name()
                && layout_binding.flags().is_mutable() == binding.flags().is_mutable()
                && layout_binding.flags().is_lexical() == binding.flags().is_lexical()
                && layout_binding.flags().needs_tdz() == binding.flags().needs_tdz()
                && layout_binding.flags().is_dynamic() == binding.flags().is_dynamic()
                && layout_binding.flags().sloppy_immutable_assign_silent()
                    == binding.flags().sloppy_immutable_assign_silent()
        })
    }

    fn direct_eval_scope_source_environment(
        &self,
        agent: &Agent,
        start: EnvironmentRef,
        scope: &lyng_js_bytecode::DirectEvalLexicalScope,
    ) -> Option<EnvironmentRef> {
        let mut current = Some(start);
        while let Some(environment) = current {
            if self.environment_matches_direct_eval_scope(agent, environment, scope) {
                return Some(environment);
            }
            current = self.environment_outer(agent, environment);
        }
        None
    }

    pub(super) fn caller_direct_eval_lexical_environment(
        &mut self,
        agent: &mut Agent,
        caller: FrameRecord,
        lexical_env: EnvironmentRef,
    ) -> VmResult<(
        EnvironmentRef,
        DirectEvalSiteFlags,
        Vec<(EnvironmentRef, u32, EnvironmentRef, u32, AtomId)>,
        Vec<AtomId>,
        Vec<AtomId>,
    )> {
        let Some(installed) = self
            .installed
            .get(code_index(caller.code()))
            .and_then(Option::as_ref)
        else {
            return Ok((
                lexical_env,
                DirectEvalSiteFlags::empty(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ));
        };
        let Some(site) = installed
            .direct_eval_lexical_site(caller.instruction_offset())
            .cloned()
        else {
            return Ok((
                lexical_env,
                DirectEvalSiteFlags::empty(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ));
        };

        let source_start = lexical_env;
        let mut current_outer = lexical_env;
        let mut annex_b_catch_environments = Vec::new();
        let annex_b_catch_names = site.annex_b_catch_names().to_vec();
        let parameter_names = site.parameter_names().to_vec();
        for scope in site.scopes() {
            let source_environment = self
                .direct_eval_scope_source_environment(agent, source_start, scope)
                .ok_or(VmError::MissingEnvironment(lexical_env))?;
            let layout = self.direct_eval_lexical_layout(agent, scope.bindings());
            let environment = agent
                .alloc_declarative_environment(
                    Some(current_outer),
                    layout,
                    AllocationLifetime::Default,
                )
                .ok_or_else(|| VmError::MissingEnvironmentLayout(caller.code()))?;
            for (index, _) in scope.bindings().iter().enumerate() {
                let slot = u32::try_from(index).unwrap_or(u32::MAX);
                let source_slot = scope
                    .source_base()
                    .checked_add(slot)
                    .ok_or(VmError::MissingEnvironment(source_environment))?;
                let value = agent
                    .environment_slot(source_environment, source_slot)
                    .ok_or(VmError::MissingEnvironment(source_environment))?;
                if !agent.set_environment_slot(environment, slot, value) {
                    return Err(VmError::MissingEnvironment(environment));
                }
            }
            if let Some(name) = scope.annex_b_catch_name()
                && let Some((index, _)) = scope
                    .bindings()
                    .iter()
                    .enumerate()
                    .find(|(_, binding)| binding.name() == Some(name))
            {
                let cloned_slot = u32::try_from(index).unwrap_or(u32::MAX);
                let source_slot = scope
                    .source_base()
                    .checked_add(cloned_slot)
                    .ok_or(VmError::MissingEnvironment(source_environment))?;
                annex_b_catch_environments.push((
                    source_environment,
                    source_slot,
                    environment,
                    cloned_slot,
                    name,
                ));
            }
            current_outer = environment;
        }

        Ok((
            current_outer,
            site.flags(),
            annex_b_catch_environments,
            annex_b_catch_names,
            parameter_names,
        ))
    }

    pub(super) fn active_direct_eval_environment(
        &self,
        frame_depth: usize,
    ) -> Option<EnvironmentRef> {
        self.direct_eval_environment_states
            .iter()
            .rev()
            .find(|state| state.frame_depth == frame_depth)
            .map(|state| state.environment)
    }

    pub(super) fn direct_eval_environment_overlay(
        &self,
        source: EnvironmentRef,
    ) -> Option<EnvironmentRef> {
        self.direct_eval_environment_overlays.get(&source).copied()
    }

    pub(super) fn register_direct_eval_environment_overlay(
        &mut self,
        source: EnvironmentRef,
        overlay: EnvironmentRef,
    ) {
        self.direct_eval_environment_overlays
            .insert(source, overlay);
    }

    pub(super) fn push_direct_eval_environment(
        &mut self,
        frame_depth: usize,
        environment: EnvironmentRef,
    ) {
        self.direct_eval_environment_states
            .push(DirectEvalEnvironmentState {
                frame_depth,
                environment,
            });
    }

    pub(super) fn lexical_name_start_environment(&self, frame: FrameRecord) -> EnvironmentRef {
        self.active_loop_iteration_environment(frame.lexical_env())
            .unwrap_or_else(|| frame.lexical_env())
    }

    pub(super) fn dynamic_name_start_environment(
        &self,
        agent: &Agent,
        frame: FrameRecord,
    ) -> EnvironmentRef {
        let lexical_env = self.lexical_name_start_environment(frame);
        if matches!(
            agent.environment(lexical_env),
            Some(lyng_js_env::EnvironmentRecord::Object(_))
        ) {
            return lexical_env;
        }
        self.active_direct_eval_environment(self.frames.len())
            .unwrap_or(lexical_env)
    }

    pub(super) fn close_direct_eval_frames(&mut self, frame_depth: usize) {
        while self
            .direct_eval_environment_states
            .last()
            .is_some_and(|state| state.frame_depth > frame_depth)
        {
            let _ = self.direct_eval_environment_states.pop();
        }
    }

    pub(super) fn drain_direct_eval_environment_state(
        &mut self,
        frame_depth: usize,
    ) -> Vec<DirectEvalEnvironmentState> {
        let mut states = Vec::new();
        let mut index = 0;
        while index < self.direct_eval_environment_states.len() {
            if self.direct_eval_environment_states[index].frame_depth == frame_depth {
                states.push(self.direct_eval_environment_states.remove(index));
            } else {
                index += 1;
            }
        }
        states
    }

    pub(super) fn restore_direct_eval_environment_state(
        &mut self,
        frame_depth: usize,
        mut states: Vec<DirectEvalEnvironmentState>,
    ) {
        for state in &mut states {
            state.frame_depth = frame_depth;
        }
        self.direct_eval_environment_states.extend(states);
    }
}
