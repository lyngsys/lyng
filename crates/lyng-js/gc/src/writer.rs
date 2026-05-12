use crate::{
    CodeSlotsRef, EnvironmentSlotsRef, FunctionPayloadRef, ObjectSlotsRef, PrimitiveHeap,
    PrimitiveValueCellRef, SideAllocationRef, SuspendedRegistersRef, TraceHeapEdges,
};
use lyng_js_types::{
    BigIntRef, CodeRef, EnvironmentRef, ObjectRef, RealmRef, ShapeId, StringRef,
    SuspendedExecutionRef, SymbolRef, Value,
};

/// Marker for copyable GC handle fields that may be written through the heap writer.
pub trait HeapRef: Copy {}

macro_rules! impl_heap_ref {
    ($($ty:ty),+ $(,)?) => {
        $(impl HeapRef for $ty {})+
    };
}

impl_heap_ref!(
    BigIntRef,
    CodeRef,
    CodeSlotsRef,
    EnvironmentRef,
    EnvironmentSlotsRef,
    FunctionPayloadRef,
    ObjectRef,
    ObjectSlotsRef,
    PrimitiveValueCellRef,
    RealmRef,
    ShapeId,
    SideAllocationRef,
    StringRef,
    SuspendedExecutionRef,
    SuspendedRegistersRef,
    SymbolRef,
);

impl<T: HeapRef> HeapRef for Option<T> {}

pub trait HeapWriteOwner: Copy {
    fn is_marked_for_incremental_write(self, heap: &PrimitiveHeap) -> bool;
}

macro_rules! impl_heap_write_owner {
    ($($ty:ty => $method:ident),+ $(,)?) => {
        $(
            impl HeapWriteOwner for $ty {
                #[inline]
                fn is_marked_for_incremental_write(self, heap: &PrimitiveHeap) -> bool {
                    heap.$method(self)
                }
            }
        )+
    };
}

impl_heap_write_owner!(
    BigIntRef => is_bigint_marked,
    CodeRef => is_code_marked,
    CodeSlotsRef => is_code_slots_marked,
    EnvironmentRef => is_environment_marked,
    EnvironmentSlotsRef => is_environment_slots_marked,
    FunctionPayloadRef => is_function_payload_marked,
    ObjectRef => is_object_marked,
    ObjectSlotsRef => is_object_slots_marked,
    PrimitiveValueCellRef => is_value_cell_marked,
    RealmRef => is_realm_marked,
    ShapeId => is_shape_marked,
    StringRef => is_string_marked,
    SuspendedExecutionRef => is_suspended_execution_marked,
    SuspendedRegistersRef => is_suspended_registers_marked,
    SymbolRef => is_symbol_marked,
);

/// Pass-through chokepoint for writes into traced heap fields.
#[derive(Clone, Copy, Debug, Default)]
pub struct HeapWriter;

impl HeapWriter {
    #[inline]
    pub const fn new() -> Self {
        Self
    }

    #[inline]
    pub(crate) fn incremental_ref_barrier<Owner, Referent>(
        heap: &mut PrimitiveHeap,
        owner: Owner,
        referent: &Referent,
    ) where
        Owner: HeapWriteOwner,
        Referent: TraceHeapEdges,
    {
        if !heap.incremental_mark_in_progress() || !owner.is_marked_for_incremental_write(heap) {
            return;
        }

        heap.shade_active_incremental_mark(referent);
    }

    #[inline]
    pub(crate) fn incremental_value_barrier<Owner>(
        heap: &mut PrimitiveHeap,
        owner: Owner,
        value: Value,
    ) where
        Owner: HeapWriteOwner,
    {
        if !value_may_reference_heap(value)
            || !heap.incremental_mark_in_progress()
            || !owner.is_marked_for_incremental_write(heap)
        {
            return;
        }

        heap.shade_active_incremental_mark(&value);
    }

    #[inline]
    pub const fn write_ref<T: HeapRef>(&mut self, slot: &mut T, value: T) {
        *slot = value;
    }

    #[inline]
    pub const fn write_value(&mut self, slot: &mut Value, value: Value) {
        *slot = value;
    }
}

#[inline]
const fn value_may_reference_heap(value: Value) -> bool {
    value.as_string_ref().is_some()
        || value.as_symbol_ref().is_some()
        || value.as_bigint_ref().is_some()
        || value.as_object_ref().is_some()
        || value.as_suspended_execution_ref().is_some()
}
