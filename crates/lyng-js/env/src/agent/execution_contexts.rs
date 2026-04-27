use super::Agent;
use crate::{CodeRef, EnvironmentRef, ExecutableId, ExecutionContext, ThisState};
use lyng_js_types::RealmRef;

impl Agent {
    #[inline]
    pub fn execution_contexts(&self) -> &[ExecutionContext] {
        &self.execution_contexts
    }

    #[inline]
    pub fn current_execution_context(&self) -> Option<ExecutionContext> {
        self.execution_contexts.last().copied()
    }

    pub fn push_execution_context(&mut self, context: ExecutionContext) {
        self.execution_contexts.push(context);
    }

    pub fn set_current_execution_context_this_state(&mut self, this_state: ThisState) -> bool {
        let Some(context) = self.execution_contexts.last_mut() else {
            return false;
        };
        *context = context.with_this_state(this_state);
        true
    }

    pub fn set_execution_context_this_state_for_lexical_env(
        &mut self,
        lexical_env: EnvironmentRef,
        this_state: ThisState,
    ) -> bool {
        let Some(context) = self
            .execution_contexts
            .iter_mut()
            .rev()
            .find(|context| context.lexical_env() == lexical_env)
        else {
            return false;
        };
        *context = context.with_this_state(this_state);
        true
    }

    pub fn push_script_context(
        &mut self,
        realm: RealmRef,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
    ) {
        self.push_execution_context(ExecutionContext::script(realm, lexical_env, variable_env));
    }

    pub fn push_module_context(
        &mut self,
        realm: RealmRef,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
    ) {
        self.push_execution_context(ExecutionContext::module(realm, lexical_env, variable_env));
    }

    pub fn push_builtin_context(
        &mut self,
        realm: RealmRef,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
    ) {
        self.push_execution_context(ExecutionContext::builtin(realm, lexical_env, variable_env));
    }

    pub fn push_eval_context(
        &mut self,
        realm: RealmRef,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
    ) {
        self.push_execution_context(ExecutionContext::eval(realm, lexical_env, variable_env));
    }

    pub fn push_job_context(
        &mut self,
        realm: RealmRef,
        executable: ExecutableId,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
    ) {
        self.push_execution_context(ExecutionContext::job(
            realm,
            executable,
            lexical_env,
            variable_env,
        ));
    }

    pub fn push_bytecode_context(
        &mut self,
        realm: RealmRef,
        code: CodeRef,
        lexical_env: EnvironmentRef,
        variable_env: EnvironmentRef,
    ) {
        self.push_execution_context(ExecutionContext::bytecode(
            realm,
            code,
            lexical_env,
            variable_env,
        ));
    }

    pub fn pop_execution_context(&mut self) -> Option<ExecutionContext> {
        self.execution_contexts.pop()
    }
}
