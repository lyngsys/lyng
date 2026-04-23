use super::{
    ElementStorageRef, FunctionObjectData, NamedSlotStorageRef, ObjectColdData, ObjectFlags,
    ObjectKind, ObjectRef, OrdinaryObjectData, ProxyObjectData, ShapeId, Value,
};

/// Allocation request for one object record and its initial slot buffers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectAllocation {
    pub(crate) kind: ObjectKind,
    pub(crate) flags: ObjectFlags,
    pub(crate) prototype: Option<ObjectRef>,
    pub(crate) shape: ShapeId,
    pub(crate) named_slot_count: usize,
    pub(crate) named_slot_fill: Value,
    pub(crate) element_capacity: usize,
    pub(crate) element_fill: Value,
    pub(crate) ordinary_payload_value: Option<Value>,
    pub(crate) cold: ObjectColdData,
}

impl ObjectAllocation {
    #[inline]
    pub fn ordinary(shape: ShapeId) -> Self {
        Self {
            kind: ObjectKind::Ordinary,
            flags: ObjectFlags::extensible(),
            prototype: None,
            shape,
            named_slot_count: 0,
            named_slot_fill: Value::empty_internal_slot(),
            element_capacity: 0,
            element_fill: Value::array_hole(),
            ordinary_payload_value: None,
            cold: ObjectColdData::Ordinary(OrdinaryObjectData::Plain),
        }
    }

    #[inline]
    pub fn function(shape: ShapeId) -> Self {
        Self {
            kind: ObjectKind::Function,
            flags: ObjectFlags::extensible(),
            prototype: None,
            shape,
            named_slot_count: 0,
            named_slot_fill: Value::empty_internal_slot(),
            element_capacity: 0,
            element_fill: Value::array_hole(),
            ordinary_payload_value: None,
            cold: ObjectColdData::Function(FunctionObjectData::default()),
        }
    }

    #[inline]
    pub fn proxy(shape: ShapeId, data: ProxyObjectData) -> Self {
        Self {
            kind: ObjectKind::Proxy,
            flags: ObjectFlags::extensible(),
            prototype: None,
            shape,
            named_slot_count: 2,
            named_slot_fill: Value::undefined(),
            element_capacity: 0,
            element_fill: Value::array_hole(),
            ordinary_payload_value: None,
            cold: ObjectColdData::Proxy(data),
        }
    }

    #[inline]
    pub fn with_flags(mut self, flags: ObjectFlags) -> Self {
        self.flags = flags;
        self
    }

    #[inline]
    pub fn with_prototype(mut self, prototype: Option<ObjectRef>) -> Self {
        self.prototype = prototype;
        self
    }

    /// Sets the minimum named-slot buffer size. If the shape requires more
    /// slots, the shape wins.
    #[inline]
    pub fn with_named_slot_count(mut self, named_slot_count: usize) -> Self {
        self.named_slot_count = named_slot_count;
        self
    }

    #[inline]
    pub fn with_named_slot_fill(mut self, fill: Value) -> Self {
        self.named_slot_fill = fill;
        self
    }

    #[inline]
    pub fn with_element_capacity(mut self, element_capacity: usize) -> Self {
        self.element_capacity = element_capacity;
        self
    }

    #[inline]
    pub fn with_element_fill(mut self, fill: Value) -> Self {
        self.element_fill = fill;
        self
    }

    #[inline]
    pub fn with_ordinary_payload_value(mut self, value: Value) -> Self {
        self.ordinary_payload_value = Some(value);
        self
    }

    #[inline]
    pub fn with_primitive_wrapper_value(self, value: Value) -> Self {
        self.with_ordinary_payload_value(value)
    }

    #[inline]
    pub fn with_date_value(mut self, value: Value) -> Self {
        self.ordinary_payload_value = Some(value);
        self
    }

    #[inline]
    pub fn with_cold_data(mut self, cold: ObjectColdData) -> Self {
        debug_assert_eq!(
            cold.kind(),
            self.kind,
            "cold payload kind must match the requested object kind"
        );
        self.cold = cold;
        self
    }
}

/// Read-only view over the compact object hot header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObjectHeader {
    id: ObjectRef,
    kind: ObjectKind,
    flags: ObjectFlags,
    prototype: Option<ObjectRef>,
    shape: ShapeId,
    named_slots: Option<NamedSlotStorageRef>,
    elements: Option<ElementStorageRef>,
}

impl ObjectHeader {
    #[inline]
    pub(crate) const fn new(
        id: ObjectRef,
        kind: ObjectKind,
        flags: ObjectFlags,
        prototype: Option<ObjectRef>,
        shape: ShapeId,
        named_slots: Option<NamedSlotStorageRef>,
        elements: Option<ElementStorageRef>,
    ) -> Self {
        Self {
            id,
            kind,
            flags,
            prototype,
            shape,
            named_slots,
            elements,
        }
    }

    #[inline]
    pub const fn id(self) -> ObjectRef {
        self.id
    }

    #[inline]
    pub const fn kind(self) -> ObjectKind {
        self.kind
    }

    #[inline]
    pub const fn flags(self) -> ObjectFlags {
        self.flags
    }

    #[inline]
    pub const fn prototype(self) -> Option<ObjectRef> {
        self.prototype
    }

    #[inline]
    pub const fn shape(self) -> ShapeId {
        self.shape
    }

    #[inline]
    pub const fn named_slots(self) -> Option<NamedSlotStorageRef> {
        self.named_slots
    }

    #[inline]
    pub const fn elements(self) -> Option<ElementStorageRef> {
        self.elements
    }
}

/// Full object view: hot header plus out-of-line cold payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectRecord {
    header: ObjectHeader,
    cold: ObjectColdData,
}

impl ObjectRecord {
    #[inline]
    pub(crate) const fn new(header: ObjectHeader, cold: ObjectColdData) -> Self {
        Self { header, cold }
    }

    #[inline]
    pub const fn header(&self) -> ObjectHeader {
        self.header
    }

    #[inline]
    pub fn cold(&self) -> &ObjectColdData {
        &self.cold
    }
}
