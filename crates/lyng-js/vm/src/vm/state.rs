#[cfg(debug_assertions)]
use lyng_js_bytecode::{DeoptFrameValue, DeoptSnapshot, DeoptValueSource, SafepointDescriptor};
use lyng_js_common::AtomId;
use lyng_js_env::PromiseCapabilityId;
use lyng_js_gc::{PrimitiveTracer, TraceHeapEdges};
use lyng_js_host::ModuleSourceRequest;
#[cfg(debug_assertions)]
use lyng_js_types::ShapeId;
use lyng_js_types::{CodeRef, EnvironmentRef, ObjectRef, RealmRef, Value};

#[cfg(debug_assertions)]
use super::install::InstalledFunction;
#[cfg(debug_assertions)]
use super::Agent;
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

        for value in self.vm.register_stack() {
            value.trace_heap_edges(tracer);
        }
        for frame in &self.vm.frames {
            trace_frame_record(frame, tracer);
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

#[cfg(debug_assertions)]
#[derive(Clone, Debug, PartialEq, Eq)]
struct MaterializedRuntimeState {
    lexical_env: Option<EnvironmentRef>,
    variable_env: Option<EnvironmentRef>,
    this_value: Option<Value>,
    new_target: Option<Value>,
    callee: Option<Value>,
    exception_value: Option<Value>,
    completion_state: bool,
}

#[cfg(debug_assertions)]
impl MaterializedRuntimeState {
    fn capture(vm: &Vm, frame: &FrameRecord, safepoint: SafepointDescriptor) -> Self {
        Self {
            lexical_env: safepoint
                .captures_lexical_env()
                .then_some(frame.lexical_env()),
            variable_env: safepoint
                .captures_variable_env()
                .then_some(frame.variable_env()),
            this_value: safepoint.captures_this().then_some(frame.this_value()),
            new_target: safepoint.captures_new_target().then_some(
                frame
                    .new_target()
                    .map_or(Value::undefined(), Value::from_object_ref),
            ),
            callee: safepoint.captures_callee().then_some(
                frame
                    .callee()
                    .map_or(Value::undefined(), Value::from_object_ref),
            ),
            exception_value: safepoint
                .captures_exception_state()
                .then_some(vm.current_exception.unwrap_or(Value::undefined())),
            completion_state: safepoint.captures_completion_state(),
        }
    }

    fn assert_matches_snapshot(
        &self,
        frame: &FrameRecord,
        safepoint: SafepointDescriptor,
        snapshot: &DeoptSnapshot,
    ) {
        assert_frame_value_capture(
            frame,
            safepoint,
            snapshot,
            "this",
            DeoptFrameValue::ThisValue,
            self.this_value.is_some(),
        );
        assert_frame_value_capture(
            frame,
            safepoint,
            snapshot,
            "new-target",
            DeoptFrameValue::NewTarget,
            self.new_target.is_some(),
        );
        assert_frame_value_capture(
            frame,
            safepoint,
            snapshot,
            "callee",
            DeoptFrameValue::Callee,
            self.callee.is_some(),
        );
        assert_frame_value_capture(
            frame,
            safepoint,
            snapshot,
            "exception",
            DeoptFrameValue::ExceptionValue,
            self.exception_value.is_some(),
        );

        if self.lexical_env.is_none() && safepoint.captures_lexical_env() {
            panic_runtime_state_mismatch(frame, safepoint, "lexical environment was not captured");
        }
        if self.variable_env.is_none() && safepoint.captures_variable_env() {
            panic_runtime_state_mismatch(frame, safepoint, "variable environment was not captured");
        }
        if !self.completion_state
            && snapshot
                .values()
                .iter()
                .copied()
                .any(is_completion_frame_value)
        {
            panic_runtime_state_mismatch(
                frame,
                safepoint,
                "completion frame values are present without completion-state capture",
            );
        }
        if safepoint.captures_exception_state() && self.completion_state {
            for value in [
                DeoptFrameValue::CompletionKind,
                DeoptFrameValue::CompletionValue,
                DeoptFrameValue::CompletionTarget,
            ] {
                assert_frame_value_capture(frame, safepoint, snapshot, "completion", value, true);
            }
        }
    }
}

#[cfg(debug_assertions)]
#[derive(Clone, Debug, PartialEq, Eq)]
struct MaterializedDeoptSnapshot {
    values: Vec<MaterializedDeoptValue>,
}

#[cfg(debug_assertions)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MaterializedDeoptValue {
    Value(Value),
    Shape(ShapeId),
}

#[cfg(debug_assertions)]
impl Vm {
    pub(in crate::vm) fn assert_deopt_safepoint_state(
        &self,
        agent: &Agent,
        frame: &FrameRecord,
        installed: &InstalledFunction,
    ) {
        let Some(safepoint) = installed.safepoint(frame.instruction_offset()) else {
            return;
        };

        assert_eq!(
            safepoint.register_window_len(),
            frame.registers().len(),
            "deopt safepoint register-window mismatch: code={:?} offset={} safepoint={} \
             kind={:?} metadata_window={} frame_window={}",
            frame.code(),
            frame.instruction_offset(),
            safepoint.id(),
            safepoint.kind(),
            safepoint.register_window_len(),
            frame.registers().len()
        );

        let Some(snapshot) = installed.deopt_snapshot(safepoint.id()) else {
            panic!(
                "deopt safepoint missing snapshot: code={:?} offset={} safepoint={} kind={:?}",
                frame.code(),
                frame.instruction_offset(),
                safepoint.id(),
                safepoint.kind()
            );
        };

        let runtime_state = MaterializedRuntimeState::capture(self, frame, safepoint);
        runtime_state.assert_matches_snapshot(frame, safepoint, snapshot);

        let materialized =
            self.materialize_deopt_snapshot(agent, frame, installed, safepoint, snapshot);
        debug_assert_eq!(
            materialized.values.len(),
            snapshot.values().len(),
            "deopt materialization should preserve snapshot arity"
        );
    }

    fn materialize_deopt_snapshot(
        &self,
        agent: &Agent,
        frame: &FrameRecord,
        installed: &InstalledFunction,
        safepoint: SafepointDescriptor,
        snapshot: &DeoptSnapshot,
    ) -> MaterializedDeoptSnapshot {
        let values = snapshot
            .values()
            .iter()
            .copied()
            .enumerate()
            .map(|(index, source)| {
                self.materialize_deopt_value(agent, frame, installed, safepoint, index, source)
            })
            .collect();
        MaterializedDeoptSnapshot { values }
    }

    fn materialize_deopt_value(
        &self,
        agent: &Agent,
        frame: &FrameRecord,
        _installed: &InstalledFunction,
        safepoint: SafepointDescriptor,
        source_index: usize,
        source: DeoptValueSource,
    ) -> MaterializedDeoptValue {
        match source {
            DeoptValueSource::Register(register) => {
                assert!(
                    register < frame.registers().len(),
                    "deopt snapshot source out of frame window: code={:?} offset={} \
                     safepoint={} kind={:?} source_index={} register={} frame_window={}",
                    frame.code(),
                    frame.instruction_offset(),
                    safepoint.id(),
                    safepoint.kind(),
                    source_index,
                    register,
                    frame.registers().len()
                );
                MaterializedDeoptValue::Value(self.read_register(frame.registers(), register))
            }
            DeoptValueSource::EnvironmentSlot { depth, slot } => {
                let Ok(depth) = u8::try_from(depth) else {
                    panic!(
                        "deopt snapshot environment depth out of range: code={:?} offset={} \
                         safepoint={} kind={:?} source_index={} depth={}",
                        frame.code(),
                        frame.instruction_offset(),
                        safepoint.id(),
                        safepoint.kind(),
                        source_index,
                        depth
                    );
                };
                let environment = self
                    .environment_for_slot_access(agent, frame.lexical_env(), depth, u32::from(slot))
                    .unwrap_or_else(|error| {
                        panic!(
                            "deopt snapshot environment lookup failed: code={:?} offset={} \
                             safepoint={} kind={:?} source_index={} depth={} slot={} error={:?}",
                            frame.code(),
                            frame.instruction_offset(),
                            safepoint.id(),
                            safepoint.kind(),
                            source_index,
                            depth,
                            slot,
                            error
                        );
                    });
                let value = agent
                    .environment_slot(environment, u32::from(slot))
                    .unwrap_or_else(|| {
                        panic!(
                            "deopt snapshot environment slot missing: code={:?} offset={} \
                             safepoint={} kind={:?} source_index={} environment={:?} slot={}",
                            frame.code(),
                            frame.instruction_offset(),
                            safepoint.id(),
                            safepoint.kind(),
                            source_index,
                            environment,
                            slot
                        );
                    });
                MaterializedDeoptValue::Value(value)
            }
            DeoptValueSource::Constant(index) => {
                let value = self
                    .read_constant(agent, frame.code(), index)
                    .unwrap_or_else(|error| {
                        panic!(
                            "deopt snapshot constant materialization failed: code={:?} offset={} \
                         safepoint={} kind={:?} source_index={} constant={} error={:?}",
                            frame.code(),
                            frame.instruction_offset(),
                            safepoint.id(),
                            safepoint.kind(),
                            source_index,
                            index,
                            error
                        );
                    });
                MaterializedDeoptValue::Value(value)
            }
            DeoptValueSource::Shape(shape) => MaterializedDeoptValue::Shape(shape),
            DeoptValueSource::FrameValue(value) => {
                MaterializedDeoptValue::Value(materialize_deopt_frame_value(self, frame, value))
            }
        }
    }
}

#[cfg(debug_assertions)]
fn materialize_deopt_frame_value(vm: &Vm, frame: &FrameRecord, value: DeoptFrameValue) -> Value {
    match value {
        DeoptFrameValue::ThisValue => frame.this_value(),
        DeoptFrameValue::NewTarget => frame
            .new_target()
            .map_or(Value::undefined(), Value::from_object_ref),
        DeoptFrameValue::Callee => frame
            .callee()
            .map_or(Value::undefined(), Value::from_object_ref),
        DeoptFrameValue::ExceptionValue => vm.current_exception.unwrap_or(Value::undefined()),
        DeoptFrameValue::CompletionKind
        | DeoptFrameValue::CompletionValue
        | DeoptFrameValue::CompletionTarget => Value::undefined(),
    }
}

#[cfg(debug_assertions)]
fn assert_frame_value_capture(
    frame: &FrameRecord,
    safepoint: SafepointDescriptor,
    snapshot: &DeoptSnapshot,
    label: &str,
    value: DeoptFrameValue,
    should_capture: bool,
) {
    let has_source = snapshot
        .values()
        .contains(&DeoptValueSource::FrameValue(value));
    assert_eq!(
        has_source,
        should_capture,
        "deopt runtime-state mismatch: code={:?} offset={} safepoint={} kind={:?} \
         field={} capture={} snapshot_has_source={}",
        frame.code(),
        frame.instruction_offset(),
        safepoint.id(),
        safepoint.kind(),
        label,
        should_capture,
        has_source
    );
}

#[cfg(debug_assertions)]
const fn is_completion_frame_value(source: DeoptValueSource) -> bool {
    matches!(
        source,
        DeoptValueSource::FrameValue(
            DeoptFrameValue::CompletionKind
                | DeoptFrameValue::CompletionValue
                | DeoptFrameValue::CompletionTarget
        )
    )
}

#[cfg(debug_assertions)]
fn panic_runtime_state_mismatch(
    frame: &FrameRecord,
    safepoint: SafepointDescriptor,
    detail: &str,
) -> ! {
    panic!(
        "deopt runtime-state mismatch: code={:?} offset={} safepoint={} kind={:?}: {}",
        frame.code(),
        frame.instruction_offset(),
        safepoint.id(),
        safepoint.kind(),
        detail
    );
}

fn trace_frame_record(frame: &FrameRecord, tracer: &mut PrimitiveTracer<'_>) {
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
