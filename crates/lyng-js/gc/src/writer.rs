use crate::{
    CodeSlotsRef, EnvironmentSlotsRef, FunctionPayloadRef, ObjectSlotsRef, PrimitiveValueCellRef,
    SideAllocationRef, SuspendedRegistersRef,
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

/// Pass-through chokepoint for writes into traced heap fields.
#[derive(Clone, Copy, Debug, Default)]
pub struct HeapWriter;

impl HeapWriter {
    #[inline]
    pub const fn new() -> Self {
        Self
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
