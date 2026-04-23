use crate::{
    CodeSlotsRef, EnvironmentSlotsRef, ObjectSlotsRef, PrimitiveBigIntRecord, PrimitiveHeap,
    PrimitiveStringRecord, PrimitiveSymbolRecord, PrimitiveValueCellRecord, PrimitiveValueCellRef,
    RuntimeBoundFunctionRecord, RuntimeCodeRecord, RuntimeEnvironmentRecord, RuntimeFunctionRecord,
    RuntimeObjectRecord, RuntimeRealmRecord, RuntimeShapeRecord, RuntimeSuspendedExecutionRecord,
    SuspendedRegistersRef,
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
    stats: PrimitiveTraceStats,
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
    pub fn scope(&self) -> PrimitiveRootScope<'_> {
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

    fn register(&self, registration: RootRegistration) -> usize {
        if let Some(slot) = self.free.borrow_mut().pop() {
            self.slots.borrow_mut()[slot] = Some(registration);
            slot
        } else {
            let mut slots = self.slots.borrow_mut();
            slots.push(Some(registration));
            slots.len() - 1
        }
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

impl<'a> PrimitiveTracer<'a> {
    #[inline]
    pub fn new(heap: &'a mut PrimitiveHeap) -> Self {
        Self {
            heap,
            stats: PrimitiveTraceStats::default(),
        }
    }

    #[inline]
    pub fn finish(self) -> PrimitiveTraceStats {
        self.stats
    }

    fn total_live_marks(&self) -> usize {
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

    #[inline]
    pub fn mark_value(&mut self, value: Value) {
        self.stats.values_traced += 1;

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
            self.stats.strings_marked += 1;
            if let Some(record) = self.heap.string(id) {
                record.trace_heap_edges(self);
            }
        }
    }

    #[inline]
    pub fn mark_symbol(&mut self, id: SymbolRef) {
        if self.heap.mark_symbol(id) {
            self.stats.symbols_marked += 1;
            if let Some(record) = self.heap.symbol(id) {
                record.trace_heap_edges(self);
            }
        }
    }

    #[inline]
    pub fn mark_bigint(&mut self, id: BigIntRef) {
        if self.heap.mark_bigint(id) {
            self.stats.bigints_marked += 1;
            if let Some(record) = self.heap.bigint(id) {
                record.trace_heap_edges(self);
            }
        }
    }

    #[inline]
    pub fn mark_value_cell(&mut self, id: PrimitiveValueCellRef) {
        if self.heap.mark_value_cell(id) {
            self.stats.value_cells_marked += 1;
            if let Some(record) = self.heap.value_cell(id) {
                record.trace_heap_edges(self);
            }
        }
    }

    #[inline]
    pub fn mark_object(&mut self, id: ObjectRef) {
        if self.heap.mark_object(id) {
            self.stats.objects_marked += 1;
            if let Some(record) = self.heap.object(id) {
                record.trace_heap_edges(self);
                if let Some(function_payload) = record.function_payload() {
                    if let Some(payload) = self.heap.function_payload(function_payload) {
                        payload.trace_heap_edges(self);
                    }
                }
            }
        }
    }

    #[inline]
    pub fn mark_suspended_execution(&mut self, id: SuspendedExecutionRef) {
        if self.heap.mark_suspended_execution(id) {
            self.stats.suspended_executions_marked += 1;
            if let Some(record) = self.heap.suspended_execution(id) {
                record.trace_heap_edges(self);
            }
        }
    }

    #[inline]
    pub fn mark_environment(&mut self, id: EnvironmentRef) {
        if self.heap.mark_environment(id) {
            self.stats.environments_marked += 1;
            if let Some(record) = self.heap.environment(id) {
                record.trace_heap_edges(self);
            }
        }
    }

    #[inline]
    pub fn mark_code(&mut self, id: CodeRef) {
        if self.heap.mark_code(id) {
            self.stats.codes_marked += 1;
            if let Some(record) = self.heap.code(id) {
                record.trace_heap_edges(self);
            }
        }
    }

    #[inline]
    pub fn mark_realm(&mut self, id: RealmRef) {
        if self.heap.mark_realm(id) {
            self.stats.realms_marked += 1;
            if let Some(record) = self.heap.realm(id) {
                record.trace_heap_edges(self);
            }
        }
    }

    #[inline]
    pub fn mark_shape(&mut self, id: ShapeId) {
        if self.heap.mark_shape(id) {
            self.stats.shapes_marked += 1;
            if let Some(record) = self.heap.shape(id) {
                record.trace_heap_edges(self);
            }
        }
    }

    fn mark_object_slots(&mut self, id: ObjectSlotsRef) {
        let Some(values) = self.heap.object_slots(id).map(<[Value]>::to_vec) else {
            return;
        };
        for value in values {
            self.mark_value(value);
        }
    }

    fn mark_suspended_registers(&mut self, id: SuspendedRegistersRef) {
        let Some(values) = self.heap.suspended_registers(id).map(<[Value]>::to_vec) else {
            return;
        };
        for value in values {
            self.mark_value(value);
        }
    }

    fn mark_environment_slots(&mut self, id: EnvironmentSlotsRef) {
        let Some(values) = self.heap.environment_slots(id).map(<[Value]>::to_vec) else {
            return;
        };
        for value in values {
            self.mark_value(value);
        }
    }

    fn mark_code_slots(&mut self, id: CodeSlotsRef) {
        let Some(values) = self.heap.code_slots(id).map(<[Value]>::to_vec) else {
            return;
        };
        for value in values {
            self.mark_value(value);
        }
    }

    fn trace_weak_structures_to_fixpoint(&mut self) -> usize {
        let mut ephemeron_fixes = 0;

        loop {
            let before = self.total_live_marks();
            let weak_maps = self.heap.weak_map_snapshots();
            for (owner, entries) in weak_maps {
                if !self.heap.is_object_marked(owner) {
                    continue;
                }
                for (key, value) in entries {
                    if self.heap.is_weak_ref_marked(key) {
                        self.mark_value(value);
                    }
                }
            }

            let finalization_registries = self.heap.finalization_registry_snapshots();
            for (owner, live_cells, pending_holdings) in finalization_registries {
                if !self.heap.is_object_marked(owner) {
                    continue;
                }
                for holdings in live_cells.into_iter().chain(pending_holdings) {
                    self.mark_value(holdings);
                }
            }

            let after = self.total_live_marks();
            if after == before {
                return ephemeron_fixes;
            }
            ephemeron_fixes += after - before;
        }
    }
}

impl PrimitiveHeap {
    pub fn collect(&mut self, roots: &PrimitiveRoots) -> PrimitiveCollectionStats {
        self.collect_tracing(roots, &())
    }

    pub fn collect_tracing<T: TraceHeapEdges + ?Sized>(
        &mut self,
        roots: &PrimitiveRoots,
        additional_roots: &T,
    ) -> PrimitiveCollectionStats {
        self.clear_string_marks();
        self.clear_symbol_marks();
        self.clear_bigint_marks();
        self.clear_value_cell_marks();
        self.clear_object_marks();
        self.clear_suspended_execution_marks();
        self.clear_environment_marks();
        self.clear_code_marks();
        self.clear_realm_marks();
        self.clear_shape_marks();

        let trace = {
            let mut tracer = PrimitiveTracer::new(self);
            roots.trace_roots(&mut tracer);
            additional_roots.trace_heap_edges(&mut tracer);
            let ephemeron_fixes = tracer.trace_weak_structures_to_fixpoint();
            let trace = tracer.finish();
            (trace, ephemeron_fixes)
        };
        let (weak_refs_cleared, finalization_cells_queued, pending_finalization_registries) =
            self.sweep_weak_state();

        PrimitiveCollectionStats {
            trace: trace.0,
            ephemeron_fixes: trace.1,
            weak_refs_cleared,
            finalization_cells_queued,
            pending_finalization_registries,
            strings_reclaimed: self.sweep_unmarked_strings(),
            symbols_reclaimed: self.sweep_unmarked_symbols(),
            bigints_reclaimed: self.sweep_unmarked_bigints(),
            value_cells_reclaimed: self.sweep_unmarked_value_cells(),
            objects_reclaimed: self.sweep_unmarked_objects(),
            suspended_executions_reclaimed: self.sweep_unmarked_suspended_executions(),
            environments_reclaimed: self.sweep_unmarked_environments(),
            codes_reclaimed: self.sweep_unmarked_codes(),
            realms_reclaimed: self.sweep_unmarked_realms(),
            shapes_reclaimed: self.sweep_unmarked_shapes(),
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
            crate::WeakHeapRef::Object(object) => object.trace_heap_edges(tracer),
            crate::WeakHeapRef::Symbol(symbol) => symbol.trace_heap_edges(tracer),
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
    fn trace_heap_edges(&self, _tracer: &mut PrimitiveTracer<'_>) {}
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

        let mut tracer = PrimitiveTracer::new(&mut heap);
        Value::from_string_ref(string).trace_heap_edges(&mut tracer);
        symbol.trace_heap_edges(&mut tracer);
        bigint.trace_heap_edges(&mut tracer);
        object.trace_heap_edges(&mut tracer);
        environment.trace_heap_edges(&mut tracer);
        code.trace_heap_edges(&mut tracer);
        realm.trace_heap_edges(&mut tracer);
        shape.trace_heap_edges(&mut tracer);
        let stats = tracer.finish();

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
