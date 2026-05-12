use crate::{
    CodeSlotsRef, EnvironmentSlotsRef, FunctionPayloadRef, ObjectSlotsRef, PrimitiveBigIntRecord,
    PrimitiveHeap, PrimitiveStringRecord, PrimitiveSymbolRecord, PrimitiveValueCellRecord,
    PrimitiveValueCellRef, RuntimeBoundFunctionRecord, RuntimeCodeRecord, RuntimeEnvironmentRecord,
    RuntimeFunctionRecord, RuntimeObjectRecord, RuntimeRealmRecord, RuntimeShapeRecord,
    RuntimeSuspendedExecutionRecord, SuspendedRegistersRef,
};
use lyng_js_types::{
    BigIntRef, CodeRef, EnvironmentRef, ObjectRef, PropertyKey, RealmRef, ShapeId, StringRef,
    SuspendedExecutionRef, SymbolRef, Value,
};
use std::cell::RefCell;
use std::marker::PhantomData;

/// Type-directed tracing entrypoint for heap values and typed handles.
pub trait TraceHeapEdges {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>);
}

/// Summary of marks performed during a trace walk.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PrimitiveTraceStats {
    pub values_traced: usize,
    pub strings_marked: usize,
    pub symbols_marked: usize,
    pub bigints_marked: usize,
    pub value_cells_marked: usize,
    pub objects_marked: usize,
    pub suspended_executions_marked: usize,
    pub environments_marked: usize,
    pub codes_marked: usize,
    pub realms_marked: usize,
    pub shapes_marked: usize,
}

/// Summary of a full mark/sweep cycle over the primitive domains.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PrimitiveCollectionStats {
    pub trace: PrimitiveTraceStats,
    pub major_mark_slices: usize,
    pub major_mark_slice_budget: usize,
    pub major_mark_work_items: usize,
    pub max_major_mark_slice_work_items: usize,
    pub total_major_mark_pause_ns: u128,
    pub max_major_mark_pause_ns: u128,
    pub major_mark_finish_work_items: usize,
    pub major_mark_finish_pause_ns: u128,
    pub major_mark_gray_work_items_after_finish: usize,
    pub background_sweep_started: bool,
    pub background_sweep_completed: bool,
    pub background_sweep_worker_thread_id: u64,
    pub background_sweep_candidates: usize,
    pub background_sweep_reclaimed: usize,
    pub background_sweep_duration_ns: u128,
    pub background_sweep_apply_pause_ns: u128,
    pub ephemeron_fixes: usize,
    pub weak_refs_cleared: usize,
    pub finalization_cells_queued: usize,
    pub pending_finalization_registries: usize,
    pub strings_reclaimed: usize,
    pub symbols_reclaimed: usize,
    pub bigints_reclaimed: usize,
    pub value_cells_reclaimed: usize,
    pub objects_reclaimed: usize,
    pub suspended_executions_reclaimed: usize,
    pub environments_reclaimed: usize,
    pub codes_reclaimed: usize,
    pub realms_reclaimed: usize,
    pub shapes_reclaimed: usize,
}

/// Explicit root registry for temporary runtime handles and values.
#[derive(Default)]
pub struct PrimitiveRoots {
    slots: RefCell<Vec<Option<RootRegistration>>>,
    free: RefCell<Vec<usize>>,
}

/// Grouped helper for building multiple typed roots from one lexical scope.
#[derive(Clone, Copy)]
pub struct PrimitiveRootScope<'a> {
    roots: &'a PrimitiveRoots,
}

/// Typed root guard whose lifetime is explicit in Rust control flow.
pub struct PrimitiveRootGuard<'a, T> {
    roots: &'a PrimitiveRoots,
    slot: usize,
    marker: PhantomData<T>,
}

/// Type-directed marker walk over `PrimitiveHeap`.
pub struct PrimitiveTracer<'a> {
    heap: &'a mut PrimitiveHeap,
    marker: &'a mut PrimitiveIncrementalMark,
}

/// Young-generation tracer for minor collections.
pub struct PrimitiveMinorTracer<'a> {
    heap: &'a mut PrimitiveHeap,
}

/// Worklist-backed state for a bounded major mark phase.
#[derive(Default)]
pub struct PrimitiveIncrementalMark {
    worklist: Vec<MarkWorkItem>,
    stats: PrimitiveTraceStats,
}

/// Progress state returned by one bounded major mark step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimitiveMarkProgress {
    Complete,
    HasMoreWork,
}

/// Summary of one bounded major mark step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrimitiveMarkStep {
    pub work_items_processed: usize,
    pub remaining_work_items: usize,
    pub max_work_items_per_slice: usize,
    pub progress: PrimitiveMarkProgress,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PrimitiveMajorMarkMetrics {
    budget: usize,
    slices: usize,
    work_items: usize,
    max_work_items: usize,
    total_pause_ns: u128,
    max_pause_ns: u128,
    finish_work_items: usize,
    finish_pause_ns: u128,
    gray_work_items_after_finish: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RootRegistration {
    Value(Value),
    String(StringRef),
    Symbol(SymbolRef),
    BigInt(BigIntRef),
    ValueCell(PrimitiveValueCellRef),
    Object(ObjectRef),
    Environment(EnvironmentRef),
    Code(CodeRef),
    Realm(RealmRef),
    Shape(ShapeId),
    SuspendedExecution(SuspendedExecutionRef),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MarkWorkItem {
    String(StringRef),
    Symbol(SymbolRef),
    BigInt(BigIntRef),
    ValueCell(PrimitiveValueCellRef),
    Object(ObjectRef),
    FunctionPayload(FunctionPayloadRef),
    SuspendedExecution(SuspendedExecutionRef),
    Environment(EnvironmentRef),
    Code(CodeRef),
    Realm(RealmRef),
    Shape(ShapeId),
    ObjectSlots(ObjectSlotsRef),
    SuspendedRegisters(SuspendedRegistersRef),
    EnvironmentSlots(EnvironmentSlotsRef),
    CodeSlots(CodeSlotsRef),
}

trait RootRegistrationCodec: Copy {
    fn into_registration(self) -> RootRegistration;
    fn from_registration(registration: RootRegistration) -> Option<Self>;
}

impl PrimitiveRoots {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub const fn scope(&self) -> PrimitiveRootScope<'_> {
        PrimitiveRootScope { roots: self }
    }

    #[inline]
    fn root<T: RootRegistrationCodec>(&self, value: T) -> PrimitiveRootGuard<'_, T> {
        let slot = self.register(value.into_registration());
        PrimitiveRootGuard {
            roots: self,
            slot,
            marker: PhantomData,
        }
    }

    #[inline]
    pub fn root_value(&self, value: Value) -> PrimitiveRootGuard<'_, Value> {
        self.root(value)
    }

    #[inline]
    pub fn root_string(&self, value: StringRef) -> PrimitiveRootGuard<'_, StringRef> {
        self.root(value)
    }

    #[inline]
    pub fn root_symbol(&self, value: SymbolRef) -> PrimitiveRootGuard<'_, SymbolRef> {
        self.root(value)
    }

    #[inline]
    pub fn root_bigint(&self, value: BigIntRef) -> PrimitiveRootGuard<'_, BigIntRef> {
        self.root(value)
    }

    #[inline]
    pub fn root_value_cell(
        &self,
        value: PrimitiveValueCellRef,
    ) -> PrimitiveRootGuard<'_, PrimitiveValueCellRef> {
        self.root(value)
    }

    #[inline]
    pub fn root_object(&self, value: ObjectRef) -> PrimitiveRootGuard<'_, ObjectRef> {
        self.root(value)
    }

    #[inline]
    pub fn root_environment(
        &self,
        value: EnvironmentRef,
    ) -> PrimitiveRootGuard<'_, EnvironmentRef> {
        self.root(value)
    }

    #[inline]
    pub fn root_code(&self, value: CodeRef) -> PrimitiveRootGuard<'_, CodeRef> {
        self.root(value)
    }

    #[inline]
    pub fn root_realm(&self, value: RealmRef) -> PrimitiveRootGuard<'_, RealmRef> {
        self.root(value)
    }

    #[inline]
    pub fn root_shape(&self, value: ShapeId) -> PrimitiveRootGuard<'_, ShapeId> {
        self.root(value)
    }

    #[inline]
    pub fn root_suspended_execution(
        &self,
        value: SuspendedExecutionRef,
    ) -> PrimitiveRootGuard<'_, SuspendedExecutionRef> {
        self.root(value)
    }

    #[inline]
    pub fn active_root_count(&self) -> usize {
        self.slots
            .borrow()
            .iter()
            .filter(|slot| slot.is_some())
            .count()
    }

    pub fn trace_roots(&self, tracer: &mut PrimitiveTracer<'_>) {
        for registration in self.slots.borrow().iter().copied().flatten() {
            match registration {
                RootRegistration::Value(value) => tracer.mark_value(value),
                RootRegistration::String(id) => tracer.mark_string(id),
                RootRegistration::Symbol(id) => tracer.mark_symbol(id),
                RootRegistration::BigInt(id) => tracer.mark_bigint(id),
                RootRegistration::ValueCell(id) => tracer.mark_value_cell(id),
                RootRegistration::Object(id) => tracer.mark_object(id),
                RootRegistration::Environment(id) => tracer.mark_environment(id),
                RootRegistration::Code(id) => tracer.mark_code(id),
                RootRegistration::Realm(id) => tracer.mark_realm(id),
                RootRegistration::Shape(id) => tracer.mark_shape(id),
                RootRegistration::SuspendedExecution(id) => tracer.mark_suspended_execution(id),
            }
        }
    }

    pub(crate) fn trace_minor_roots(&self, tracer: &mut PrimitiveMinorTracer<'_>) {
        for registration in self.slots.borrow().iter().copied().flatten() {
            match registration {
                RootRegistration::Value(value) => tracer.mark_value(value),
                RootRegistration::String(id) => tracer.mark_string(id),
                RootRegistration::Symbol(id) => tracer.mark_symbol(id),
                RootRegistration::BigInt(id) => tracer.mark_bigint(id),
                RootRegistration::ValueCell(id) => tracer.mark_value_cell(id),
                RootRegistration::Object(id) => tracer.mark_object(id),
                RootRegistration::Environment(id) => tracer.mark_environment(id),
                RootRegistration::SuspendedExecution(id) => tracer.mark_suspended_execution(id),
                RootRegistration::Code(_)
                | RootRegistration::Realm(_)
                | RootRegistration::Shape(_) => {}
            }
        }
    }

    fn register(&self, registration: RootRegistration) -> usize {
        let free_slot = self.free.borrow_mut().pop();
        free_slot.map_or_else(
            || {
                let mut slots = self.slots.borrow_mut();
                slots.push(Some(registration));
                slots.len() - 1
            },
            |slot| {
                self.slots.borrow_mut()[slot] = Some(registration);
                slot
            },
        )
    }

    fn unregister(&self, slot: usize) {
        let mut slots = self.slots.borrow_mut();
        if slots.get(slot).is_some() && slots[slot].take().is_some() {
            self.free.borrow_mut().push(slot);
        }
    }

    fn read(&self, slot: usize) -> RootRegistration {
        self.slots.borrow()[slot].expect("live root guard must reference a live root slot")
    }

    fn write(&self, slot: usize, registration: RootRegistration) {
        self.slots.borrow_mut()[slot] = Some(registration);
    }
}

impl<'a> PrimitiveRootScope<'a> {
    #[inline]
    fn root<T: RootRegistrationCodec>(self, value: T) -> PrimitiveRootGuard<'a, T> {
        self.roots.root(value)
    }

    #[inline]
    pub fn root_value(&self, value: Value) -> PrimitiveRootGuard<'a, Value> {
        self.root(value)
    }

    #[inline]
    pub fn root_string(&self, value: StringRef) -> PrimitiveRootGuard<'a, StringRef> {
        self.root(value)
    }

    #[inline]
    pub fn root_symbol(&self, value: SymbolRef) -> PrimitiveRootGuard<'a, SymbolRef> {
        self.root(value)
    }

    #[inline]
    pub fn root_bigint(&self, value: BigIntRef) -> PrimitiveRootGuard<'a, BigIntRef> {
        self.root(value)
    }

    #[inline]
    pub fn root_value_cell(
        &self,
        value: PrimitiveValueCellRef,
    ) -> PrimitiveRootGuard<'a, PrimitiveValueCellRef> {
        self.root(value)
    }

    #[inline]
    pub fn root_object(&self, value: ObjectRef) -> PrimitiveRootGuard<'a, ObjectRef> {
        self.root(value)
    }

    #[inline]
    pub fn root_environment(
        &self,
        value: EnvironmentRef,
    ) -> PrimitiveRootGuard<'a, EnvironmentRef> {
        self.root(value)
    }

    #[inline]
    pub fn root_code(&self, value: CodeRef) -> PrimitiveRootGuard<'a, CodeRef> {
        self.root(value)
    }

    #[inline]
    pub fn root_realm(&self, value: RealmRef) -> PrimitiveRootGuard<'a, RealmRef> {
        self.root(value)
    }

    #[inline]
    pub fn root_shape(&self, value: ShapeId) -> PrimitiveRootGuard<'a, ShapeId> {
        self.root(value)
    }

    #[inline]
    pub fn root_suspended_execution(
        &self,
        value: SuspendedExecutionRef,
    ) -> PrimitiveRootGuard<'a, SuspendedExecutionRef> {
        self.root(value)
    }

    #[inline]
    pub fn active_root_count(&self) -> usize {
        self.roots.active_root_count()
    }
}

impl<T> Drop for PrimitiveRootGuard<'_, T> {
    fn drop(&mut self) {
        self.roots.unregister(self.slot);
    }
}

macro_rules! impl_root_guard_accessors {
    ($ty:ty) => {
        impl<'a> PrimitiveRootGuard<'a, $ty> {
            #[inline]
            pub fn get(&self) -> $ty {
                <$ty as RootRegistrationCodec>::from_registration(self.roots.read(self.slot))
                    .expect("typed root guard must decode to its original root kind")
            }

            #[inline]
            pub fn set(&mut self, value: $ty) {
                self.roots.write(
                    self.slot,
                    <$ty as RootRegistrationCodec>::into_registration(value),
                );
            }
        }
    };
}

impl_root_guard_accessors!(Value);
impl_root_guard_accessors!(StringRef);
impl_root_guard_accessors!(SymbolRef);
impl_root_guard_accessors!(BigIntRef);
impl_root_guard_accessors!(ObjectRef);
impl_root_guard_accessors!(EnvironmentRef);
impl_root_guard_accessors!(CodeRef);
impl_root_guard_accessors!(RealmRef);
impl_root_guard_accessors!(ShapeId);
impl_root_guard_accessors!(SuspendedExecutionRef);

impl PrimitiveIncrementalMark {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub const fn trace_stats(&self) -> PrimitiveTraceStats {
        self.stats
    }

    #[inline]
    pub const fn has_work(&self) -> bool {
        !self.worklist.is_empty()
    }

    #[inline]
    pub const fn pending_work_items(&self) -> usize {
        self.worklist.len()
    }

    const fn total_live_marks(&self) -> usize {
        self.stats.strings_marked
            + self.stats.symbols_marked
            + self.stats.bigints_marked
            + self.stats.value_cells_marked
            + self.stats.objects_marked
            + self.stats.suspended_executions_marked
            + self.stats.environments_marked
            + self.stats.codes_marked
            + self.stats.realms_marked
            + self.stats.shapes_marked
    }
}

impl PrimitiveMarkStep {
    #[inline]
    pub const fn has_more_work(self) -> bool {
        matches!(self.progress, PrimitiveMarkProgress::HasMoreWork)
    }

    #[inline]
    pub const fn is_complete(self) -> bool {
        matches!(self.progress, PrimitiveMarkProgress::Complete)
    }
}

impl PrimitiveMajorMarkMetrics {
    #[inline]
    pub const fn new(budget: usize) -> Self {
        Self {
            budget,
            slices: 0,
            work_items: 0,
            max_work_items: 0,
            total_pause_ns: 0,
            max_pause_ns: 0,
            finish_work_items: 0,
            finish_pause_ns: 0,
            gray_work_items_after_finish: 0,
        }
    }

    fn record_slice(&mut self, work_items: usize, pause: std::time::Duration) {
        self.slices += 1;
        self.work_items += work_items;
        self.max_work_items = self.max_work_items.max(work_items);
        let pause_ns = pause.as_nanos();
        self.total_pause_ns = self.total_pause_ns.saturating_add(pause_ns);
        self.max_pause_ns = self.max_pause_ns.max(pause_ns);
    }

    fn record_finish(
        &mut self,
        work_items: usize,
        pause: std::time::Duration,
        remaining_work_items: usize,
    ) {
        self.finish_work_items = work_items;
        self.finish_pause_ns = pause.as_nanos().max(1);
        self.gray_work_items_after_finish = remaining_work_items;
    }
}

impl<'a> PrimitiveTracer<'a> {
    #[inline]
    pub const fn new(
        heap: &'a mut PrimitiveHeap,
        marker: &'a mut PrimitiveIncrementalMark,
    ) -> Self {
        Self { heap, marker }
    }

    #[inline]
    pub fn mark_value(&mut self, value: Value) {
        self.marker.stats.values_traced += 1;

        if let Some(id) = value.as_string_ref() {
            self.mark_string(id);
        } else if let Some(id) = value.as_symbol_ref() {
            self.mark_symbol(id);
        } else if let Some(id) = value.as_bigint_ref() {
            self.mark_bigint(id);
        } else if let Some(id) = value.as_object_ref() {
            self.mark_object(id);
        } else if let Some(id) = value.as_suspended_execution_ref() {
            self.mark_suspended_execution(id);
        }
    }

    #[inline]
    pub fn mark_string(&mut self, id: StringRef) {
        if self.heap.mark_string(id) {
            self.marker.stats.strings_marked += 1;
            self.marker.worklist.push(MarkWorkItem::String(id));
        }
    }

    #[inline]
    pub fn mark_symbol(&mut self, id: SymbolRef) {
        if self.heap.mark_symbol(id) {
            self.marker.stats.symbols_marked += 1;
            self.marker.worklist.push(MarkWorkItem::Symbol(id));
        }
    }

    #[inline]
    pub fn mark_bigint(&mut self, id: BigIntRef) {
        if self.heap.mark_bigint(id) {
            self.marker.stats.bigints_marked += 1;
            self.marker.worklist.push(MarkWorkItem::BigInt(id));
        }
    }

    #[inline]
    pub fn mark_value_cell(&mut self, id: PrimitiveValueCellRef) {
        if self.heap.mark_value_cell(id) {
            self.marker.stats.value_cells_marked += 1;
            self.marker.worklist.push(MarkWorkItem::ValueCell(id));
        }
    }

    #[inline]
    pub fn mark_object(&mut self, id: ObjectRef) {
        if self.heap.mark_object(id) {
            self.marker.stats.objects_marked += 1;
            self.marker.worklist.push(MarkWorkItem::Object(id));
        }
    }

    #[inline]
    pub fn mark_function_payload(&mut self, id: FunctionPayloadRef) {
        if self.heap.mark_function_payload(id) {
            self.marker.worklist.push(MarkWorkItem::FunctionPayload(id));
        }
    }

    #[inline]
    pub fn mark_suspended_execution(&mut self, id: SuspendedExecutionRef) {
        if self.heap.mark_suspended_execution(id) {
            self.marker.stats.suspended_executions_marked += 1;
            self.marker
                .worklist
                .push(MarkWorkItem::SuspendedExecution(id));
        }
    }

    #[inline]
    pub fn mark_environment(&mut self, id: EnvironmentRef) {
        if self.heap.mark_environment(id) {
            self.marker.stats.environments_marked += 1;
            self.marker.worklist.push(MarkWorkItem::Environment(id));
        }
    }

    #[inline]
    pub fn mark_code(&mut self, id: CodeRef) {
        if self.heap.mark_code(id) {
            self.marker.stats.codes_marked += 1;
            self.marker.worklist.push(MarkWorkItem::Code(id));
        }
    }

    #[inline]
    pub fn mark_realm(&mut self, id: RealmRef) {
        if self.heap.mark_realm(id) {
            self.marker.stats.realms_marked += 1;
            self.marker.worklist.push(MarkWorkItem::Realm(id));
        }
    }

    #[inline]
    pub fn mark_shape(&mut self, id: ShapeId) {
        if self.heap.mark_shape(id) {
            self.marker.stats.shapes_marked += 1;
            self.marker.worklist.push(MarkWorkItem::Shape(id));
        }
    }

    fn mark_object_slots(&mut self, id: ObjectSlotsRef) {
        if self.heap.mark_object_slots(id) {
            self.marker.worklist.push(MarkWorkItem::ObjectSlots(id));
        }
    }

    fn mark_suspended_registers(&mut self, id: SuspendedRegistersRef) {
        if self.heap.mark_suspended_registers(id) {
            self.marker
                .worklist
                .push(MarkWorkItem::SuspendedRegisters(id));
        }
    }

    fn mark_environment_slots(&mut self, id: EnvironmentSlotsRef) {
        if self.heap.mark_environment_slots(id) {
            self.marker
                .worklist
                .push(MarkWorkItem::EnvironmentSlots(id));
        }
    }

    fn mark_code_slots(&mut self, id: CodeSlotsRef) {
        if self.heap.mark_code_slots(id) {
            self.marker.worklist.push(MarkWorkItem::CodeSlots(id));
        }
    }

    fn trace_work_item(&mut self, item: MarkWorkItem) {
        match item {
            MarkWorkItem::String(id) => {
                if let Some(record) = self.heap.string(id) {
                    record.trace_heap_edges(self);
                }
            }
            MarkWorkItem::Symbol(id) => {
                if let Some(record) = self.heap.symbol(id) {
                    record.trace_heap_edges(self);
                }
            }
            MarkWorkItem::BigInt(id) => {
                if let Some(record) = self.heap.bigint(id) {
                    record.trace_heap_edges(self);
                }
            }
            MarkWorkItem::ValueCell(id) => {
                if let Some(record) = self.heap.value_cell(id) {
                    record.trace_heap_edges(self);
                }
            }
            MarkWorkItem::Object(id) => {
                if let Some(record) = self.heap.object(id) {
                    record.trace_heap_edges(self);
                }
            }
            MarkWorkItem::FunctionPayload(id) => {
                if let Some(record) = self.heap.function_payload(id) {
                    record.trace_heap_edges(self);
                }
            }
            MarkWorkItem::SuspendedExecution(id) => {
                if let Some(record) = self.heap.suspended_execution(id) {
                    record.trace_heap_edges(self);
                }
            }
            MarkWorkItem::Environment(id) => {
                if let Some(record) = self.heap.environment(id) {
                    record.trace_heap_edges(self);
                }
            }
            MarkWorkItem::Code(id) => {
                if let Some(record) = self.heap.code(id) {
                    record.trace_heap_edges(self);
                }
            }
            MarkWorkItem::Realm(id) => {
                if let Some(record) = self.heap.realm(id) {
                    record.trace_heap_edges(self);
                }
            }
            MarkWorkItem::Shape(id) => {
                if let Some(record) = self.heap.shape(id) {
                    record.trace_heap_edges(self);
                }
            }
            MarkWorkItem::ObjectSlots(id) => {
                let Some(values) = self.heap.object_slots(id).map(<[Value]>::to_vec) else {
                    return;
                };
                for value in values {
                    self.mark_value(value);
                }
            }
            MarkWorkItem::SuspendedRegisters(id) => {
                let Some(values) = self.heap.suspended_registers(id).map(<[Value]>::to_vec) else {
                    return;
                };
                for value in values {
                    self.mark_value(value);
                }
            }
            MarkWorkItem::EnvironmentSlots(id) => {
                let Some(values) = self.heap.environment_slots(id).map(<[Value]>::to_vec) else {
                    return;
                };
                for value in values {
                    self.mark_value(value);
                }
            }
            MarkWorkItem::CodeSlots(id) => {
                let Some(values) = self.heap.code_slots(id).map(<[Value]>::to_vec) else {
                    return;
                };
                for value in values {
                    self.mark_value(value);
                }
            }
        }
    }

    fn mark_step(&mut self, budget: usize) -> PrimitiveMarkStep {
        let mut work_items_processed = 0;
        while work_items_processed < budget {
            let Some(item) = self.marker.worklist.pop() else {
                break;
            };
            work_items_processed += 1;
            self.trace_work_item(item);
        }

        let progress = if self.marker.has_work() {
            PrimitiveMarkProgress::HasMoreWork
        } else {
            PrimitiveMarkProgress::Complete
        };
        PrimitiveMarkStep {
            work_items_processed,
            remaining_work_items: self.marker.pending_work_items(),
            max_work_items_per_slice: budget,
            progress,
        }
    }
}

impl<'a> PrimitiveMinorTracer<'a> {
    #[inline]
    pub(crate) const fn new(heap: &'a mut PrimitiveHeap) -> Self {
        Self { heap }
    }

    #[inline]
    pub(crate) fn mark_value(&mut self, value: Value) {
        if let Some(id) = value.as_string_ref() {
            self.mark_string(id);
        } else if let Some(id) = value.as_symbol_ref() {
            self.mark_symbol(id);
        } else if let Some(id) = value.as_bigint_ref() {
            self.mark_bigint(id);
        } else if let Some(id) = value.as_object_ref() {
            self.mark_object(id);
        } else if let Some(id) = value.as_suspended_execution_ref() {
            self.mark_suspended_execution(id);
        }
    }

    pub(crate) fn mark_string(&mut self, id: StringRef) {
        if !self.heap.is_young_string(id) || !self.heap.mark_string(id) {
            return;
        }
        if let Some(record) = self.heap.string(id)
            && let Some((left, right)) = record.cons_children()
        {
            self.mark_string(left);
            self.mark_string(right);
        }
    }

    pub(crate) fn mark_symbol(&mut self, id: SymbolRef) {
        if !self.heap.is_young_symbol(id) || !self.heap.mark_symbol(id) {
            return;
        }
        if let Some(record) = self.heap.symbol(id)
            && let Some(description) = record.description()
        {
            self.mark_string(description);
        }
    }

    pub(crate) fn mark_bigint(&mut self, id: BigIntRef) {
        if self.heap.is_young_bigint(id) {
            self.heap.mark_bigint(id);
        }
    }

    pub(crate) fn mark_value_cell(&mut self, id: PrimitiveValueCellRef) {
        if !self.heap.is_young_value_cell(id) || !self.heap.mark_value_cell(id) {
            return;
        }
        if let Some(record) = self.heap.value_cell(id) {
            self.mark_value(record.stored_value());
            if let Some(string) = record.linked_string() {
                self.mark_string(string);
            }
        }
    }

    pub(crate) fn mark_object(&mut self, id: ObjectRef) {
        if !self.heap.is_young_object(id) || !self.heap.mark_object(id) {
            return;
        }
        self.trace_object_record(id);
    }

    pub(crate) fn mark_function_payload(&mut self, id: FunctionPayloadRef) {
        if !self.heap.is_young_function_payload(id) || !self.heap.mark_function_payload(id) {
            return;
        }
        if let Some(record) = self.heap.function_payload(id) {
            self.trace_function_payload_edges(record);
        }
    }

    pub(crate) fn mark_environment(&mut self, id: EnvironmentRef) {
        if !self.heap.is_young_environment(id) || !self.heap.mark_environment(id) {
            return;
        }
        self.trace_environment_record(id);
    }

    pub(crate) fn mark_suspended_execution(&mut self, id: SuspendedExecutionRef) {
        if !self.heap.is_young_suspended_execution(id) || !self.heap.mark_suspended_execution(id) {
            return;
        }
        self.trace_suspended_execution_record(id);
    }

    pub(crate) fn mark_object_slots(&mut self, id: ObjectSlotsRef) {
        if self.heap.is_young_object_slots(id) {
            self.heap.mark_object_slots(id);
        }
        let Some(values) = self.heap.object_slots(id).map(<[Value]>::to_vec) else {
            return;
        };
        for value in values {
            self.mark_value(value);
        }
    }

    pub(crate) fn mark_environment_slots(&mut self, id: EnvironmentSlotsRef) {
        if self.heap.is_young_environment_slots(id) {
            self.heap.mark_environment_slots(id);
        }
        let Some(values) = self.heap.environment_slots(id).map(<[Value]>::to_vec) else {
            return;
        };
        for value in values {
            self.mark_value(value);
        }
    }

    pub(crate) fn mark_suspended_registers(&mut self, id: SuspendedRegistersRef) {
        if self.heap.is_young_suspended_registers(id) {
            self.heap.mark_suspended_registers(id);
        }
        let Some(values) = self.heap.suspended_registers(id).map(<[Value]>::to_vec) else {
            return;
        };
        for value in values {
            self.mark_value(value);
        }
    }

    pub(crate) fn trace_string_card(&mut self, card_index: usize) {
        let mut records = Vec::new();
        self.heap
            .scan_string_card(card_index, |record| records.push(record));
        for record in records {
            self.trace_string_edges(record);
        }
    }

    pub(crate) fn trace_symbol_card(&mut self, card_index: usize) {
        let mut records = Vec::new();
        self.heap
            .scan_symbol_card(card_index, |record| records.push(record));
        for record in records {
            self.trace_symbol_edges(record);
        }
    }

    pub(crate) fn trace_object_slots_card(&mut self, card_index: usize) {
        let mut values = Vec::new();
        self.heap.scan_object_slots_card(card_index, |slot_values| {
            values.extend_from_slice(slot_values);
        });
        for value in values {
            self.mark_value(value);
        }
    }

    pub(crate) fn trace_environment_slots_card(&mut self, card_index: usize) {
        let mut values = Vec::new();
        self.heap
            .scan_environment_slots_card(card_index, |slot_values| {
                values.extend_from_slice(slot_values);
            });
        for value in values {
            self.mark_value(value);
        }
    }

    pub(crate) fn trace_code_slots_card(&mut self, card_index: usize) {
        let mut values = Vec::new();
        self.heap.scan_code_slots_card(card_index, |slot_values| {
            values.extend_from_slice(slot_values);
        });
        for value in values {
            self.mark_value(value);
        }
    }

    pub(crate) fn trace_object_card(&mut self, card_index: usize) {
        let mut records = Vec::new();
        self.heap
            .scan_object_card(card_index, |record| records.push(record));
        for record in records {
            self.trace_object_edges(record);
        }
    }

    pub(crate) fn trace_environment_card(&mut self, card_index: usize) {
        let mut records = Vec::new();
        self.heap
            .scan_environment_card(card_index, |record| records.push(record));
        for record in records {
            self.trace_environment_edges(record);
        }
    }

    pub(crate) fn trace_function_payload_card(&mut self, card_index: usize) {
        let mut records = Vec::new();
        self.heap
            .scan_function_payload_card(card_index, |record| records.push(record));
        for record in records {
            self.trace_function_payload_edges(record);
        }
    }

    pub(crate) fn trace_value_cell_card(&mut self, card_index: usize) {
        let mut records = Vec::new();
        self.heap
            .scan_value_cell_card(card_index, |record| records.push(record));
        for record in records {
            self.mark_value(record.stored_value());
            if let Some(string) = record.linked_string() {
                self.mark_string(string);
            }
        }
    }

    pub(crate) fn trace_suspended_execution_card(&mut self, card_index: usize) {
        let mut records = Vec::new();
        self.heap
            .scan_suspended_execution_card(card_index, |record| records.push(record));
        for record in records {
            self.trace_suspended_execution_edges(record);
        }
    }

    pub(crate) fn trace_suspended_registers_card(&mut self, card_index: usize) {
        let mut values = Vec::new();
        self.heap
            .scan_suspended_registers_card(card_index, |slot_values| {
                values.extend_from_slice(slot_values);
            });
        for value in values {
            self.mark_value(value);
        }
    }

    pub(crate) fn trace_realm_card(&mut self, card_index: usize) {
        let mut records = Vec::new();
        self.heap
            .scan_realm_card(card_index, |record| records.push(record));
        for record in records {
            self.trace_realm_edges(record);
        }
    }

    pub(crate) fn trace_shape_card(&mut self, card_index: usize) {
        let mut records = Vec::new();
        self.heap
            .scan_shape_card(card_index, |record| records.push(record));
        for record in records {
            self.trace_shape_edges(record);
        }
    }

    fn trace_object_record(&mut self, id: ObjectRef) {
        if let Some(record) = self.heap.object(id) {
            self.trace_object_edges(record);
        }
    }

    fn trace_string_edges(&mut self, record: PrimitiveStringRecord) {
        if let Some((left, right)) = record.cons_children() {
            self.mark_string(left);
            self.mark_string(right);
        }
    }

    fn trace_symbol_edges(&mut self, record: PrimitiveSymbolRecord) {
        if let Some(description) = record.description() {
            self.mark_string(description);
        }
    }

    fn trace_object_edges(&mut self, record: RuntimeObjectRecord) {
        if let Some(prototype) = record.prototype() {
            self.mark_object(prototype);
        }
        if let Some(slots) = record.named_slots() {
            self.mark_object_slots(slots);
        }
        if let Some(elements) = record.elements() {
            self.mark_object_slots(elements);
        }
        if let Some(private_slots) = record.private_slots() {
            self.mark_object_slots(private_slots);
        }
        if let Some(payload) = record.function_payload() {
            self.mark_function_payload(payload);
        }
        if let Some(payload) = record.ordinary_payload() {
            self.mark_value_cell(payload);
        }
    }

    fn trace_function_payload_edges(&mut self, record: RuntimeFunctionRecord) {
        if let Some(environment) = record.environment() {
            self.mark_environment(environment);
        }
        if let Some(private_env) = record.private_env() {
            self.mark_environment(private_env);
        }
        if let Some(home_object) = record.home_object() {
            self.mark_object(home_object);
        }
        if let Some(bound) = record.bound() {
            self.mark_object(bound.target());
            self.mark_value(bound.this_value());
            if let Some(arguments) = bound.arguments() {
                self.mark_object_slots(arguments);
            }
        }
    }

    fn trace_environment_record(&mut self, id: EnvironmentRef) {
        if let Some(record) = self.heap.environment(id) {
            self.trace_environment_edges(record);
        }
    }

    fn trace_environment_edges(&mut self, record: RuntimeEnvironmentRecord) {
        if let Some(outer) = record.outer() {
            self.mark_environment(outer);
        }
        if let Some(slots) = record.slots() {
            self.mark_environment_slots(slots);
        }
        if let Some(function_object) = record.function_object() {
            self.mark_object(function_object);
        }
        self.mark_value(record.this_value());
        if let Some(new_target) = record.new_target() {
            self.mark_object(new_target);
        }
        if let Some(home_object) = record.home_object() {
            self.mark_object(home_object);
        }
    }

    fn trace_suspended_execution_record(&mut self, id: SuspendedExecutionRef) {
        if let Some(record) = self.heap.suspended_execution(id) {
            self.trace_suspended_execution_edges(record);
        }
    }

    fn trace_suspended_execution_edges(&mut self, record: RuntimeSuspendedExecutionRecord) {
        self.mark_environment(record.lexical_env());
        self.mark_environment(record.variable_env());
        if let Some(private_env) = record.private_env() {
            self.mark_environment(private_env);
        }
        self.mark_value(record.this_value());
        if let Some(construct_this) = record.construct_this() {
            self.mark_object(construct_this);
        }
        if let Some(new_target) = record.new_target() {
            self.mark_object(new_target);
        }
        if let Some(callee) = record.callee() {
            self.mark_object(callee);
        }
        if let Some(registers) = record.registers() {
            self.mark_suspended_registers(registers);
        }
    }

    fn trace_realm_edges(&mut self, record: RuntimeRealmRecord) {
        if let Some(global_object) = record.global_object() {
            self.mark_object(global_object);
        }
        if let Some(global_env) = record.global_env() {
            self.mark_environment(global_env);
        }
    }

    fn trace_shape_edges(&mut self, record: RuntimeShapeRecord) {
        if let Some(prototype_guard) = record.prototype_guard() {
            self.mark_object(prototype_guard);
        }
    }
}

impl PrimitiveHeap {
    #[inline]
    pub const fn incremental_mark_in_progress(&self) -> bool {
        self.active_major_mark.is_some()
    }

    #[inline]
    pub fn active_incremental_mark_pending_work_items(&self) -> Option<usize> {
        self.active_major_mark
            .as_ref()
            .map(PrimitiveIncrementalMark::pending_work_items)
    }

    pub fn begin_incremental_mark(&mut self, roots: &PrimitiveRoots) -> bool {
        self.begin_incremental_mark_tracing(roots, &())
    }

    pub fn begin_incremental_mark_tracing<T: TraceHeapEdges + ?Sized>(
        &mut self,
        roots: &PrimitiveRoots,
        additional_roots: &T,
    ) -> bool {
        if self.active_major_mark.is_some() {
            return false;
        }

        let marker = self.start_incremental_mark_tracing(roots, additional_roots);
        self.active_major_mark = Some(marker);
        true
    }

    pub fn poll_incremental_mark_step(&mut self) -> Option<PrimitiveMarkStep> {
        let mut marker = self.active_major_mark.take()?;
        let step = self.mark_step(&mut marker, self.major_mark_slice_budget());
        self.active_major_mark = Some(marker);
        Some(step)
    }

    pub fn finish_active_incremental_mark(&mut self) -> Option<PrimitiveCollectionStats> {
        let marker = self.active_major_mark.take()?;
        Some(self.finish_incremental_mark(marker))
    }

    pub fn start_incremental_mark(&mut self, roots: &PrimitiveRoots) -> PrimitiveIncrementalMark {
        self.start_incremental_mark_tracing(roots, &())
    }

    pub fn start_incremental_mark_tracing<T: TraceHeapEdges + ?Sized>(
        &mut self,
        roots: &PrimitiveRoots,
        additional_roots: &T,
    ) -> PrimitiveIncrementalMark {
        self.active_major_mark = None;
        self.clear_all_marks();
        let mut marker = PrimitiveIncrementalMark::new();
        {
            let mut tracer = PrimitiveTracer::new(self, &mut marker);
            roots.trace_roots(&mut tracer);
            additional_roots.trace_heap_edges(&mut tracer);
        }
        marker
    }

    pub fn mark_step(
        &mut self,
        marker: &mut PrimitiveIncrementalMark,
        budget: usize,
    ) -> PrimitiveMarkStep {
        let mut tracer = PrimitiveTracer::new(self, marker);
        tracer.mark_step(budget)
    }

    pub(crate) fn shade_active_incremental_mark<T: TraceHeapEdges>(&mut self, value: &T) {
        let Some(mut marker) = self.active_major_mark.take() else {
            return;
        };
        {
            let mut tracer = PrimitiveTracer::new(self, &mut marker);
            value.trace_heap_edges(&mut tracer);
        }
        self.active_major_mark = Some(marker);
    }

    pub fn finish_incremental_mark(
        &mut self,
        mut marker: PrimitiveIncrementalMark,
    ) -> PrimitiveCollectionStats {
        self.finish_incremental_mark_with_budget(&mut marker, usize::MAX, None)
    }

    pub fn collect(&mut self, roots: &PrimitiveRoots) -> PrimitiveCollectionStats {
        self.collect_tracing(roots, &())
    }

    pub fn collect_tracing<T: TraceHeapEdges + ?Sized>(
        &mut self,
        roots: &PrimitiveRoots,
        additional_roots: &T,
    ) -> PrimitiveCollectionStats {
        let mut marker = self.start_incremental_mark_tracing(roots, additional_roots);
        self.finish_incremental_mark_with_budget(&mut marker, self.major_mark_slice_budget(), None)
    }

    pub(crate) fn collect_tracing_with_mark_metrics<T: TraceHeapEdges + ?Sized>(
        &mut self,
        roots: &PrimitiveRoots,
        additional_roots: &T,
        metrics: &mut PrimitiveMajorMarkMetrics,
    ) -> PrimitiveCollectionStats {
        let mut marker = self.start_incremental_mark_tracing(roots, additional_roots);
        self.finish_incremental_mark_with_budget(
            &mut marker,
            self.major_mark_slice_budget(),
            Some(metrics),
        )
    }

    fn finish_incremental_mark_with_budget(
        &mut self,
        marker: &mut PrimitiveIncrementalMark,
        budget: usize,
        mut metrics: Option<&mut PrimitiveMajorMarkMetrics>,
    ) -> PrimitiveCollectionStats {
        let budget = budget.max(1);
        while marker.pending_work_items() > budget {
            let start = std::time::Instant::now();
            let step = self.mark_step(marker, budget);
            if let Some(metrics) = metrics.as_deref_mut() {
                metrics.record_slice(step.work_items_processed, start.elapsed());
            }
            if step.is_complete() {
                break;
            }
        }

        let finish_start = std::time::Instant::now();
        let mut finish_work_items = self.drain_mark_work_unbounded(marker);
        let ephemeron_fixes =
            self.trace_weak_structures_to_fixpoint(marker, &mut finish_work_items);
        let gray_work_items_after_finish = marker.pending_work_items();
        if let Some(metrics) = metrics.as_deref_mut() {
            metrics.record_finish(
                finish_work_items,
                finish_start.elapsed(),
                gray_work_items_after_finish,
            );
        }
        assert_eq!(
            gray_work_items_after_finish, 0,
            "atomic major mark finish must drain all gray work before sweep"
        );

        let trace = marker.trace_stats();
        let (weak_refs_cleared, finalization_cells_queued, pending_finalization_registries) =
            self.sweep_weak_state();
        let sweep_candidates = self.collect_major_sweep_candidates();
        let (reclaimed, background_sweep) =
            self.run_background_sweep_to_completion(sweep_candidates);

        let major = metrics.as_deref().copied().unwrap_or_default();
        PrimitiveCollectionStats {
            trace,
            major_mark_slices: major.slices,
            major_mark_slice_budget: major.budget,
            major_mark_work_items: major.work_items,
            max_major_mark_slice_work_items: major.max_work_items,
            total_major_mark_pause_ns: major.total_pause_ns,
            max_major_mark_pause_ns: major.max_pause_ns,
            major_mark_finish_work_items: major.finish_work_items,
            major_mark_finish_pause_ns: major.finish_pause_ns,
            major_mark_gray_work_items_after_finish: major.gray_work_items_after_finish,
            background_sweep_started: background_sweep.started,
            background_sweep_completed: background_sweep.completed,
            background_sweep_worker_thread_id: background_sweep.worker_thread_id,
            background_sweep_candidates: background_sweep.candidates,
            background_sweep_reclaimed: background_sweep.reclaimed,
            background_sweep_duration_ns: background_sweep.duration_ns,
            background_sweep_apply_pause_ns: background_sweep.apply_pause_ns,
            ephemeron_fixes,
            weak_refs_cleared,
            finalization_cells_queued,
            pending_finalization_registries,
            strings_reclaimed: reclaimed.strings,
            symbols_reclaimed: reclaimed.symbols,
            bigints_reclaimed: reclaimed.bigints,
            value_cells_reclaimed: reclaimed.value_cells,
            objects_reclaimed: reclaimed.objects,
            suspended_executions_reclaimed: reclaimed.suspended_executions,
            environments_reclaimed: reclaimed.environments,
            codes_reclaimed: reclaimed.codes,
            realms_reclaimed: reclaimed.realms,
            shapes_reclaimed: reclaimed.shapes,
        }
    }

    fn drain_mark_work_unbounded(&mut self, marker: &mut PrimitiveIncrementalMark) -> usize {
        let mut work_items = 0;
        while marker.has_work() {
            work_items += self.mark_step(marker, usize::MAX).work_items_processed;
        }
        work_items
    }

    fn trace_weak_structures_to_fixpoint(
        &mut self,
        marker: &mut PrimitiveIncrementalMark,
        finish_work_items: &mut usize,
    ) -> usize {
        let mut ephemeron_fixes = 0;

        loop {
            let before = marker.total_live_marks();
            let weak_maps = self.weak_map_snapshots();
            {
                let mut tracer = PrimitiveTracer::new(self, marker);
                for (owner, entries) in weak_maps {
                    if !tracer.heap.is_object_marked(owner) {
                        continue;
                    }
                    for (key, value) in entries {
                        if tracer.heap.is_weak_ref_marked(key) {
                            tracer.mark_value(value);
                        }
                    }
                }

                let finalization_registries = tracer.heap.finalization_registry_snapshots();
                for (owner, live_cells, pending_holdings) in finalization_registries {
                    if !tracer.heap.is_object_marked(owner) {
                        continue;
                    }
                    for holdings in live_cells.into_iter().chain(pending_holdings) {
                        tracer.mark_value(holdings);
                    }
                }
            }

            *finish_work_items += self.drain_mark_work_unbounded(marker);

            let after = marker.total_live_marks();
            if after == before {
                return ephemeron_fixes;
            }
            ephemeron_fixes += after - before;
        }
    }
}

impl TraceHeapEdges for () {
    fn trace_heap_edges(&self, _tracer: &mut PrimitiveTracer<'_>) {}
}

impl TraceHeapEdges for Value {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        tracer.mark_value(*self);
    }
}

impl TraceHeapEdges for StringRef {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        tracer.mark_string(*self);
    }
}

impl TraceHeapEdges for SymbolRef {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        tracer.mark_symbol(*self);
    }
}

impl TraceHeapEdges for BigIntRef {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        tracer.mark_bigint(*self);
    }
}

impl TraceHeapEdges for PrimitiveValueCellRef {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        tracer.mark_value_cell(*self);
    }
}

impl TraceHeapEdges for ObjectRef {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        tracer.mark_object(*self);
    }
}

impl TraceHeapEdges for FunctionPayloadRef {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        tracer.mark_function_payload(*self);
    }
}

impl TraceHeapEdges for EnvironmentRef {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        tracer.mark_environment(*self);
    }
}

impl TraceHeapEdges for CodeRef {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        tracer.mark_code(*self);
    }
}

impl TraceHeapEdges for RealmRef {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        tracer.mark_realm(*self);
    }
}

impl TraceHeapEdges for ShapeId {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        tracer.mark_shape(*self);
    }
}

impl TraceHeapEdges for crate::WeakHeapRef {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        match self {
            Self::Object(object) => object.trace_heap_edges(tracer),
            Self::Symbol(symbol) => symbol.trace_heap_edges(tracer),
        }
    }
}

impl TraceHeapEdges for SuspendedExecutionRef {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        tracer.mark_suspended_execution(*self);
    }
}

impl TraceHeapEdges for ObjectSlotsRef {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        tracer.mark_object_slots(*self);
    }
}

impl TraceHeapEdges for EnvironmentSlotsRef {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        tracer.mark_environment_slots(*self);
    }
}

impl TraceHeapEdges for CodeSlotsRef {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        tracer.mark_code_slots(*self);
    }
}

impl TraceHeapEdges for SuspendedRegistersRef {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        tracer.mark_suspended_registers(*self);
    }
}

impl<T: TraceHeapEdges> TraceHeapEdges for Option<T> {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        if let Some(value) = self {
            value.trace_heap_edges(tracer);
        }
    }
}

impl TraceHeapEdges for PrimitiveStringRecord {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        if let Some((left, right)) = self.cons_children() {
            tracer.mark_string(left);
            tracer.mark_string(right);
        }
    }
}

impl TraceHeapEdges for PrimitiveSymbolRecord {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.description().trace_heap_edges(tracer);
    }
}

impl TraceHeapEdges for PrimitiveBigIntRecord {
    fn trace_heap_edges(&self, _tracer: &mut PrimitiveTracer<'_>) {}
}

impl TraceHeapEdges for PrimitiveValueCellRecord {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.stored_value().trace_heap_edges(tracer);
        self.linked_string().trace_heap_edges(tracer);
    }
}

impl TraceHeapEdges for RuntimeObjectRecord {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.prototype().trace_heap_edges(tracer);
        self.shape().trace_heap_edges(tracer);
        self.named_slots().trace_heap_edges(tracer);
        self.elements().trace_heap_edges(tracer);
        self.private_slots().trace_heap_edges(tracer);
        self.function_payload().trace_heap_edges(tracer);
        self.ordinary_payload().trace_heap_edges(tracer);
    }
}

impl TraceHeapEdges for RuntimeFunctionRecord {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.realm().trace_heap_edges(tracer);
        self.environment().trace_heap_edges(tracer);
        self.private_env().trace_heap_edges(tracer);
        self.home_object().trace_heap_edges(tracer);
        self.bytecode().trace_heap_edges(tracer);
        self.bound().trace_heap_edges(tracer);
    }
}

impl TraceHeapEdges for RuntimeSuspendedExecutionRecord {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.realm().trace_heap_edges(tracer);
        self.code().trace_heap_edges(tracer);
        self.lexical_env().trace_heap_edges(tracer);
        self.variable_env().trace_heap_edges(tracer);
        self.private_env().trace_heap_edges(tracer);
        self.this_value().trace_heap_edges(tracer);
        self.construct_this().trace_heap_edges(tracer);
        self.new_target().trace_heap_edges(tracer);
        self.callee().trace_heap_edges(tracer);
        self.registers().trace_heap_edges(tracer);
    }
}

impl TraceHeapEdges for RuntimeBoundFunctionRecord {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.target().trace_heap_edges(tracer);
        self.this_value().trace_heap_edges(tracer);
        self.arguments().trace_heap_edges(tracer);
    }
}

impl TraceHeapEdges for RuntimeEnvironmentRecord {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.outer().trace_heap_edges(tracer);
        self.slots().trace_heap_edges(tracer);
        self.function_object().trace_heap_edges(tracer);
        self.this_value().trace_heap_edges(tracer);
        self.new_target().trace_heap_edges(tracer);
        self.home_object().trace_heap_edges(tracer);
    }
}

impl TraceHeapEdges for RuntimeCodeRecord {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.parent().trace_heap_edges(tracer);
        self.realm().trace_heap_edges(tracer);
        self.constants().trace_heap_edges(tracer);
    }
}

impl TraceHeapEdges for RuntimeRealmRecord {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.global_object().trace_heap_edges(tracer);
        self.global_env().trace_heap_edges(tracer);
        self.bootstrap_code().trace_heap_edges(tracer);
        self.root_shape().trace_heap_edges(tracer);
    }
}

impl TraceHeapEdges for RuntimeShapeRecord {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        self.parent().trace_heap_edges(tracer);
        self.prototype_guard().trace_heap_edges(tracer);
    }
}

impl TraceHeapEdges for PropertyKey {
    fn trace_heap_edges(&self, tracer: &mut PrimitiveTracer<'_>) {
        if let Some(symbol) = self.as_symbol() {
            symbol.trace_heap_edges(tracer);
        }
    }
}

impl RootRegistrationCodec for Value {
    fn into_registration(self) -> RootRegistration {
        RootRegistration::Value(self)
    }

    fn from_registration(registration: RootRegistration) -> Option<Self> {
        match registration {
            RootRegistration::Value(value) => Some(value),
            _ => None,
        }
    }
}

macro_rules! impl_root_registration_codec {
    ($ty:ty, $variant:ident) => {
        impl RootRegistrationCodec for $ty {
            fn into_registration(self) -> RootRegistration {
                RootRegistration::$variant(self)
            }

            fn from_registration(registration: RootRegistration) -> Option<Self> {
                match registration {
                    RootRegistration::$variant(value) => Some(value),
                    _ => None,
                }
            }
        }
    };
}

impl_root_registration_codec!(StringRef, String);
impl_root_registration_codec!(SymbolRef, Symbol);
impl_root_registration_codec!(BigIntRef, BigInt);
impl_root_registration_codec!(PrimitiveValueCellRef, ValueCell);
impl_root_registration_codec!(ObjectRef, Object);
impl_root_registration_codec!(EnvironmentRef, Environment);
impl_root_registration_codec!(CodeRef, Code);
impl_root_registration_codec!(RealmRef, Realm);
impl_root_registration_codec!(ShapeId, Shape);
impl_root_registration_codec!(SuspendedExecutionRef, SuspendedExecution);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AllocationLifetime, BigIntSign, StringEncoding, SymbolFlags};

    #[test]
    fn typed_root_guards_follow_explicit_lifetimes() {
        let roots = PrimitiveRoots::new();
        let mut outer = roots.scope().root_string(StringRef::from_raw(7).unwrap());

        assert_eq!(roots.active_root_count(), 1);
        assert_eq!(outer.get(), StringRef::from_raw(7).unwrap());

        outer.set(StringRef::from_raw(9).unwrap());
        assert_eq!(outer.get(), StringRef::from_raw(9).unwrap());

        {
            let inner_scope = roots.scope();
            let _inner = inner_scope.root_bigint(BigIntRef::from_raw(11).unwrap());
            assert_eq!(inner_scope.active_root_count(), 2);
        }

        assert_eq!(roots.active_root_count(), 1);
        drop(outer);
        assert_eq!(roots.active_root_count(), 0);
    }

    #[test]
    fn rooted_handles_survive_collection_and_dead_slots_are_reclaimed() {
        let mut heap = PrimitiveHeap::new();
        let roots = PrimitiveRoots::new();
        let live = heap.alloc_string(
            StringEncoding::Latin1,
            4,
            b"live",
            None,
            AllocationLifetime::Default,
        );
        let dead = heap.alloc_string(
            StringEncoding::Latin1,
            4,
            b"dead",
            None,
            AllocationLifetime::Default,
        );

        let scope = roots.scope();
        let rooted = scope.root_string(live);
        let stats = heap.collect(&roots);

        assert_eq!(stats.trace.strings_marked, 1);
        assert_eq!(stats.strings_reclaimed, 1);
        assert_eq!(rooted.get(), live);
        assert_eq!(heap.string_payload(live), Some(&b"live"[..]));
        assert_eq!(heap.string(dead), None);

        let replacement = heap.alloc_string(
            StringEncoding::Latin1,
            4,
            b"next",
            None,
            AllocationLifetime::Default,
        );

        assert_eq!(replacement, dead);

        drop(rooted);
        let stats = heap.collect(&roots);
        assert_eq!(stats.strings_reclaimed, 2);
        assert_eq!(heap.string(live), None);
    }

    #[test]
    fn nested_root_scopes_keep_cross_domain_edges_alive_until_drop() {
        let mut heap = PrimitiveHeap::new();
        let roots = PrimitiveRoots::new();
        let description = heap.alloc_string(
            StringEncoding::Latin1,
            4,
            b"desc",
            None,
            AllocationLifetime::Default,
        );
        let symbol = heap.alloc_symbol(
            Some(description),
            SymbolFlags::ordinary(),
            AllocationLifetime::Default,
        );
        let bigint = heap.alloc_bigint(BigIntSign::Negative, &[3, 1], AllocationLifetime::Default);
        let dead_string = heap.alloc_string(
            StringEncoding::Latin1,
            4,
            b"dead",
            None,
            AllocationLifetime::Default,
        );

        {
            let outer = roots.scope();
            let _symbol_value = outer.root_value(Value::from_symbol_ref(symbol));

            {
                let inner = roots.scope();
                let _bigint = inner.root_bigint(bigint);
                let stats = heap.collect(&roots);

                assert_eq!(stats.trace.values_traced, 1);
                assert_eq!(stats.trace.symbols_marked, 1);
                assert_eq!(stats.trace.strings_marked, 1);
                assert_eq!(stats.trace.bigints_marked, 1);
                assert_eq!(stats.strings_reclaimed, 1);
                assert_eq!(
                    heap.symbol(symbol).unwrap().description(),
                    Some(description)
                );
                assert_eq!(heap.bigint(bigint).unwrap().limb_count(), 2);
                assert_eq!(heap.string(dead_string), None);
            }

            let stats = heap.collect(&roots);
            assert_eq!(stats.bigints_reclaimed, 1);
            assert_eq!(heap.bigint(bigint), None);
            assert_eq!(
                heap.symbol(symbol).unwrap().description(),
                Some(description)
            );
            assert_eq!(
                heap.string(description),
                Some(heap.string(description).unwrap())
            );
        }

        let stats = heap.collect(&roots);
        assert_eq!(stats.symbols_reclaimed, 1);
        assert_eq!(stats.strings_reclaimed, 1);
        assert_eq!(heap.symbol(symbol), None);
        assert_eq!(heap.string(description), None);
    }

    #[test]
    fn typed_tracing_marks_supported_handle_domains_without_stack_scanning() {
        let mut heap = PrimitiveHeap::new();
        let string = heap.alloc_string(
            StringEncoding::Latin1,
            3,
            b"key",
            None,
            AllocationLifetime::Default,
        );
        let symbol = heap.alloc_symbol(
            Some(string),
            SymbolFlags::well_known(),
            AllocationLifetime::Default,
        );
        let bigint = heap.alloc_bigint(BigIntSign::Negative, &[9], AllocationLifetime::Default);
        let object = heap.alloc_object(
            RuntimeObjectRecord::new(None, None, None, None, None),
            AllocationLifetime::Default,
        );
        let environment = heap.alloc_environment(
            RuntimeEnvironmentRecord::new(
                None,
                None,
                None,
                Value::empty_internal_slot(),
                None,
                None,
            ),
            AllocationLifetime::Default,
        );
        let code = heap.alloc_code(
            RuntimeCodeRecord::new(None, None, None),
            AllocationLifetime::Default,
        );
        let realm = heap.alloc_realm(
            RuntimeRealmRecord::new(None, None, None, None),
            AllocationLifetime::Default,
        );
        let shape = heap.alloc_shape(
            RuntimeShapeRecord::new(None, None, 0),
            AllocationLifetime::Default,
        );

        let mut marker = PrimitiveIncrementalMark::new();
        {
            let mut tracer = PrimitiveTracer::new(&mut heap, &mut marker);
            Value::from_string_ref(string).trace_heap_edges(&mut tracer);
            symbol.trace_heap_edges(&mut tracer);
            bigint.trace_heap_edges(&mut tracer);
            object.trace_heap_edges(&mut tracer);
            environment.trace_heap_edges(&mut tracer);
            code.trace_heap_edges(&mut tracer);
            realm.trace_heap_edges(&mut tracer);
            shape.trace_heap_edges(&mut tracer);
        }
        while heap.mark_step(&mut marker, 16).has_more_work() {}
        let stats = marker.trace_stats();

        assert_eq!(stats.values_traced, 2);
        assert_eq!(stats.strings_marked, 1);
        assert_eq!(stats.symbols_marked, 1);
        assert_eq!(stats.bigints_marked, 1);
        assert_eq!(stats.objects_marked, 1);
        assert_eq!(stats.environments_marked, 1);
        assert_eq!(stats.codes_marked, 1);
        assert_eq!(stats.realms_marked, 1);
        assert_eq!(stats.shapes_marked, 1);
    }
}
