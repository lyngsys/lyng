use super::*;
use lyng_js_bytecode::{CaptureDescriptor, CaptureSource};

impl Vm {
    pub(super) fn push_loop_iteration_environment(
        &mut self,
        agent: &mut Agent,
        frame: FrameRecord,
        site: Option<lyng_js_bytecode::LoopIterationEnvironmentSite>,
        mirrored_slot: Option<u32>,
    ) -> VmResult<()> {
        let source_environment = frame.lexical_env();
        let (iteration_slots, shared_slots) = match site {
            Some(site) => (
                site.iteration_slots()
                    .iter()
                    .map(|slot| u32::from(*slot))
                    .collect::<Vec<_>>(),
                site.shared_slots()
                    .iter()
                    .map(|slot| u32::from(*slot))
                    .collect::<Vec<_>>(),
            ),
            None => (mirrored_slot.into_iter().collect(), Vec::new()),
        };
        let iteration_environment =
            self.create_loop_iteration_environment(agent, source_environment, &iteration_slots)?;
        self.loop_iteration_envs.push(LoopIterationEnvironment {
            frame_depth: self.frames.len(),
            source_environment,
            iteration_environment,
            iteration_slots,
            shared_slots,
            active: true,
        });
        Ok(())
    }

    pub(super) fn pop_loop_iteration_environment(&mut self) {
        if let Some(index) = self.loop_iteration_envs.iter().rposition(|environment| {
            environment.active && environment.frame_depth == self.frames.len()
        }) {
            if self.loop_iteration_envs[index].shared_slots.is_empty() {
                let _ = self.loop_iteration_envs.remove(index);
                return;
            }
            self.loop_iteration_envs[index].iteration_slots.clear();
            self.loop_iteration_envs[index].active = false;
        }
    }

    pub(super) fn close_loop_iteration_frames(&mut self, frame_depth: usize) {
        let mut index = 0;
        while index < self.loop_iteration_envs.len() {
            let environment = &mut self.loop_iteration_envs[index];
            if !environment.active || environment.frame_depth <= frame_depth {
                index += 1;
                continue;
            }
            if environment.shared_slots.is_empty() {
                let _ = self.loop_iteration_envs.remove(index);
                continue;
            }
            environment.iteration_slots.clear();
            environment.active = false;
            index += 1;
        }
    }

    pub(super) fn drain_loop_iteration_state(
        &mut self,
        frame_depth: usize,
    ) -> Vec<LoopIterationEnvironment> {
        let mut drained = Vec::new();
        let mut index = 0;
        while index < self.loop_iteration_envs.len() {
            if self.loop_iteration_envs[index].frame_depth == frame_depth {
                drained.push(self.loop_iteration_envs.remove(index));
            } else {
                index += 1;
            }
        }
        drained
    }

    pub(super) fn restore_loop_iteration_state(
        &mut self,
        frame_depth: usize,
        mut environments: Vec<LoopIterationEnvironment>,
    ) {
        for environment in &mut environments {
            environment.frame_depth = frame_depth;
        }
        self.loop_iteration_envs.extend(environments);
    }

    fn create_loop_iteration_environment(
        &mut self,
        agent: &mut Agent,
        environment: EnvironmentRef,
        iteration_slots: &[u32],
    ) -> VmResult<EnvironmentRef> {
        let active_iteration_outer = self.active_loop_iteration_environment(environment);
        let (outer, source_layout) = match agent.environment(environment) {
            Some(lyng_js_env::EnvironmentRecord::Declarative(record)) => (
                active_iteration_outer.or(record.outer()),
                Some(record.layout()),
            ),
            Some(lyng_js_env::EnvironmentRecord::Private(record)) => (
                active_iteration_outer.or(record.outer()),
                Some(record.layout()),
            ),
            Some(lyng_js_env::EnvironmentRecord::Function(record)) => {
                let declarative = record.declarative();
                (
                    active_iteration_outer.or(declarative.outer()),
                    Some(declarative.layout()),
                )
            }
            Some(lyng_js_env::EnvironmentRecord::Module(record)) => (
                active_iteration_outer.or(record.outer()),
                Some(record.layout()),
            ),
            Some(lyng_js_env::EnvironmentRecord::Global(record)) => (
                active_iteration_outer.or(record.outer()),
                Some(record.layout()),
            ),
            Some(lyng_js_env::EnvironmentRecord::Object(record)) => {
                (active_iteration_outer.or(record.outer()), None)
            }
            None => return Err(VmError::MissingEnvironment(environment)),
        };
        let layout = self.loop_iteration_layout(agent, source_layout);
        let iteration_environment = agent
            .alloc_declarative_environment(outer, layout, AllocationLifetime::Default)
            .ok_or(VmError::MissingEnvironment(environment))?;
        if let Some(source_layout) = source_layout {
            let slot_count = agent
                .environment_layout(source_layout)
                .map(|layout| layout.slot_count())
                .unwrap_or(0);
            for slot in 0..slot_count {
                if iteration_slots.contains(&slot) {
                    continue;
                }
                self.copy_environment_slot(agent, environment, iteration_environment, slot)?;
            }
        }
        Ok(iteration_environment)
    }

    fn loop_iteration_layout(
        &mut self,
        agent: &mut Agent,
        source_layout: Option<EnvironmentLayoutId>,
    ) -> EnvironmentLayoutId {
        if let Some(layout) = self.loop_iteration_layouts.get(&source_layout).copied() {
            return layout;
        }
        let bindings = source_layout
            .and_then(|layout| {
                agent
                    .environment_layout(layout)
                    .map(|layout| layout.bindings().to_vec())
            })
            .unwrap_or_default();
        let layout = agent.alloc_environment_layout(EnvironmentLayout::new(
            EnvironmentLayoutKind::Declarative,
            bindings,
            true,
        ));
        self.loop_iteration_layouts.insert(source_layout, layout);
        layout
    }

    fn loop_iteration_sources(
        &self,
        environment: EnvironmentRef,
        slot: u32,
        active_only: bool,
    ) -> Vec<EnvironmentRef> {
        let mut sources = Vec::new();
        for environment_record in &self.loop_iteration_envs {
            if active_only && !environment_record.active {
                continue;
            }
            let slots = if active_only {
                &environment_record.iteration_slots
            } else {
                &environment_record.shared_slots
            };
            if !slots.contains(&slot) {
                continue;
            }
            if environment_record.source_environment != environment
                && environment_record.iteration_environment != environment
            {
                continue;
            }
            if !sources.contains(&environment_record.source_environment) {
                sources.push(environment_record.source_environment);
            }
        }
        sources
    }

    fn extend_loop_iteration_targets(
        &self,
        targets: &mut Vec<EnvironmentRef>,
        source_environment: EnvironmentRef,
        slot: u32,
        active_only: bool,
    ) {
        if !targets.contains(&source_environment) {
            targets.push(source_environment);
        }
        for environment_record in &self.loop_iteration_envs {
            if environment_record.source_environment != source_environment {
                continue;
            }
            if active_only {
                if !environment_record.active || !environment_record.iteration_slots.contains(&slot)
                {
                    continue;
                }
            } else if !environment_record.shared_slots.contains(&slot) {
                continue;
            }
            if !targets.contains(&environment_record.iteration_environment) {
                targets.push(environment_record.iteration_environment);
            }
        }
    }

    pub(super) fn sync_loop_iteration_slot(
        &mut self,
        agent: &mut Agent,
        environment: EnvironmentRef,
        slot: u32,
        value: Value,
    ) -> VmResult<()> {
        let mut targets = Vec::new();
        for source in self.loop_iteration_sources(environment, slot, true) {
            self.extend_loop_iteration_targets(&mut targets, source, slot, true);
        }
        for source in self.loop_iteration_sources(environment, slot, false) {
            self.extend_loop_iteration_targets(&mut targets, source, slot, false);
        }
        for target in targets {
            if target == environment {
                continue;
            }
            self.mirror_environment_slot(agent, target, slot, value)?;
        }
        Ok(())
    }

    pub(super) fn active_loop_iteration_environment(
        &self,
        environment: EnvironmentRef,
    ) -> Option<EnvironmentRef> {
        self.loop_iteration_envs
            .iter()
            .rev()
            .find(|record| {
                record.active
                    && record.frame_depth == self.frames.len()
                    && record.source_environment == environment
            })
            .map(|record| record.iteration_environment)
    }

    pub(super) fn active_loop_iteration_environment_for_captures(
        &self,
        captures: &[CaptureDescriptor],
    ) -> Option<EnvironmentRef> {
        let active = self
            .loop_iteration_envs
            .iter()
            .rev()
            .find(|record| record.active && record.frame_depth == self.frames.len())?;
        let uses_loop_environment = captures.iter().any(|capture| match capture.source() {
            CaptureSource::EnvironmentSlot { slot, .. } => {
                let slot = u32::from(slot);
                active.iteration_slots.contains(&slot) || active.shared_slots.contains(&slot)
            }
            _ => false,
        });
        uses_loop_environment.then_some(active.iteration_environment)
    }

    pub(super) fn environment_for_slot_access(
        &self,
        agent: &Agent,
        start: EnvironmentRef,
        depth: u8,
        slot: u32,
    ) -> VmResult<EnvironmentRef> {
        let environment = self.environment_at_depth(agent, start, depth)?;
        if depth != 0 {
            return Ok(environment);
        }
        Ok(self
            .loop_iteration_envs
            .iter()
            .rev()
            .find(|record| {
                record.active
                    && record.frame_depth == self.frames.len()
                    && record.source_environment == environment
                    && (record.iteration_slots.contains(&slot)
                        || record.shared_slots.contains(&slot))
            })
            .map(|record| record.iteration_environment)
            .unwrap_or(environment))
    }
}
