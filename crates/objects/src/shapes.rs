use super::{DescriptorAttributes, ObjectFlags, ObjectRef, PropertyKey, ShapeId, Value};

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

/// One shape dependency recorded by a property inline-cache entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PropertyCacheDependency {
    object: ObjectRef,
    shape: ShapeId,
}

impl PropertyCacheDependency {
    #[inline]
    pub const fn new(object: ObjectRef, shape: ShapeId) -> Self {
        Self { object, shape }
    }

    #[inline]
    pub const fn object(self) -> ObjectRef {
        self.object
    }

    #[inline]
    pub const fn shape(self) -> ShapeId {
        self.shape
    }
}

/// Cache purpose used when deriving one named-property inline-cache entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NamedPropertyCachePurpose {
    Load,
    Store,
}

/// Direct path kind for one named-property cache entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NamedPropertyCachePath {
    OwnData,
    OwnDataTransition,
    PrototypeData,
}

/// Result of a direct named-data-property probe.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NamedPropertyDirectGet {
    Data(Value),
    Absent,
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

/// Bit-packed monomorphic `OwnData` inline-cache handler.
///
/// Layout (LSB-first):
///   bit   31     `inline_slot` flag (1 = `Inline`, 0 = `OutOfLine` — same convention as
///                [`INLINE_SLOT_OFFSET_FLAG`])
///   bit   30     writable flag (1 = property is writable, 0 = read-only).
///                Stores short-circuit on read-only entries; loads ignore it.
///   bits  0..30  `slot_offset` (30 bits — 1B values, well above any practical slot count)
///   bits 32..64  receiver shape raw `u32` (`NonZero` — `0` in the high half
///                means "no cache hit path available")
///
/// A whole-word value of `0` is the canonical "no cache hit path" sentinel, made
/// possible by `ShapeId` being `NonZeroU32`. This matches V8's `LoadHandler`
/// bit-field pattern and lets the IC cache hit path do a single 64-bit load +
/// zero-check before unpacking.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NamedPropertyHandler(u64);

const HANDLER_WRITABLE_FLAG: u32 = 0x4000_0000;
const HANDLER_SLOT_OFFSET_MASK: u32 = 0x3FFF_FFFF;

#[expect(
    clippy::cast_possible_truncation,
    reason = "handler words intentionally unpack fixed-width bit fields"
)]
impl NamedPropertyHandler {
    /// Sentinel value indicating "no cache handler available". Set when the
    /// cache is uninitialized, polymorphic, megamorphic, or installed with a
    /// `PrototypeData` entry that the inline cache hit path cannot service.
    pub const NONE: Self = Self(0);

    #[inline]
    #[must_use]
    pub const fn bits(self) -> u64 {
        self.0
    }

    /// Build a cache handler from a cache entry. Returns [`Self::NONE`] for
    /// entries that the inline cache hit path cannot service:
    /// `PrototypeData` paths, multi-dependency entries, any entry whose
    /// `holder_shape` differs from its `receiver_shape`, and any entry whose
    /// slot offset doesn't fit in 30 bits (defensive — never seen in
    /// practice).
    #[inline]
    #[must_use]
    pub const fn from_entry(entry: NamedPropertyCacheEntry) -> Self {
        match entry.path() {
            NamedPropertyCachePath::OwnData => {}
            NamedPropertyCachePath::OwnDataTransition | NamedPropertyCachePath::PrototypeData => {
                return Self::NONE;
            }
        }
        if entry.dependency_count() != 1 {
            return Self::NONE;
        }
        let receiver_shape = entry.receiver_shape();
        if entry.holder_shape().get() != receiver_shape.get() {
            return Self::NONE;
        }
        let raw_shape = receiver_shape.get() as u64;
        let encoded_offset = entry.slot_offset();
        let inline_bit = encoded_offset & INLINE_SLOT_OFFSET_FLAG;
        let offset_bits = encoded_offset & INLINE_SLOT_OFFSET_MASK;
        if offset_bits > HANDLER_SLOT_OFFSET_MASK {
            return Self::NONE;
        }
        let writable_bit = if entry.attrs().writable() {
            HANDLER_WRITABLE_FLAG
        } else {
            0
        };
        let low = inline_bit | writable_bit | offset_bits;
        Self((raw_shape << 32) | (low as u64))
    }

    /// Returns the cached receiver `ShapeId`, or `None` when this is the
    /// sentinel [`Self::NONE`] value.
    #[inline]
    #[must_use]
    pub const fn receiver_shape(self) -> Option<ShapeId> {
        ShapeId::from_raw((self.0 >> 32) as u32)
    }

    /// Decoded slot location. Only meaningful when [`Self::is_valid`] is true.
    #[inline]
    #[must_use]
    pub const fn slot_location(self) -> SlotLocation {
        let low = self.0 as u32;
        let offset = low & HANDLER_SLOT_OFFSET_MASK;
        if low & INLINE_SLOT_OFFSET_FLAG == 0 {
            SlotLocation::OutOfLine(offset)
        } else {
            SlotLocation::Inline(offset)
        }
    }

    /// `true` when the cached property is writable. Stores must check this
    /// and treat a read-only hit as `stored = false` (semantics identical to
    /// the slow chain's `store_to_named_property_cache → Ok(Some(false))`).
    /// Loads ignore this bit.
    #[inline]
    #[must_use]
    pub const fn writable(self) -> bool {
        (self.0 as u32) & HANDLER_WRITABLE_FLAG != 0
    }

    /// `true` when this handler carries a valid monomorphic-OwnData cache
    /// path. `false` for [`Self::NONE`].
    #[inline]
    #[must_use]
    pub const fn is_valid(self) -> bool {
        self.0 != 0
    }
}

/// Bit-packed monomorphic `OwnDataInlineWrite` cache handler.
///
/// Layout — two 64-bit words:
///   `handler_bits` (same layout as [`NamedPropertyHandler`]):
///     bits  0..30  inline slot index
///     bit   30     writable flag (`HANDLER_WRITABLE_FLAG`)
///     bit   31     inline-slot flag (`INLINE_SLOT_OFFSET_FLAG`) — must be set
///     bits 32..64  source `ShapeId` raw `u32` (pre-write shape, `NonZero`)
///   `target_bits`:
///     bits  0..32  target `ShapeId` raw `u32` (post-write shape; equal to
///                  source for non-transitioning writes)
///     bits 32..64  reserved (currently always zero)
///
/// `is_valid()` is `false` when `handler_bits == 0` (the NONE sentinel).
/// `INLINE_SLOT_OFFSET_FLAG` is structurally guaranteed set in every
/// non-NONE handler by [`Self::from_entry`] — the sole constructor — so
/// `slot_location()` can unconditionally return `SlotLocation::Inline`.
///
/// **ShapeId stability assumption:** the handler stores raw shape ids that
/// rely on the existing slab persistence in `ObjectRuntime::shape_metadata`.
/// If shape collection is ever introduced, this struct's consumers (the
/// asm fast path + the IC cache hit-path verifier) would need to participate
/// in pinning or sweep — see the design doc §6.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NamedPropertyInlineWriteHandler {
    handler_bits: u64,
    target_bits: u64,
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "handler words intentionally unpack fixed-width bit fields"
)]
impl NamedPropertyInlineWriteHandler {
    /// Sentinel value indicating "no cache handler available".
    pub const NONE: Self = Self {
        handler_bits: 0,
        target_bits: 0,
    };

    /// Raw handler word: source shape in high 32 bits, inline-slot flag +
    /// writable flag + slot index in low 32 bits. Consumed by the asm
    /// `op_assign_named_property` fast path via `load_named_handler_bits!`.
    #[inline]
    #[must_use]
    pub const fn handler_bits(self) -> u64 {
        self.handler_bits
    }

    /// Raw target word: post-write target `ShapeId` raw u32 in low 32 bits,
    /// high 32 bits reserved. Consumed by the asm `op_assign_named_property`
    /// fast path via `load_named_target_shape!`.
    #[inline]
    #[must_use]
    pub const fn target_bits(self) -> u64 {
        self.target_bits
    }

    /// Build a write handler from a cache entry. Returns [`Self::NONE`] for
    /// entries the asm write fast path cannot service:
    /// - `PrototypeData` paths (no own-data write semantics)
    /// - Multi-dependency entries (more than one shape guard required)
    /// - Out-of-line slot entries (MVP scope; deferred)
    /// - Slot offsets exceeding 30 bits (defensive)
    #[inline]
    #[must_use]
    pub const fn from_entry(entry: NamedPropertyCacheEntry) -> Self {
        match entry.path() {
            NamedPropertyCachePath::OwnData | NamedPropertyCachePath::OwnDataTransition => {}
            NamedPropertyCachePath::PrototypeData => return Self::NONE,
        }
        if entry.dependency_count() != 1 {
            return Self::NONE;
        }
        let encoded_offset = entry.slot_offset();
        if encoded_offset & INLINE_SLOT_OFFSET_FLAG == 0 {
            return Self::NONE;
        }
        let offset_bits = encoded_offset & INLINE_SLOT_OFFSET_MASK;
        if offset_bits > HANDLER_SLOT_OFFSET_MASK {
            return Self::NONE;
        }
        let source_shape = entry.receiver_shape();
        let target_shape = entry.holder_shape();
        let writable_bit = if entry.attrs().writable() {
            HANDLER_WRITABLE_FLAG
        } else {
            0
        };
        let low = INLINE_SLOT_OFFSET_FLAG | writable_bit | offset_bits;
        let handler_bits = ((source_shape.get() as u64) << 32) | (low as u64);
        let target_bits = target_shape.get() as u64;
        Self {
            handler_bits,
            target_bits,
        }
    }

    /// Returns the cached source (pre-write) `ShapeId`.
    #[inline]
    #[must_use]
    pub const fn source_shape(self) -> Option<ShapeId> {
        ShapeId::from_raw((self.handler_bits >> 32) as u32)
    }

    /// Returns the cached target (post-write) `ShapeId`.
    #[inline]
    #[must_use]
    pub const fn target_shape(self) -> Option<ShapeId> {
        ShapeId::from_raw(self.target_bits as u32)
    }

    /// Decoded slot location. Only meaningful when [`Self::is_valid`].
    /// MVP: always returns `SlotLocation::Inline` (outline writes are
    /// filtered to [`Self::NONE`] by [`Self::from_entry`]).
    #[inline]
    #[must_use]
    pub const fn slot_location(self) -> SlotLocation {
        let low = self.handler_bits as u32;
        let offset = low & HANDLER_SLOT_OFFSET_MASK;
        SlotLocation::Inline(offset)
    }

    /// `true` when the cached property is writable.
    #[inline]
    #[must_use]
    pub const fn writable(self) -> bool {
        (self.handler_bits as u32) & HANDLER_WRITABLE_FLAG != 0
    }

    /// `true` when this handler carries a valid `OwnDataInlineWrite` cache path.
    #[inline]
    #[must_use]
    pub const fn is_valid(self) -> bool {
        self.handler_bits != 0
    }
}

/// Bit-packed monomorphic one-hop `PrototypeData` inline-cache handler.
///
/// Phase 3e extension of [`NamedPropertyHandler`] for `PrototypeData` cache
/// entries with `dependency_count == 2` — receiver → one prototype object
/// (the dominant class-method-dispatch / `Object.prototype` pattern). The
/// inline cache hit path validates receiver shape + receiver epoch + prototype
/// shape + prototype epoch, then reads the cached slot from the prototype
/// without touching the slow chain.
///
/// Layout — two 64-bit words, both required for a valid handler:
///   `receiver_word`: receiver shape in the low 32 bits (`NonZeroU32`; `0` ⇒
///   NONE sentinel). High 32 bits reserved (currently always zero).
///   `proto_word`: mirrors [`NamedPropertyHandler`]'s u64 layout — prototype
///   shape in the high 32 bits, slot offset / inline / writable flags in the
///   low 32 bits.
///
/// The whole-handler `NONE` sentinel is `(0, 0)`. Because both shape IDs
/// are `NonZeroU32`, a non-zero `receiver_word` implies a populated handler.
///
/// Receiver-epoch invalidation covers prototype swaps: `set_prototype()`
/// bumps the receiver's `invalidation_epoch` with cause
/// `PrototypeMutation`, so the receiver-epoch compare catches a swap before
/// the prototype object is even examined — no need to store the prototype's
/// `ObjectRef` in the handler.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NamedPropertyProtoHandler {
    receiver_word: u64,
    proto_word: u64,
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "handler words intentionally unpack fixed-width bit fields"
)]
impl NamedPropertyProtoHandler {
    /// Sentinel value indicating "no proto cache handler available". Set when
    /// the cache is uninitialized, polymorphic, megamorphic, an `OwnData`
    /// entry (Phase 3a–3d covers those), or a `PrototypeData` entry whose
    /// dependency chain isn't exactly the one-hop case.
    pub const NONE: Self = Self {
        receiver_word: 0,
        proto_word: 0,
    };

    /// Build a cache handler from a cache entry. Returns [`Self::NONE`] for
    /// entries the one-hop proto cache hit path cannot service:
    /// `OwnData` paths (Phase 3a–3d's [`NamedPropertyHandler`] handles
    /// these), any `PrototypeData` entry with `dependency_count != 2`
    /// (multi-hop chains fall through to the slow path), entries missing
    /// either dependency record, and any entry whose slot offset doesn't
    /// fit in 30 bits.
    #[inline]
    #[must_use]
    pub const fn from_entry(entry: NamedPropertyCacheEntry) -> Self {
        match entry.path() {
            NamedPropertyCachePath::OwnData | NamedPropertyCachePath::OwnDataTransition => {
                return Self::NONE;
            }
            NamedPropertyCachePath::PrototypeData => {}
        }
        if entry.dependency_count() != 2 {
            return Self::NONE;
        }
        let Some(receiver_dep) = entry.dependency(0) else {
            return Self::NONE;
        };
        let Some(proto_dep) = entry.dependency(1) else {
            return Self::NONE;
        };
        let receiver_shape = entry.receiver_shape();
        if receiver_dep.shape().get() != receiver_shape.get() {
            return Self::NONE;
        }
        if proto_dep.shape().get() != entry.holder_shape().get() {
            return Self::NONE;
        }
        let encoded_offset = entry.slot_offset();
        let inline_bit = encoded_offset & INLINE_SLOT_OFFSET_FLAG;
        let offset_bits = encoded_offset & INLINE_SLOT_OFFSET_MASK;
        if offset_bits > HANDLER_SLOT_OFFSET_MASK {
            return Self::NONE;
        }
        let writable_bit = if entry.attrs().writable() {
            HANDLER_WRITABLE_FLAG
        } else {
            0
        };
        let low = inline_bit | writable_bit | offset_bits;
        let proto_shape_raw = entry.holder_shape().get() as u64;
        Self {
            receiver_word: receiver_shape.get() as u64,
            proto_word: (proto_shape_raw << 32) | (low as u64),
        }
    }

    /// Raw receiver-shape word used by `LLInt` feedback mirrors.
    #[inline]
    #[must_use]
    pub const fn receiver_word(self) -> u64 {
        self.receiver_word
    }

    /// Raw prototype-shape/slot word used by `LLInt` feedback mirrors.
    #[inline]
    #[must_use]
    pub const fn proto_word(self) -> u64 {
        self.proto_word
    }

    /// Returns the cached receiver `ShapeId`, or `None` when this is the
    /// sentinel [`Self::NONE`] value.
    #[inline]
    #[must_use]
    pub const fn receiver_shape(self) -> Option<ShapeId> {
        ShapeId::from_raw(self.receiver_word as u32)
    }

    /// Returns the cached prototype `ShapeId`, or `None` when this is the
    /// sentinel [`Self::NONE`] value.
    #[inline]
    #[must_use]
    pub const fn prototype_shape(self) -> Option<ShapeId> {
        ShapeId::from_raw((self.proto_word >> 32) as u32)
    }

    /// Decoded slot location on the prototype holder. Only meaningful when
    /// [`Self::is_valid`] is true.
    #[inline]
    #[must_use]
    pub const fn slot_location(self) -> SlotLocation {
        let low = self.proto_word as u32;
        let offset = low & HANDLER_SLOT_OFFSET_MASK;
        if low & INLINE_SLOT_OFFSET_FLAG == 0 {
            SlotLocation::OutOfLine(offset)
        } else {
            SlotLocation::Inline(offset)
        }
    }

    /// `true` when the cached property is writable. Loads ignore this bit;
    /// it's reserved for a potential future setter-aware store cache hit path.
    #[inline]
    #[must_use]
    pub const fn writable(self) -> bool {
        (self.proto_word as u32) & HANDLER_WRITABLE_FLAG != 0
    }

    /// `true` when this handler carries a valid one-hop `PrototypeData` cache
    /// path. `false` for [`Self::NONE`].
    #[inline]
    #[must_use]
    pub const fn is_valid(self) -> bool {
        self.receiver_word != 0
    }
}

/// Bit-packed monomorphic dense-index keyed IC handler.
///
/// Used by the Phase 3d keyed-property cache hit path for the dense-index family
/// (numeric SMI keys against array-shaped receivers). Encodes the receiver
/// shape and flag snapshot the IC needs to compare against — the slot
/// offset is the runtime SMI index itself, not part of the handler.
///
/// Layout (LSB-first):
///   bits  0..32  receiver shape raw `u32` (`NonZeroU32`; `0` in the low half
///                ⇒ NONE sentinel)
///   bits 32..48  receiver flags ([`ObjectFlags`] u16 bits)
///   bits 48..64  reserved (always zero)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct KeyedDenseIndexHandler(u64);

#[expect(
    clippy::cast_possible_truncation,
    reason = "handler words intentionally unpack fixed-width bit fields"
)]
impl KeyedDenseIndexHandler {
    /// Sentinel value indicating "no cache handler available".
    pub const NONE: Self = Self(0);

    /// Pack a `(receiver_shape, receiver_flags)` pair into a single 64-bit
    /// handler word. Always returns a valid handler — there is no
    /// equivalent of the [`NamedPropertyHandler`] eligibility filter, since
    /// the dense IC only emits cache entries for receivers that already
    /// pass the dense-index cacheability check.
    #[inline]
    #[must_use]
    pub const fn new(receiver_shape: ShapeId, receiver_flags: ObjectFlags) -> Self {
        let shape_raw = receiver_shape.get() as u64;
        let flags_raw = receiver_flags.bits() as u64;
        Self(shape_raw | (flags_raw << 32))
    }

    /// Returns the cached receiver `ShapeId`, or `None` when this is the
    /// sentinel [`Self::NONE`] value.
    #[inline]
    #[must_use]
    pub const fn receiver_shape(self) -> Option<ShapeId> {
        ShapeId::from_raw(self.0 as u32)
    }

    /// Returns the cached receiver flag snapshot.
    #[inline]
    #[must_use]
    pub const fn receiver_flags(self) -> ObjectFlags {
        ObjectFlags::from_bits((self.0 >> 32) as u16)
    }

    /// `true` when this handler carries a valid monomorphic-DenseIndex cache
    /// path. `false` for [`Self::NONE`].
    #[inline]
    #[must_use]
    pub const fn is_valid(self) -> bool {
        (self.0 as u32) != 0
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

#[cfg(test)]
mod inline_write_handler_tests {
    use super::*;

    fn writable_attrs() -> DescriptorAttributes {
        let mut attrs = DescriptorAttributes::empty();
        attrs.set_writable(true);
        attrs
    }

    #[test]
    fn from_transition_entry_packs_source_shape_target_shape_and_inline_slot() {
        let source = ShapeId::from_raw(7).expect("non-zero");
        let target = ShapeId::from_raw(11).expect("non-zero");
        let entry = NamedPropertyCacheEntry::new(
            /* receiver_shape */ source,
            /* holder */ ObjectRef::from_raw(1).expect("non-zero"),
            /* holder_shape */ target,
            /* slot_offset */ INLINE_SLOT_OFFSET_FLAG | 3, // inline slot 3
            /* attrs */ writable_attrs(),
            NamedPropertyCachePath::OwnDataTransition,
            /* dependency_count */ 1,
            /* dependencies */ [None; PROPERTY_CACHE_MAX_DEPENDENCIES],
        );
        let handler = NamedPropertyInlineWriteHandler::from_entry(entry);
        assert!(handler.is_valid());
        assert_eq!(handler.source_shape(), Some(source));
        assert_eq!(handler.target_shape(), Some(target));
        assert_eq!(handler.slot_location(), SlotLocation::Inline(3));
        assert!(handler.writable());
    }

    #[test]
    fn from_own_data_entry_uses_same_source_and_target_shape() {
        let shape = ShapeId::from_raw(42).expect("non-zero");
        let entry = NamedPropertyCacheEntry::new(
            shape,
            ObjectRef::from_raw(1).expect("non-zero"),
            shape, // holder == receiver — no transition
            INLINE_SLOT_OFFSET_FLAG | 0,
            writable_attrs(),
            NamedPropertyCachePath::OwnData,
            1,
            [None; PROPERTY_CACHE_MAX_DEPENDENCIES],
        );
        let handler = NamedPropertyInlineWriteHandler::from_entry(entry);
        assert!(handler.is_valid());
        assert_eq!(handler.source_shape(), Some(shape));
        assert_eq!(handler.target_shape(), Some(shape));
    }

    #[test]
    fn from_outline_entry_is_none() {
        let shape = ShapeId::from_raw(5).expect("non-zero");
        let entry = NamedPropertyCacheEntry::new(
            shape,
            ObjectRef::from_raw(1).expect("non-zero"),
            shape,
            7, // INLINE_SLOT_OFFSET_FLAG NOT set → outline slot 7
            writable_attrs(),
            NamedPropertyCachePath::OwnData,
            1,
            [None; PROPERTY_CACHE_MAX_DEPENDENCIES],
        );
        let handler = NamedPropertyInlineWriteHandler::from_entry(entry);
        assert!(!handler.is_valid());
    }

    #[test]
    fn from_prototype_data_entry_is_none() {
        let receiver = ShapeId::from_raw(1).expect("non-zero");
        let holder = ShapeId::from_raw(2).expect("non-zero");
        let entry = NamedPropertyCacheEntry::new(
            receiver,
            ObjectRef::from_raw(1).expect("non-zero"),
            holder,
            INLINE_SLOT_OFFSET_FLAG | 0,
            writable_attrs(),
            NamedPropertyCachePath::PrototypeData,
            2,
            [None; PROPERTY_CACHE_MAX_DEPENDENCIES],
        );
        let handler = NamedPropertyInlineWriteHandler::from_entry(entry);
        assert!(!handler.is_valid());
    }

    #[test]
    fn none_sentinel_is_invalid() {
        assert!(!NamedPropertyInlineWriteHandler::NONE.is_valid());
    }

    #[test]
    fn from_multi_dependency_entry_is_none() {
        // Even an inline OwnData entry must be rejected when the
        // dependency count exceeds 1 — the asm shape guard cannot
        // validate more than one shape.
        let shape = ShapeId::from_raw(13).expect("non-zero");
        let entry = NamedPropertyCacheEntry::new(
            shape,
            ObjectRef::from_raw(1).expect("non-zero"),
            shape,
            INLINE_SLOT_OFFSET_FLAG | 0,
            writable_attrs(),
            NamedPropertyCachePath::OwnData,
            2, // dependency_count > 1 → must reject
            [None; PROPERTY_CACHE_MAX_DEPENDENCIES],
        );
        let handler = NamedPropertyInlineWriteHandler::from_entry(entry);
        assert!(!handler.is_valid());
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
