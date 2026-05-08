use lyng_js_common::AtomId;
use lyng_js_env::PromiseCapabilityId;
use lyng_js_gc::{PrimitiveTracer, TraceHeapEdges};
use lyng_js_host::ModuleSourceRequest;
use lyng_js_types::{CodeRef, EnvironmentRef, ObjectRef, RealmRef, Value};

use crate::frame::GeneratorResumeKind;
use crate::name_refs::CapturedNameReference;
use crate::{FrameRecord, Vm};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(in crate::vm) struct TemplateCacheKey {
    pub(in crate::vm) realm: RealmRef,
    pub(in crate::vm) code: CodeRef,
    pub(in crate::vm) site: u32,
}

/// Runtime state for one compiler-planned loop-iteration lexical environment.
///
/// `iteration_slots` mirror per-iteration bindings while the loop body is
/// active unless they are detached normal-for copies. `shared_slots` continue
/// to alias the source environment after the per-iteration environment is
/// retained by a closure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::vm) struct LoopIterationEnvironment {
    pub(in crate::vm) frame_depth: usize,
    pub(in crate::vm) source_environment: EnvironmentRef,
    pub(in crate::vm) iteration_environment: EnvironmentRef,
    pub(in crate::vm) iteration_slots: Vec<u32>,
    pub(in crate::vm) shared_slots: Vec<u32>,
    pub(in crate::vm) detached_slots: Vec<u32>,
    pub(in crate::vm) active: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::vm) struct WithEnvironmentState {
    pub(in crate::vm) frame_depth: usize,
    pub(in crate::vm) previous_lexical_env: EnvironmentRef,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::vm) struct DirectEvalEnvironmentState {
    pub(in crate::vm) frame_depth: usize,
    pub(in crate::vm) environment: EnvironmentRef,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::vm) struct AsyncFrameState {
    pub(in crate::vm) capability: PromiseCapabilityId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::vm) struct AsyncGeneratorRequest {
    pub(in crate::vm) kind: GeneratorResumeKind,
    pub(in crate::vm) value: Value,
    pub(in crate::vm) capability: PromiseCapabilityId,
    pub(in crate::vm) realm: RealmRef,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::vm) struct AsyncGeneratorFrameState {
    pub(in crate::vm) generator: ObjectRef,
    pub(in crate::vm) capability: PromiseCapabilityId,
    pub(in crate::vm) realm: RealmRef,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::vm) struct EntryExecutionOverride {
    pub(in crate::vm) this_value: Value,
    pub(in crate::vm) new_target: Option<ObjectRef>,
    pub(in crate::vm) home_object: Option<ObjectRef>,
    pub(in crate::vm) active_function: Option<ObjectRef>,
    pub(in crate::vm) private_env: Option<EnvironmentRef>,
    pub(in crate::vm) lexical_this: bool,
}

pub(in crate::vm) struct ActiveVmRoots<'a> {
    pub(in crate::vm) vm: &'a Vm,
    pub(in crate::vm) caller_frame: FrameRecord,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::vm) struct DynamicImportRequest {
    pub(in crate::vm) capability: PromiseCapabilityId,
    pub(in crate::vm) request: ModuleSourceRequest,
    pub(in crate::vm) phase: DynamicImportPhase,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::vm) enum DynamicImportPhase {
    Evaluation,
    Source,
    Defer,
}

impl DynamicImportPhase {
    pub(in crate::vm) fn from_value(value: Option<Value>) -> Self {
        match value.and_then(Value::as_smi) {
            Some(1) => Self::Source,
            Some(2) => Self::Defer,
            _ => Self::Evaluation,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::vm) struct PendingDynamicImport {
    pub(in crate::vm) capability: PromiseCapabilityId,
    pub(in crate::vm) realm: RealmRef,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::vm) struct SuspendedExecutionSideState {
    pub(in crate::vm) iterator_states: Vec<(u16, lyng_js_ops::iterator::IteratorRecord)>,
    pub(in crate::vm) for_in_states: Vec<(u16, lyng_js_ops::enumeration::ForInEnumerator)>,
    pub(in crate::vm) captured_name_references: Vec<(u16, CapturedNameReference)>,
    pub(in crate::vm) loop_iteration_envs: Vec<LoopIterationEnvironment>,
    pub(in crate::vm) with_environment_states: Vec<WithEnvironmentState>,
    pub(in crate::vm) direct_eval_environment_states: Vec<DirectEvalEnvironmentState>,
    pub(in crate::vm) active_env_scopes: Vec<ActiveEnvScopeRange>,
    pub(in crate::vm) async_frame_state: Option<AsyncFrameState>,
    pub(in crate::vm) async_generator_frame_state: Option<AsyncGeneratorFrameState>,
    pub(in crate::vm) script_or_module_referrer: Option<AtomId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::vm) struct ActiveEnvScopeRange {
    pub(in crate::vm) frame_depth: usize,
    pub(in crate::vm) environment: EnvironmentRef,
    pub(in crate::vm) start: u32,
    pub(in crate::vm) end: u32,
}

impl ActiveEnvScopeRange {
    pub(in crate::vm) const fn new(
        frame_depth: usize,
        environment: EnvironmentRef,
        start: u32,
        count: u32,
    ) -> Self {
        Self {
            frame_depth,
            environment,
            start,
            end: start.saturating_add(count),
        }
    }

    pub(in crate::vm) fn contains(self, environment: EnvironmentRef, slot: u32) -> bool {
        self.environment == environment && self.start <= slot && slot < self.end
    }
}

impl TraceHeapEdges for TemplateCacheKey {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.realm.trace_heap_edges(tracer);
        self.code.trace_heap_edges(tracer);
    }
}

impl TraceHeapEdges for LoopIterationEnvironment {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.source_environment.trace_heap_edges(tracer);
        self.iteration_environment.trace_heap_edges(tracer);
    }
}

impl TraceHeapEdges for WithEnvironmentState {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.previous_lexical_env.trace_heap_edges(tracer);
    }
}

impl TraceHeapEdges for DirectEvalEnvironmentState {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.environment.trace_heap_edges(tracer);
    }
}

impl TraceHeapEdges for AsyncGeneratorRequest {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.value.trace_heap_edges(tracer);
        self.realm.trace_heap_edges(tracer);
    }
}

impl TraceHeapEdges for AsyncGeneratorFrameState {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.generator.trace_heap_edges(tracer);
        self.realm.trace_heap_edges(tracer);
    }
}

impl TraceHeapEdges for EntryExecutionOverride {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.this_value.trace_heap_edges(tracer);
        self.new_target.trace_heap_edges(tracer);
        self.home_object.trace_heap_edges(tracer);
        self.active_function.trace_heap_edges(tracer);
        self.private_env.trace_heap_edges(tracer);
    }
}

impl TraceHeapEdges for ActiveVmRoots<'_> {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        trace_frame_record(self.caller_frame, tracer);

        for value in &self.vm.register_stack {
            value.trace_heap_edges(tracer);
        }
        for frame in &self.vm.frames {
            trace_frame_record(*frame, tracer);
        }
        self.vm.current_exception.trace_heap_edges(tracer);

        for installed in self.vm.installed.iter().flatten() {
            for code in &installed.child_codes {
                code.trace_heap_edges(tracer);
            }
        }
        self.vm.builtin_cache.trace_heap_edges(tracer);
        for (key, value) in &self.vm.template_cache {
            key.trace_heap_edges(tracer);
            value.trace_heap_edges(tracer);
        }
        for code in self.vm.dynamic_function_cache.values() {
            code.code().trace_heap_edges(tracer);
        }
        for state in self.vm.suspended_side_states.values() {
            for (_, iterator) in &state.iterator_states {
                iterator.trace_heap_edges(tracer);
            }
            for (_, enumerator) in &state.for_in_states {
                enumerator.trace_heap_edges(tracer);
            }
            for state in &state.loop_iteration_envs {
                state.trace_heap_edges(tracer);
            }
            for state in &state.with_environment_states {
                state.trace_heap_edges(tracer);
            }
            for state in &state.direct_eval_environment_states {
                state.trace_heap_edges(tracer);
            }
            for state in &state.active_env_scopes {
                state.environment.trace_heap_edges(tracer);
            }
            state.async_generator_frame_state.trace_heap_edges(tracer);
        }
        for state in self.vm.async_generator_frame_states.values() {
            state.trace_heap_edges(tracer);
        }
        for object in &self.vm.async_generator_objects {
            object.trace_heap_edges(tracer);
        }
        for (object, queue) in &self.vm.async_generator_queues {
            object.trace_heap_edges(tracer);
            for request in queue {
                request.trace_heap_edges(tracer);
            }
        }
        for object in self.vm.deferred_module_namespaces.keys() {
            object.trace_heap_edges(tracer);
        }
        for state in &self.vm.loop_iteration_envs {
            state.trace_heap_edges(tracer);
        }
        for environment in &self.vm.loop_iteration_source_scratch {
            environment.trace_heap_edges(tracer);
        }
        for environment in &self.vm.loop_iteration_target_scratch {
            environment.trace_heap_edges(tracer);
        }
        for state in &self.vm.with_environment_states {
            state.trace_heap_edges(tracer);
        }
        for state in &self.vm.direct_eval_environment_states {
            state.trace_heap_edges(tracer);
        }
        for state in &self.vm.active_env_scopes {
            state.environment.trace_heap_edges(tracer);
        }
        for (overlay, source) in &self.vm.direct_eval_environment_overlays {
            overlay.trace_heap_edges(tracer);
            source.trace_heap_edges(tracer);
        }
        for value in &self.vm.argument_scratch {
            value.trace_heap_edges(tracer);
        }
    }
}

fn trace_frame_record(frame: FrameRecord, tracer: &mut PrimitiveTracer<'_>) {
    frame.code().trace_heap_edges(tracer);
    frame.realm().trace_heap_edges(tracer);
    frame.lexical_env().trace_heap_edges(tracer);
    frame.variable_env().trace_heap_edges(tracer);
    frame.this_value().trace_heap_edges(tracer);
    frame.construct_this().trace_heap_edges(tracer);
    frame.new_target().trace_heap_edges(tracer);
    frame.callee().trace_heap_edges(tracer);
    frame.tail_caller().trace_heap_edges(tracer);
    frame.resume_value().trace_heap_edges(tracer);
}
