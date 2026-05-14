use super::{Agent, AllocationLifetime, FrameRecord, Value, Vm, VmResult, WithEnvironmentState};

impl Vm {
    pub(super) fn push_with_environment(
        &mut self,
        agent: &mut Agent,
        frame: &mut FrameRecord,
        value: Value,
    ) -> VmResult<()> {
        let previous_lexical_env = frame.lexical_env();
        let outer = self
            .active_loop_iteration_environment(previous_lexical_env)
            .unwrap_or(previous_lexical_env);
        let binding_object = Self::to_object_for_value(agent, frame.realm(), value)?;
        let with_environment = agent.alloc_object_environment(
            Some(outer),
            binding_object,
            true,
            AllocationLifetime::Default,
        );
        self.with_environment_states.push(WithEnvironmentState {
            frame_depth: self.frames.len(),
            previous_lexical_env,
        });
        frame.set_lexical_env(with_environment);
        Ok(())
    }

    pub(super) fn pop_with_environment(&mut self, frame: &mut FrameRecord) {
        let frame_depth = self.frames.len();
        let Some(index) = self
            .with_environment_states
            .iter()
            .rposition(|state| state.frame_depth == frame_depth)
        else {
            return;
        };
        let state = self.with_environment_states.remove(index);
        frame.set_lexical_env(state.previous_lexical_env);
    }

    pub(super) fn close_with_environment_frames(&mut self, frame_depth: usize) {
        while self
            .with_environment_states
            .last()
            .is_some_and(|state| state.frame_depth > frame_depth)
        {
            let _ = self.with_environment_states.pop();
        }
    }

    pub(super) fn drain_with_environment_state(
        &mut self,
        frame_depth: usize,
    ) -> Vec<WithEnvironmentState> {
        let mut states = Vec::new();
        let mut index = 0;
        while index < self.with_environment_states.len() {
            if self.with_environment_states[index].frame_depth == frame_depth {
                states.push(self.with_environment_states.remove(index));
            } else {
                index += 1;
            }
        }
        states
    }

    pub(super) fn restore_with_environment_state(
        &mut self,
        frame_depth: usize,
        mut states: Vec<WithEnvironmentState>,
    ) {
        for state in &mut states {
            state.frame_depth = frame_depth;
        }
        self.with_environment_states.extend(states);
    }
}
