use lyng_common::AtomId;
use lyng_env::PromiseCapabilityId;
use lyng_gc::{PrimitiveTracer, TraceHeapEdges};
use lyng_host::ModuleSourceRequest;
use lyng_types::{CodeRef, EnvironmentRef, ObjectRef, RealmRef, Value};

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
pub struct EntryExecutionOverride {
    pub(in crate::vm) this_value: Value,
    pub(in crate::vm) new_target: Option<ObjectRef>,
    pub(in crate::vm) home_object: Option<ObjectRef>,
    pub(in crate::vm) active_function: Option<ObjectRef>,
    pub(in crate::vm) private_env: Option<EnvironmentRef>,
    pub(in crate::vm) lexical_this: bool,
}

pub(in crate::vm) struct ActiveVmRoots<'a> {
    pub(in crate::vm) vm: &'a Vm,
    pub(in crate::vm) caller_frame: &'a FrameRecord,
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
    pub(in crate::vm) iterator_states: Vec<(u16, lyng_ops::iterator::IteratorRecord)>,
    pub(in crate::vm) for_in_states: Vec<(u16, lyng_ops::enumeration::ForInEnumerator)>,
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
        // Trace the LIVE arena frames via the cfr-walk. The frame layout is
        // `[FrameHeader(7 slots)][window]`; header slots 0-1 and 3-6 hold packed
        // integers (NOT valid `Value`s) — only slot 2 (`this_value`) is a real
        // `Value`. So (a) scan each frame's WINDOW as `Value`s (never header slots)
        // and (b) trace every header heap ref through its typed getter.
        for cfr in self.vm.frame_cfrs() {
            // (a) Register window only.
            let base = cfr as usize + crate::frame_header::HEADER_SLOTS;
            let len = usize::from(self.vm.frame_window_len(cfr));
            for value in &self.vm.arena_slots()[base..base + len] {
                value.trace_heap_edges(tracer);
            }
            // (b) Typed header refs (packed u32s not covered by the window scan)
            // plus `this_value`.
            let header = self.vm.frame_header(cfr);
            header.code().trace_heap_edges(tracer);
            header.callee().trace_heap_edges(tracer);
            header.variable_env().trace_heap_edges(tracer);
            header.private_env().trace_heap_edges(tracer);
            header.new_target().trace_heap_edges(tracer);
            header.lexical_env().trace_heap_edges(tracer);
            header.construct_this().trace_heap_edges(tracer);
            header.this_value().trace_heap_edges(tracer);
        }
        // `tail_caller`/`resume_value` live in the cold side-table.
        for cold in self.vm.frame_cold_live_slots() {
            cold.tail_caller.trace_heap_edges(tracer);
            cold.resume_value.trace_heap_edges(tracer);
        }
        // Function-frame realms are reachable via the traced `callee`; callee-less
        // root realms live on the establishment side-stack.
        for realm in self.vm.establishment_realms() {
            realm.trace_heap_edges(tracer);
        }
        // Trace the active `caller_frame`. For frames already pushed into the arena
        // this is harmless over-tracing (covered by the cfr-walk above). For
        // synthetic frames (e.g. `force_collect`) never pushed into the arena,
        // this is the ONLY trace. Conservative: over-trace beats under-trace.
        // The realm is NOT traced off the record; it is already rooted either via
        // the traced `callee` (function frame) or via the establishment side-stack
        // (callee-less / synthetic frames).
        trace_all_frame_edges(self.caller_frame, tracer);
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

/// Trace every heap edge a `FrameRecord` snapshot holds, for the synthetic
/// `caller_frame` path where the record (not a live arena frame) is authoritative.
///
/// Trace all heap fields of a `FrameRecord`. `realm` is not traced (rooted via
/// the establishment side-stack or `callee`). Used only for the `caller_frame`
/// snapshot that may never have been pushed into the arena.
fn trace_all_frame_edges(frame: &FrameRecord, tracer: &mut PrimitiveTracer<'_>) {
    frame.variable_env().trace_heap_edges(tracer);
    frame.private_env().trace_heap_edges(tracer);
    frame.new_target().trace_heap_edges(tracer);
    frame.callee().trace_heap_edges(tracer);
    frame.code().trace_heap_edges(tracer);
    frame.this_value().trace_heap_edges(tracer);
    frame.lexical_env().trace_heap_edges(tracer);
    frame.construct_this().trace_heap_edges(tracer);
    frame.tail_caller().trace_heap_edges(tracer);
    frame.resume_value().trace_heap_edges(tracer);
}
