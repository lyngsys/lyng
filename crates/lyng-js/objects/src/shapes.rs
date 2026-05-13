use super::{DescriptorAttributes, ObjectRef, PropertyKey, ShapeId, Value};

pub const PROPERTY_CACHE_MAX_DEPENDENCIES: usize = 4;

/// Named-property storage mode for one object.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NamedPropertyStorageMode {
    ShapeStable,
    Dictionary,
}

/// Indexed-element storage mode for one object.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ElementMode {
    Empty,
    Dense,
    Sparse,
}

/// Coarse invalidation cause family for shape/prototype dependent runtime work.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InvalidationCause {
    PrototypeMutation,
    PropertyRedefinition,
    PropertyDeletion,
    DictionaryTransition,
}

/// Last invalidation event observed for one object.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InvalidationEvent {
    epoch: u64,
    cause: InvalidationCause,
}

impl InvalidationEvent {
    #[inline]
    pub const fn new(epoch: u64, cause: InvalidationCause) -> Self {
        Self { epoch, cause }
    }

    #[inline]
    pub const fn epoch(self) -> u64 {
        self.epoch
    }

    #[inline]
    pub const fn cause(self) -> InvalidationCause {
        self.cause
    }
}

/// One shape/invalidation dependency recorded by a property inline-cache entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PropertyCacheDependency {
    object: ObjectRef,
    shape: ShapeId,
    invalidation_epoch: Option<u64>,
}

impl PropertyCacheDependency {
    #[inline]
    pub const fn new(object: ObjectRef, shape: ShapeId, invalidation_epoch: Option<u64>) -> Self {
        Self {
            object,
            shape,
            invalidation_epoch,
        }
    }

    #[inline]
    pub const fn object(self) -> ObjectRef {
        self.object
    }

    #[inline]
    pub const fn shape(self) -> ShapeId {
        self.shape
    }

    #[inline]
    pub const fn invalidation_epoch(self) -> Option<u64> {
        self.invalidation_epoch
    }
}

/// Cache purpose used when deriving one named-property inline-cache entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NamedPropertyCachePurpose {
    Load,
    Store,
}

/// Fast-path path kind for one named-property cache entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NamedPropertyCachePath {
    OwnData,
    PrototypeData,
}

/// Substrate-owned cache record for one shaped named-property access path.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NamedPropertyCacheEntry {
    receiver_shape: ShapeId,
    holder: ObjectRef,
    holder_shape: ShapeId,
    slot_offset: u32,
    attrs: DescriptorAttributes,
    path: NamedPropertyCachePath,
    dependency_count: u8,
    dependencies: [Option<PropertyCacheDependency>; PROPERTY_CACHE_MAX_DEPENDENCIES],
}

impl NamedPropertyCacheEntry {
    #[inline]
    #[allow(
        clippy::too_many_arguments,
        reason = "cache entry construction mirrors the fixed cache-entry fields"
    )]
    pub(crate) const fn new(
        receiver_shape: ShapeId,
        holder: ObjectRef,
        holder_shape: ShapeId,
        slot_offset: u32,
        attrs: DescriptorAttributes,
        path: NamedPropertyCachePath,
        dependency_count: u8,
        dependencies: [Option<PropertyCacheDependency>; PROPERTY_CACHE_MAX_DEPENDENCIES],
    ) -> Self {
        Self {
            receiver_shape,
            holder,
            holder_shape,
            slot_offset,
            attrs,
            path,
            dependency_count,
            dependencies,
        }
    }

    #[inline]
    pub const fn receiver_shape(self) -> ShapeId {
        self.receiver_shape
    }

    #[inline]
    pub const fn holder(self) -> ObjectRef {
        self.holder
    }

    #[inline]
    pub const fn holder_shape(self) -> ShapeId {
        self.holder_shape
    }

    #[inline]
    pub const fn slot_offset(self) -> u32 {
        self.slot_offset
    }

    #[inline]
    pub const fn attrs(self) -> DescriptorAttributes {
        self.attrs
    }

    #[inline]
    pub const fn path(self) -> NamedPropertyCachePath {
        self.path
    }

    #[inline]
    pub const fn dependency_count(self) -> u8 {
        self.dependency_count
    }

    #[inline]
    pub const fn dependency(self, index: usize) -> Option<PropertyCacheDependency> {
        if index < self.dependency_count as usize {
            self.dependencies[index]
        } else {
            None
        }
    }
}

/// Direct payload stored by one named-property dictionary entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NamedPropertyValue {
    Data(Value),
    Accessor { get: Value, set: Value },
}

impl NamedPropertyValue {
    #[inline]
    pub const fn data(value: Value) -> Self {
        Self::Data(value)
    }

    #[inline]
    pub const fn accessor(get: Value, set: Value) -> Self {
        Self::Accessor { get, set }
    }

    #[inline]
    pub const fn kind(self) -> ShapePropertyKind {
        match self {
            Self::Data(_) => ShapePropertyKind::Data,
            Self::Accessor { .. } => ShapePropertyKind::Accessor,
        }
    }

    #[inline]
    pub const fn data_value(self) -> Option<Value> {
        match self {
            Self::Data(value) => Some(value),
            Self::Accessor { .. } => None,
        }
    }

    #[inline]
    pub const fn accessor_values(self) -> Option<(Value, Value)> {
        match self {
            Self::Data(_) => None,
            Self::Accessor { get, set } => Some((get, set)),
        }
    }
}

/// One direct named-property dictionary entry in slow-path mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NamedPropertyDictionaryEntry {
    pub(crate) key: PropertyKey,
    pub(crate) attrs: DescriptorAttributes,
    pub(crate) payload: NamedPropertyValue,
    pub(crate) enumeration_index: u32,
}

impl NamedPropertyDictionaryEntry {
    #[inline]
    pub const fn new(
        key: PropertyKey,
        attrs: DescriptorAttributes,
        payload: NamedPropertyValue,
        enumeration_index: u32,
    ) -> Self {
        Self {
            key,
            attrs,
            payload,
            enumeration_index,
        }
    }

    #[inline]
    pub const fn key(self) -> PropertyKey {
        self.key
    }

    #[inline]
    pub const fn attrs(self) -> DescriptorAttributes {
        self.attrs
    }

    #[inline]
    pub const fn payload(self) -> NamedPropertyValue {
        self.payload
    }

    #[inline]
    pub const fn enumeration_index(self) -> u32 {
        self.enumeration_index
    }
}

/// One sparse indexed-element entry with normalized attributes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SparseElementEntry {
    payload: NamedPropertyValue,
    attrs: DescriptorAttributes,
}

impl SparseElementEntry {
    #[inline]
    pub const fn new(payload: NamedPropertyValue, attrs: DescriptorAttributes) -> Self {
        Self { payload, attrs }
    }

    #[inline]
    pub const fn payload(self) -> NamedPropertyValue {
        self.payload
    }

    #[inline]
    pub const fn data_value(self) -> Option<Value> {
        self.payload.data_value()
    }

    #[inline]
    pub const fn attrs(self) -> DescriptorAttributes {
        self.attrs
    }
}

/// Named-property storage mode used by one object shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ShapePropertyKind {
    Data,
    Accessor,
}

impl ShapePropertyKind {
    #[inline]
    pub const fn slot_width(self) -> u32 {
        match self {
            Self::Data => 1,
            Self::Accessor => 2,
        }
    }

    #[inline]
    pub const fn is_accessor(self) -> bool {
        matches!(self, Self::Accessor)
    }
}

/// Canonical transition key used to derive one new shape from a parent shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ShapeTransitionKey {
    property_key: PropertyKey,
    property_kind: ShapePropertyKind,
    attrs: DescriptorAttributes,
}

impl ShapeTransitionKey {
    #[inline]
    pub const fn new(
        property_key: PropertyKey,
        property_kind: ShapePropertyKind,
        attrs: DescriptorAttributes,
    ) -> Self {
        Self {
            property_key,
            property_kind,
            attrs,
        }
    }

    #[inline]
    pub const fn property_key(self) -> PropertyKey {
        self.property_key
    }

    #[inline]
    pub const fn property_kind(self) -> ShapePropertyKind {
        self.property_kind
    }

    #[inline]
    pub const fn attrs(self) -> DescriptorAttributes {
        self.attrs
    }
}

/// Slot-offset encoding used by [`ShapeProperty`] and [`NamedPropertyCacheEntry`].
///
/// The high bit of the 32-bit offset distinguishes inline storage (the slot lives in
/// [`ObjectMetadata::inline_slots`], a fixed-size `[Value; 4]` array packed in the runtime's
/// `Vec<Option<ObjectMetadata>>`) from out-of-line storage (the slot lives in the
/// heap-allocated `NamedSlotStorage` array referenced from the object header):
///
/// - `0b1_xxxxxxxx…` → inline at position `xxxxxxxx…` (only positions 0..=3 are valid)
/// - `0b0_xxxxxxxx…` → out-of-line at position `xxxxxxxx…` in the `NamedSlotStorage` array
///
/// Property #5+ on any shape goes out-of-line. Accessor properties (2 slots) that would
/// otherwise span the inline/out-of-line boundary are pushed entirely out-of-line so a single
/// `slot_offset` value identifies both halves of the slot pair.
pub const INLINE_SLOT_OFFSET_FLAG: u32 = 0x8000_0000;
const INLINE_SLOT_OFFSET_MASK: u32 = 0x7FFF_FFFF;

/// Number of inline named-property slots packed into every `ObjectMetadata`.
pub const INLINE_NAMED_SLOT_COUNT: u32 = 4;

/// Decoded slot-offset target — either a position in an object's inline slot array or an
/// index into its heap-side `NamedSlotStorage`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SlotLocation {
    /// Inline slot at index `0..INLINE_NAMED_SLOT_COUNT` of `ObjectMetadata.inline_slots`.
    Inline(u32),
    /// Out-of-line slot at the given index of the heap-allocated `NamedSlotStorage` array.
    OutOfLine(u32),
}

impl SlotLocation {
    /// Encode this location back into a `slot_offset: u32` matching the on-shape encoding.
    #[inline]
    #[must_use]
    pub const fn encode(self) -> u32 {
        match self {
            Self::Inline(index) => INLINE_SLOT_OFFSET_FLAG | (index & INLINE_SLOT_OFFSET_MASK),
            Self::OutOfLine(index) => index & INLINE_SLOT_OFFSET_MASK,
        }
    }

    /// Decode a raw `slot_offset` field as written into a `ShapeProperty` or
    /// `NamedPropertyCacheEntry`.
    #[inline]
    #[must_use]
    pub const fn decode(slot_offset: u32) -> Self {
        if slot_offset & INLINE_SLOT_OFFSET_FLAG == 0 {
            Self::OutOfLine(slot_offset)
        } else {
            Self::Inline(slot_offset & INLINE_SLOT_OFFSET_MASK)
        }
    }

    /// Position of the *second* slot used by an accessor property (getter at this location,
    /// setter at the next consecutive position within the same storage).
    #[inline]
    #[must_use]
    pub const fn accessor_setter_location(self) -> Self {
        match self {
            Self::Inline(index) => Self::Inline(index + 1),
            Self::OutOfLine(index) => Self::OutOfLine(index + 1),
        }
    }

    #[inline]
    #[must_use]
    pub const fn is_inline(self) -> bool {
        matches!(self, Self::Inline(_))
    }
}

/// One canonical property entry recorded by a shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ShapeProperty {
    key: PropertyKey,
    kind: ShapePropertyKind,
    attrs: DescriptorAttributes,
    slot_offset: u32,
    enumeration_index: u32,
}

impl ShapeProperty {
    #[inline]
    pub(crate) const fn from_transition(
        transition: ShapeTransitionKey,
        slot_offset: u32,
        enumeration_index: u32,
    ) -> Self {
        Self {
            key: transition.property_key(),
            kind: transition.property_kind(),
            attrs: transition.attrs(),
            slot_offset,
            enumeration_index,
        }
    }

    #[inline]
    pub const fn key(self) -> PropertyKey {
        self.key
    }

    #[inline]
    pub const fn kind(self) -> ShapePropertyKind {
        self.kind
    }

    #[inline]
    pub const fn attrs(self) -> DescriptorAttributes {
        self.attrs
    }

    /// Raw `slot_offset` field as stored on the shape. Use [`Self::slot_location`] to decode
    /// the inline/out-of-line storage choice.
    #[inline]
    pub const fn slot_offset(self) -> u32 {
        self.slot_offset
    }

    /// Decoded inline-or-out-of-line slot location for this property's first (or only) slot.
    /// For accessor properties, the setter sits at `self.slot_location().accessor_setter_location()`.
    #[inline]
    pub const fn slot_location(self) -> SlotLocation {
        SlotLocation::decode(self.slot_offset)
    }

    #[inline]
    pub const fn slot_width(self) -> u32 {
        self.kind.slot_width()
    }

    #[inline]
    pub const fn enumeration_index(self) -> u32 {
        self.enumeration_index
    }
}

/// Minimal shape allocation request for low-level bootstrap shapes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShapeAllocation {
    parent: Option<ShapeId>,
    prototype_guard: Option<ObjectRef>,
    slot_count: u32,
}

impl ShapeAllocation {
    #[inline]
    pub const fn new(
        parent: Option<ShapeId>,
        prototype_guard: Option<ObjectRef>,
        slot_count: u32,
    ) -> Self {
        Self {
            parent,
            prototype_guard,
            slot_count,
        }
    }

    #[inline]
    pub const fn parent(self) -> Option<ShapeId> {
        self.parent
    }

    #[inline]
    pub const fn prototype_guard(self) -> Option<ObjectRef> {
        self.prototype_guard
    }

    #[inline]
    pub const fn slot_count(self) -> u32 {
        self.slot_count
    }
}

/// Read-only shape header view exposed by the object substrate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShapeRecord {
    id: ShapeId,
    parent: Option<ShapeId>,
    prototype_guard: Option<ObjectRef>,
    slot_count: u32,
    property_count: u32,
    transition_key: Option<ShapeTransitionKey>,
    property: Option<ShapeProperty>,
    uses_flat_lookup: bool,
}

impl ShapeRecord {
    #[inline]
    #[allow(
        clippy::too_many_arguments,
        reason = "shape records are immutable field aggregates allocated by the shape table"
    )]
    pub(crate) const fn new(
        id: ShapeId,
        parent: Option<ShapeId>,
        prototype_guard: Option<ObjectRef>,
        slot_count: u32,
        property_count: u32,
        transition_key: Option<ShapeTransitionKey>,
        property: Option<ShapeProperty>,
        uses_flat_lookup: bool,
    ) -> Self {
        Self {
            id,
            parent,
            prototype_guard,
            slot_count,
            property_count,
            transition_key,
            property,
            uses_flat_lookup,
        }
    }

    #[inline]
    pub const fn id(self) -> ShapeId {
        self.id
    }

    #[inline]
    pub const fn parent(self) -> Option<ShapeId> {
        self.parent
    }

    #[inline]
    pub const fn prototype_guard(self) -> Option<ObjectRef> {
        self.prototype_guard
    }

    #[inline]
    pub const fn slot_count(self) -> u32 {
        self.slot_count
    }

    #[inline]
    pub const fn property_count(self) -> u32 {
        self.property_count
    }

    #[inline]
    pub const fn transition_key(self) -> Option<ShapeTransitionKey> {
        self.transition_key
    }

    #[inline]
    pub const fn property(self) -> Option<ShapeProperty> {
        self.property
    }

    #[inline]
    pub const fn uses_flat_lookup(self) -> bool {
        self.uses_flat_lookup
    }
}
