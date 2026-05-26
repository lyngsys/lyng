#![allow(
    clippy::inline_always,
    reason = "Feedback helpers are dispatch hot-path probes where the call boundary shows up in tight opcode loops"
)]

use super::{
    code_index, Agent, AtomId, CodeRef, FeedbackVectorFootprint, ObjectRef, RealmRef, Value, Vm,
};
use lyng_bytecode::{FeedbackSiteDescriptor, FeedbackSiteKind};
use lyng_gc::ValueStoreTarget;
use lyng_objects::{
    FunctionEntryIdentity, KeyedDenseIndexHandler, NamedPropertyCacheEntry, NamedPropertyCachePath,
    NamedPropertyCachePurpose, NamedPropertyHandler, NamedPropertyProtoHandler, ObjectFlags,
    ObjectHeader, ObjectKind, PrimitiveWrapperKind, PropertyCacheDependency, SlotLocation,
    PROPERTY_CACHE_MAX_DEPENDENCIES,
};
use lyng_types::{BuiltinFunctionId, FeedbackSlotId, PropertyKey, ShapeId};
use std::{cmp::Ordering, mem::size_of};

mod polymorphic;

pub(crate) use polymorphic::PolymorphicChain;

const FEEDBACK_ALLOCATION_THRESHOLD: u16 = 2;
const POLYMORPHIC_PROPERTY_CACHE_LIMIT: usize = 8;
const POLYMORPHIC_CALL_CACHE_LIMIT: usize = 8;

/// Phase 3f polymorphic IC cache-hit sidecar capacity. The first `POLY_LIMIT`
/// `entries` are mirrored into a flat `[NamedPropertyHandler; POLY_LIMIT]`
/// array on the feedback site so the inline check can walk shapes 2..N
/// without entering the binary-search slow chain. Entries beyond
/// `POLY_LIMIT` (up to `POLYMORPHIC_PROPERTY_CACHE_LIMIT`) still live in
/// `entries` and are reached via the slow path; mega-poly transition is
/// unchanged. Value chosen by bench evidence on V8 v7 — see
/// `reports/lyng/phase-3f-bench.md`.
pub(in crate::vm) const POLY_LIMIT: usize = 2;

#[inline]
pub(super) fn call_feedback_builtin_is_frame_safe(entry: BuiltinFunctionId) -> bool {
    // Keep this whitelist narrow: these direct-call targets do not inspect caller
    // strictness, dynamically compile source, or re-enter through Function.prototype
    // call helpers, so dispatching from a monomorphic feedback entry preserves the
    // general call path's caller frame and callee realm behavior.
    entry == lyng_types::regexp_exec_builtin()
        || entry == lyng_types::regexp_symbol_replace_builtin()
        || entry == lyng_types::regexp_test_builtin()
        || entry == lyng_types::string_char_code_at_builtin()
        || entry == lyng_types::string_from_char_code_builtin()
        || entry == lyng_types::string_replace_builtin()
        || entry == lyng_types::string_to_upper_case_builtin()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FeedbackInlineCacheState {
    Uninitialized,
    Monomorphic,
    Polymorphic,
    Megamorphic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FeedbackKeyedPropertyFamily {
    DenseIndex,
    NamedAtom,
    Generic,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedPropertyCacheEntrySnapshot {
    receiver_shape: ShapeId,
    holder: ObjectRef,
    holder_shape: ShapeId,
    slot_offset: u32,
    path: NamedPropertyCachePath,
    dependencies: Vec<PropertyCacheDependency>,
}

impl NamedPropertyCacheEntrySnapshot {
    #[inline]
    fn from_entry(entry: NamedPropertyCacheEntry) -> Self {
        let dependencies = (0..usize::from(entry.dependency_count()))
            .filter_map(|index| entry.dependency(index))
            .collect();
        Self {
            receiver_shape: entry.receiver_shape(),
            holder: entry.holder(),
            holder_shape: entry.holder_shape(),
            slot_offset: entry.slot_offset(),
            path: entry.path(),
            dependencies,
        }
    }

    #[inline]
    pub const fn receiver_shape(&self) -> ShapeId {
        self.receiver_shape
    }

    #[inline]
    pub const fn holder(&self) -> ObjectRef {
        self.holder
    }

    #[inline]
    pub const fn holder_shape(&self) -> ShapeId {
        self.holder_shape
    }

    #[inline]
    pub const fn slot_offset(&self) -> u32 {
        self.slot_offset
    }

    #[inline]
    pub const fn path(&self) -> NamedPropertyCachePath {
        self.path
    }

    #[inline]
    pub fn dependencies(&self) -> &[PropertyCacheDependency] {
        &self.dependencies
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedPropertyFeedbackSnapshot {
    execution_count: u32,
    state: FeedbackInlineCacheState,
    entries: Vec<NamedPropertyCacheEntrySnapshot>,
}

impl NamedPropertyFeedbackSnapshot {
    #[inline]
    const fn uninitialized(execution_count: u32) -> Self {
        Self {
            execution_count,
            state: FeedbackInlineCacheState::Uninitialized,
            entries: Vec::new(),
        }
    }

    #[inline]
    fn from_feedback(feedback: &NamedPropertyFeedback) -> Self {
        Self {
            execution_count: feedback.execution_count,
            state: feedback.cache_state.into(),
            entries: feedback
                .active_entries()
                .map(NamedPropertyCacheEntrySnapshot::from_entry)
                .collect(),
        }
    }

    #[inline]
    pub const fn execution_count(&self) -> u32 {
        self.execution_count
    }

    #[inline]
    pub const fn state(&self) -> FeedbackInlineCacheState {
        self.state
    }

    #[inline]
    pub fn entries(&self) -> &[NamedPropertyCacheEntrySnapshot] {
        &self.entries
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedNamedPropertyCacheEntrySnapshot {
    atom: AtomId,
    entry: NamedPropertyCacheEntrySnapshot,
}

impl KeyedNamedPropertyCacheEntrySnapshot {
    #[inline]
    fn from_entry(entry: KeyedNamedPropertyCacheEntry) -> Self {
        Self {
            atom: entry.atom,
            entry: NamedPropertyCacheEntrySnapshot::from_entry(entry.entry),
        }
    }

    #[inline]
    pub const fn atom(&self) -> AtomId {
        self.atom
    }

    #[inline]
    pub const fn entry(&self) -> &NamedPropertyCacheEntrySnapshot {
        &self.entry
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedPropertyFeedbackSnapshot {
    execution_count: u32,
    state: FeedbackInlineCacheState,
    family: Option<FeedbackKeyedPropertyFamily>,
    entries: Vec<KeyedNamedPropertyCacheEntrySnapshot>,
    dense_entries: Vec<DenseIndexCacheEntrySnapshot>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LlIntNamedPropertyHeader {
    OwnInline {
        handler_bits: u64,
    },
    OwnOutline {
        handler_bits: u64,
    },
    ProtoInline {
        receiver_word: u64,
        proto_word: u64,
    },
    OwnPolymorphic {
        slot0_handler_bits: u64,
        slot1_handler_bits: u64,
    },
}

impl KeyedPropertyFeedbackSnapshot {
    #[inline]
    const fn uninitialized(execution_count: u32) -> Self {
        Self {
            execution_count,
            state: FeedbackInlineCacheState::Uninitialized,
            family: None,
            entries: Vec::new(),
            dense_entries: Vec::new(),
        }
    }

    #[inline]
    fn from_feedback(feedback: &KeyedPropertyFeedback) -> Self {
        Self {
            execution_count: feedback.execution_count,
            state: feedback.cache_state.into(),
            family: feedback.family.map(FeedbackKeyedPropertyFamily::from),
            entries: feedback
                .active_named_entries()
                .map(KeyedNamedPropertyCacheEntrySnapshot::from_entry)
                .collect(),
            dense_entries: feedback
                .active_dense_entries()
                .map(DenseIndexCacheEntrySnapshot::from_entry)
                .collect(),
        }
    }

    #[inline]
    pub const fn execution_count(&self) -> u32 {
        self.execution_count
    }

    #[inline]
    pub const fn state(&self) -> FeedbackInlineCacheState {
        self.state
    }

    #[inline]
    pub const fn family(&self) -> Option<FeedbackKeyedPropertyFamily> {
        self.family
    }

    #[inline]
    pub fn entries(&self) -> &[KeyedNamedPropertyCacheEntrySnapshot] {
        &self.entries
    }

    #[inline]
    pub fn dense_entries(&self) -> &[DenseIndexCacheEntrySnapshot] {
        &self.dense_entries
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CallCacheEntrySnapshot {
    callee: ObjectRef,
    callee_shape: ShapeId,
    realm: Option<RealmRef>,
    builtin: Option<BuiltinFunctionId>,
}

impl CallCacheEntrySnapshot {
    #[inline]
    const fn from_entry(entry: CallCacheEntry) -> Self {
        Self {
            callee: entry.callee,
            callee_shape: entry.callee_shape,
            realm: entry.realm,
            builtin: entry.builtin,
        }
    }

    #[inline]
    pub const fn callee(self) -> ObjectRef {
        self.callee
    }

    #[inline]
    pub const fn callee_shape(self) -> ShapeId {
        self.callee_shape
    }

    #[inline]
    pub const fn realm(self) -> Option<RealmRef> {
        self.realm
    }

    #[inline]
    pub const fn builtin(self) -> Option<BuiltinFunctionId> {
        self.builtin
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallFeedbackSnapshot {
    execution_count: u32,
    expected_arity: Option<u16>,
    state: FeedbackInlineCacheState,
    entries: Vec<CallCacheEntrySnapshot>,
}

impl CallFeedbackSnapshot {
    #[inline]
    const fn uninitialized(expected_arity: Option<u16>, execution_count: u32) -> Self {
        Self {
            execution_count,
            expected_arity,
            state: FeedbackInlineCacheState::Uninitialized,
            entries: Vec::new(),
        }
    }

    #[inline]
    fn from_feedback(feedback: &CallFeedback) -> Self {
        Self {
            execution_count: feedback.execution_count,
            expected_arity: feedback.expected_arity,
            state: feedback.cache_state.into(),
            entries: feedback
                .active_entries()
                .map(CallCacheEntrySnapshot::from_entry)
                .collect(),
        }
    }

    #[inline]
    pub const fn execution_count(&self) -> u32 {
        self.execution_count
    }

    #[inline]
    pub const fn expected_arity(&self) -> Option<u16> {
        self.expected_arity
    }

    #[inline]
    pub const fn state(&self) -> FeedbackInlineCacheState {
        self.state
    }

    #[inline]
    pub fn entries(&self) -> &[CallCacheEntrySnapshot] {
        &self.entries
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConstructCacheEntrySnapshot {
    constructor: ObjectRef,
    constructor_shape: ShapeId,
    realm: Option<RealmRef>,
    created_shape: Option<ShapeId>,
}

impl ConstructCacheEntrySnapshot {
    #[inline]
    const fn from_entry(entry: ConstructCacheEntry) -> Self {
        Self {
            constructor: entry.constructor,
            constructor_shape: entry.constructor_shape,
            realm: entry.realm,
            created_shape: entry.created_shape,
        }
    }

    #[inline]
    pub const fn constructor(self) -> ObjectRef {
        self.constructor
    }

    #[inline]
    pub const fn constructor_shape(self) -> ShapeId {
        self.constructor_shape
    }

    #[inline]
    pub const fn realm(self) -> Option<RealmRef> {
        self.realm
    }

    #[inline]
    pub const fn created_shape(self) -> Option<ShapeId> {
        self.created_shape
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstructFeedbackSnapshot {
    execution_count: u32,
    expected_arity: Option<u16>,
    state: FeedbackInlineCacheState,
    entries: Vec<ConstructCacheEntrySnapshot>,
}

impl ConstructFeedbackSnapshot {
    #[inline]
    const fn uninitialized(expected_arity: Option<u16>, execution_count: u32) -> Self {
        Self {
            execution_count,
            expected_arity,
            state: FeedbackInlineCacheState::Uninitialized,
            entries: Vec::new(),
        }
    }

    #[inline]
    fn from_feedback(feedback: &ConstructFeedback) -> Self {
        Self {
            execution_count: feedback.execution_count,
            expected_arity: feedback.expected_arity,
            state: feedback.cache_state.into(),
            entries: feedback
                .active_entries()
                .map(ConstructCacheEntrySnapshot::from_entry)
                .collect(),
        }
    }

    #[inline]
    pub const fn execution_count(&self) -> u32 {
        self.execution_count
    }

    #[inline]
    pub const fn expected_arity(&self) -> Option<u16> {
        self.expected_arity
    }

    #[inline]
    pub const fn state(&self) -> FeedbackInlineCacheState {
        self.state
    }

    #[inline]
    pub fn entries(&self) -> &[ConstructCacheEntrySnapshot] {
        &self.entries
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DenseIndexCacheEntrySnapshot {
    receiver_shape: ShapeId,
    receiver_flags: ObjectFlags,
}

impl DenseIndexCacheEntrySnapshot {
    #[inline]
    const fn from_entry(entry: DenseIndexCacheEntry) -> Self {
        Self {
            receiver_shape: entry.receiver_shape,
            receiver_flags: entry.receiver_flags,
        }
    }

    #[inline]
    pub const fn receiver_shape(self) -> ShapeId {
        self.receiver_shape
    }

    #[inline]
    pub const fn receiver_flags(self) -> ObjectFlags {
        self.receiver_flags
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FeedbackSiteDetail {
    Arithmetic,
    Comparison,
    NamedProperty(NamedPropertyFeedbackSnapshot),
    KeyedProperty(KeyedPropertyFeedbackSnapshot),
    Call(CallFeedbackSnapshot),
    Construct(ConstructFeedbackSnapshot),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeedbackSiteSnapshot {
    slot: FeedbackSlotId,
    instruction_offset: u32,
    kind: FeedbackSiteKind,
    execution_count: u32,
    detail: FeedbackSiteDetail,
}

impl FeedbackSiteSnapshot {
    #[inline]
    const fn new(
        descriptor: FeedbackSiteDescriptor,
        execution_count: u32,
        detail: FeedbackSiteDetail,
    ) -> Self {
        Self {
            slot: descriptor.slot(),
            instruction_offset: descriptor.instruction_offset(),
            kind: descriptor.kind(),
            execution_count,
            detail,
        }
    }

    #[inline]
    pub const fn slot(&self) -> FeedbackSlotId {
        self.slot
    }

    #[inline]
    pub const fn instruction_offset(&self) -> u32 {
        self.instruction_offset
    }

    #[inline]
    pub const fn kind(&self) -> FeedbackSiteKind {
        self.kind
    }

    #[inline]
    pub const fn execution_count(&self) -> u32 {
        self.execution_count
    }

    #[inline]
    pub fn detail(&self) -> FeedbackSiteDetail {
        self.detail.clone()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeedbackVectorSnapshot {
    allocated: bool,
    warmup_counter: u16,
    slot_count: usize,
    live_site_count: usize,
    sites: Vec<FeedbackSiteSnapshot>,
}

impl FeedbackVectorSnapshot {
    #[inline]
    const fn new(
        allocated: bool,
        warmup_counter: u16,
        slot_count: usize,
        sites: Vec<FeedbackSiteSnapshot>,
    ) -> Self {
        let live_site_count = sites.len();
        Self {
            allocated,
            warmup_counter,
            slot_count,
            live_site_count,
            sites,
        }
    }

    #[inline]
    pub const fn allocated(&self) -> bool {
        self.allocated
    }

    #[inline]
    pub const fn warmup_counter(&self) -> u16 {
        self.warmup_counter
    }

    #[inline]
    pub const fn slot_count(&self) -> usize {
        self.slot_count
    }

    #[inline]
    pub const fn live_site_count(&self) -> usize {
        self.live_site_count
    }

    #[inline]
    pub fn sites(&self) -> &[FeedbackSiteSnapshot] {
        &self.sites
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum InlineCacheState {
    Uninitialized,
    Monomorphic,
    Polymorphic,
    Megamorphic,
}

impl From<InlineCacheState> for FeedbackInlineCacheState {
    #[inline]
    fn from(value: InlineCacheState) -> Self {
        match value {
            InlineCacheState::Uninitialized => Self::Uninitialized,
            InlineCacheState::Monomorphic => Self::Monomorphic,
            InlineCacheState::Polymorphic => Self::Polymorphic,
            InlineCacheState::Megamorphic => Self::Megamorphic,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum KeyedPropertyFamily {
    DenseIndex,
    NamedAtom,
    Generic,
}

impl From<KeyedPropertyFamily> for FeedbackKeyedPropertyFamily {
    #[inline]
    fn from(value: KeyedPropertyFamily) -> Self {
        match value {
            KeyedPropertyFamily::DenseIndex => Self::DenseIndex,
            KeyedPropertyFamily::NamedAtom => Self::NamedAtom,
            KeyedPropertyFamily::Generic => Self::Generic,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArithmeticFeedback {
    execution_count: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComparisonFeedback {
    execution_count: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NamedPropertyFeedback {
    execution_count: u32,
    cache_state: InlineCacheState,
    entry_count: u8,
    entries: [Option<NamedPropertyCacheEntry>; POLYMORPHIC_PROPERTY_CACHE_LIMIT],
    /// Bit-packed handler derived from `entries[0]` whenever the cache is
    /// monomorphic and the entry is `OwnData`. `NamedPropertyHandler::NONE` in
    /// every other state. Lets the IC cache hit path skip the four-deep call chain
    /// on the common case. Sidecar — `entries` remains the system of record.
    monomorphic_own_data_handler: NamedPropertyHandler,
    /// Phase 3e proto cache handler — packed receiver shape, prototype shape,
    /// and prototype slot offset derived from `entries[0]` whenever the
    /// cache is monomorphic and the entry is a one-hop `PrototypeData`
    /// chain (`dependency_count == 2`). `NamedPropertyProtoHandler::NONE`
    /// in every other state, including multi-hop `PrototypeData`. Mutually
    /// exclusive with `monomorphic_own_data_handler` in practice — a cache entry is
    /// either `OwnData` or `PrototypeData`.
    monomorphic_proto_data_handler: NamedPropertyProtoHandler,
    /// Phase 3f polymorphic-OwnData sidecar — mirrors the first `POLY_LIMIT`
    /// `entries` whenever the cache is polymorphic and the entry is
    /// cache-hit eligible (`NamedPropertyHandler::from_entry` returns a
    /// valid handler). Each slot is `NamedPropertyHandler::NONE` when the
    /// matching entry is absent, is a `PrototypeData` path, or fails any
    /// other packing eligibility check. The inline cache hit path walks
    /// `polymorphic_own_data_handlers` after the monomorphic word miss and before the
    /// proto-cache hit path, skipping the slow chain on shapes `2..POLY_LIMIT`.
    /// Mega-poly transition leaves this array zeroed. Sidecar — `entries`
    /// remains the system of record.
    polymorphic_own_data_handlers: [NamedPropertyHandler; POLY_LIMIT],
    /// Spec 2 Phase A: per-site install generation. Bumped on every install /
    /// re-install. `AdaptiveProtoLoad` watchpoints carry the generation at
    /// registration time and no-op when this has advanced past it.
    pub(crate) generation: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct KeyedNamedPropertyCacheEntry {
    atom: AtomId,
    entry: NamedPropertyCacheEntry,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DenseIndexCacheEntry {
    receiver_shape: ShapeId,
    receiver_flags: ObjectFlags,
}

impl DenseIndexCacheEntry {
    #[inline]
    const fn new(receiver_shape: ShapeId, receiver_flags: ObjectFlags) -> Self {
        Self {
            receiver_shape,
            receiver_flags,
        }
    }

    #[inline]
    const fn from_header(header: ObjectHeader) -> Self {
        Self::new(header.shape(), header.flags())
    }

    #[inline]
    fn matches_header(self, header: ObjectHeader) -> bool {
        self.receiver_shape == header.shape() && self.receiver_flags == header.flags()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyedPropertyFeedback {
    execution_count: u32,
    family: Option<KeyedPropertyFamily>,
    cache_state: InlineCacheState,
    named_entry_count: u8,
    named_entries: [Option<KeyedNamedPropertyCacheEntry>; POLYMORPHIC_PROPERTY_CACHE_LIMIT],
    dense_entry_count: u8,
    dense_entries: [Option<DenseIndexCacheEntry>; POLYMORPHIC_PROPERTY_CACHE_LIMIT],
    /// Phase 3d named-atom cache handler — packed shape + `slot_offset` +
    /// writable derived from `named_entries[0].entry` when the cache is
    /// monomorphic and the family is `NamedAtom`. NONE otherwise.
    monomorphic_named_own_data_handler: NamedPropertyHandler,
    /// Raw `AtomId` (`NonZeroU32`) of `named_entries[0]` when cache-hit
    /// eligible; `0` otherwise. The cache hit path compares against the runtime
    /// atom of the keyed access.
    monomorphic_named_atom: u32,
    /// Phase 3e named-atom proto cache handler — packed receiver shape,
    /// prototype shape, and slot offset derived from
    /// `named_entries[0].entry` when monomorphic, family is `NamedAtom`,
    /// and the entry is one-hop `PrototypeData`. NONE otherwise; mutually
    /// exclusive with `monomorphic_named_own_data_handler` for any given keyed-atom
    /// site.
    monomorphic_named_proto_data_handler: NamedPropertyProtoHandler,
    /// Phase 3d dense-index cache handler — packed shape + flags derived
    /// from `dense_entries[0]` when the cache is monomorphic and the
    /// family is `DenseIndex`. NONE otherwise.
    monomorphic_dense_index_handler: KeyedDenseIndexHandler,
    /// Phase 3f polymorphic named-atom `OwnData` sidecar — mirrors the first
    /// `POLY_LIMIT` `named_entries` whenever the cache is polymorphic, the
    /// family is `NamedAtom`, and the underlying entry packs into a valid
    /// `NamedPropertyHandler`. Each slot is `NamedPropertyHandler::NONE`
    /// for the corresponding miss conditions (absent entry, `PrototypeData`
    /// path, or unpackable handler).
    polymorphic_named_own_data_handlers: [NamedPropertyHandler; POLY_LIMIT],
    /// Raw `AtomId` (`NonZeroU32`) paired with each `polymorphic_named_own_data_handlers[i]`.
    /// Zero when the slot is empty. The inline cache hit path matches both the
    /// receiver shape (carried in the handler) and the atom against the
    /// runtime keyed access.
    polymorphic_named_atoms: [u32; POLY_LIMIT],
    /// Phase 3f polymorphic dense-index sidecar — mirrors the first
    /// `POLY_LIMIT` `dense_entries` whenever the cache is polymorphic and
    /// the family is `DenseIndex`. Each slot is `KeyedDenseIndexHandler::NONE`
    /// for empty / unpackable entries.
    polymorphic_dense_index_handlers: [KeyedDenseIndexHandler; POLY_LIMIT],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CallCacheEntry {
    callee: ObjectRef,
    callee_shape: ShapeId,
    realm: Option<RealmRef>,
    builtin: Option<BuiltinFunctionId>,
}

impl CallCacheEntry {
    #[inline]
    fn from_callee(agent: &Agent, callee: ObjectRef) -> Option<Self> {
        let callee_shape = agent
            .objects()
            .object_header(agent.heap().view(), callee)?
            .shape();
        let function = agent.objects().function_data(callee);
        let realm = function.and_then(lyng_objects::FunctionObjectData::realm);
        let builtin = function.and_then(|function| {
            let FunctionEntryIdentity::Native(entry) = function.entry()? else {
                return None;
            };
            entry.builtin_entry()
        });
        Some(Self {
            callee,
            callee_shape,
            realm,
            builtin,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CallFeedback {
    execution_count: u32,
    expected_arity: Option<u16>,
    cache_state: InlineCacheState,
    entry_count: u8,
    entries: [Option<CallCacheEntry>; POLYMORPHIC_CALL_CACHE_LIMIT],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ConstructCacheEntry {
    constructor: ObjectRef,
    constructor_shape: ShapeId,
    realm: Option<RealmRef>,
    created_shape: Option<ShapeId>,
}

impl ConstructCacheEntry {
    #[inline]
    fn from_constructor(
        agent: &Agent,
        constructor: ObjectRef,
        created: Option<ObjectRef>,
    ) -> Option<Self> {
        let constructor_shape = agent
            .objects()
            .object_header(agent.heap().view(), constructor)?
            .shape();
        let realm = agent
            .objects()
            .function_data(constructor)
            .and_then(lyng_objects::FunctionObjectData::realm);
        let created_shape = Self::created_shape(agent, created);
        Some(Self {
            constructor,
            constructor_shape,
            realm,
            created_shape,
        })
    }

    #[inline]
    fn created_shape(agent: &Agent, created: Option<ObjectRef>) -> Option<ShapeId> {
        created.and_then(|object| {
            agent
                .objects()
                .object_header(agent.heap().view(), object)
                .map(lyng_objects::ObjectHeader::shape)
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConstructFeedback {
    execution_count: u32,
    expected_arity: Option<u16>,
    cache_state: InlineCacheState,
    entry_count: u8,
    entries: [Option<ConstructCacheEntry>; POLYMORPHIC_CALL_CACHE_LIMIT],
}

// Per-site feedback content. Promoted to `pub(crate)` so the DSL-0b
// flat-array storage (`crate::dsl::feedback_flat`) can wrap it inside a
// `FeedbackEntry`. The enum variants are still constructed only through
// the methods on this file; outside this module the type is opaque.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FeedbackSiteState {
    Arithmetic(ArithmeticFeedback),
    Comparison(ComparisonFeedback),
    NamedProperty(NamedPropertyFeedback),
    KeyedProperty(KeyedPropertyFeedback),
    Call(CallFeedback),
    Construct(ConstructFeedback),
}

impl NamedPropertyFeedback {
    #[inline]
    const fn new() -> Self {
        Self {
            execution_count: 0,
            cache_state: InlineCacheState::Uninitialized,
            entry_count: 0,
            entries: [None; POLYMORPHIC_PROPERTY_CACHE_LIMIT],
            monomorphic_own_data_handler: NamedPropertyHandler::NONE,
            monomorphic_proto_data_handler: NamedPropertyProtoHandler::NONE,
            polymorphic_own_data_handlers: [NamedPropertyHandler::NONE; POLY_LIMIT],
            generation: 0,
        }
    }

    /// Recompute every cache-hit sidecar (`monomorphic_own_data_handler`,
    /// `monomorphic_proto_data_handler`, `polymorphic_own_data_handlers`, and their paired
    /// invalidation-epoch snapshots) from the current cache state. Called
    /// after any mutation that may have changed an active entry, the
    /// `cache_state`, or the `entry_count`.
    ///
    /// State semantics:
    /// - `Uninitialized` / `Megamorphic` — all sidecars cleared.
    /// - `Monomorphic` — populate `monomorphic_own_data_handler` (`OwnData`) or
    ///   `monomorphic_proto_data_handler` (one-hop `PrototypeData`) from `entries[0]`.
    /// - `Polymorphic` (Phase 3f) — pack the first `POLY_LIMIT` `entries`
    ///   into `polymorphic_own_data_handlers` via `NamedPropertyHandler::from_entry`;
    ///   non-OwnData entries and unpackable handlers leave a `NONE` slot
    ///   that the inline walk skips. Entries `[POLY_LIMIT..entry_count]`
    ///   remain reachable only via the binary-search slow chain.
    #[inline]
    const fn refresh_monomorphic_own_data_handler(&mut self) {
        self.monomorphic_own_data_handler = NamedPropertyHandler::NONE;
        self.monomorphic_proto_data_handler = NamedPropertyProtoHandler::NONE;
        let mut i = 0;
        while i < POLY_LIMIT {
            self.polymorphic_own_data_handlers[i] = NamedPropertyHandler::NONE;
            i += 1;
        }
        match self.cache_state {
            InlineCacheState::Monomorphic => {
                let Some(entry) = self.entries[0] else {
                    return;
                };
                match entry.path() {
                    NamedPropertyCachePath::OwnData => {
                        self.monomorphic_own_data_handler = NamedPropertyHandler::from_entry(entry);
                    }
                    NamedPropertyCachePath::OwnDataTransition => {}
                    NamedPropertyCachePath::PrototypeData => {
                        let handler = NamedPropertyProtoHandler::from_entry(entry);
                        if handler.is_valid() {
                            self.monomorphic_proto_data_handler = handler;
                        }
                    }
                }
            }
            InlineCacheState::Polymorphic => {
                let active = self.entry_count as usize;
                let limit = if active < POLY_LIMIT {
                    active
                } else {
                    POLY_LIMIT
                };
                let mut idx = 0;
                while idx < limit {
                    if let Some(entry) = self.entries[idx] {
                        let handler = NamedPropertyHandler::from_entry(entry);
                        if handler.is_valid() {
                            self.polymorphic_own_data_handlers[idx] = handler;
                        }
                    }
                    idx += 1;
                }
            }
            InlineCacheState::Uninitialized | InlineCacheState::Megamorphic => {}
        }
    }

    #[inline]
    fn try_load(&self, agent: &Agent, receiver: ObjectRef) -> Option<Value> {
        match self.cache_state {
            InlineCacheState::Monomorphic | InlineCacheState::Polymorphic => {}
            InlineCacheState::Uninitialized | InlineCacheState::Megamorphic => return None,
        }
        let receiver_shape = agent
            .objects()
            .object_header(agent.heap().view(), receiver)?
            .shape();
        let entry = self.entries[self.find_entry_index(receiver_shape)?]?;
        if let Ok(Some(value)) =
            agent
                .objects()
                .load_from_named_property_cache(agent.heap().view(), receiver, entry)
        {
            return Some(value);
        }
        None
    }

    #[inline]
    fn try_store(
        &self,
        agent: &mut Agent,
        receiver: ObjectRef,
        atom: AtomId,
        value: Value,
    ) -> Option<bool> {
        match self.cache_state {
            InlineCacheState::Monomorphic | InlineCacheState::Polymorphic => {}
            InlineCacheState::Uninitialized | InlineCacheState::Megamorphic => return None,
        }
        let receiver_shape = agent
            .objects()
            .object_header(agent.heap().view(), receiver)?
            .shape();
        let entry = self.entries[self.find_entry_index(receiver_shape)?]?;
        let result = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.store_to_named_property_cache(
                &mut mutator,
                receiver,
                PropertyKey::from_atom(atom),
                entry,
                value,
            )
        });
        if let Ok(Some(stored)) = result {
            return Some(stored);
        }
        None
    }

    #[inline]
    fn observe_slow_path(&mut self, plan: Option<NamedPropertyCacheEntry>) {
        let Some(plan) = plan else {
            self.promote_to_megamorphic();
            return;
        };
        match self.cache_state {
            InlineCacheState::Megamorphic => {}
            InlineCacheState::Uninitialized => self.install_first_entry(plan),
            InlineCacheState::Monomorphic | InlineCacheState::Polymorphic => {
                match self.search_entry_index(plan.receiver_shape()) {
                    Ok(index) => {
                        self.entries[index] = Some(plan);
                        // Any update to entries[0..POLY_LIMIT] may invalidate
                        // the inline sidecar (handler word, slot location, or
                        // dependency epoch all derive from the entry).
                        if index < POLY_LIMIT {
                            self.refresh_monomorphic_own_data_handler();
                        }
                    }
                    Err(index) => self.insert_entry_at(index, plan),
                }
            }
        }
    }

    #[inline]
    fn active_entries(&self) -> impl Iterator<Item = NamedPropertyCacheEntry> + '_ {
        self.entries
            .iter()
            .take(usize::from(self.entry_count))
            .filter_map(|entry| *entry)
    }

    #[inline]
    const fn install_first_entry(&mut self, entry: NamedPropertyCacheEntry) {
        self.entries[0] = Some(entry);
        self.entry_count = 1;
        self.cache_state = InlineCacheState::Monomorphic;
        self.refresh_monomorphic_own_data_handler();
    }

    #[inline]
    const fn promote_to_megamorphic(&mut self) {
        self.cache_state = InlineCacheState::Megamorphic;
        self.entry_count = 0;
        self.entries = [None; POLYMORPHIC_PROPERTY_CACHE_LIMIT];
        self.refresh_monomorphic_own_data_handler();
    }

    #[inline]
    fn find_entry_index(&self, receiver_shape: ShapeId) -> Option<usize> {
        self.search_entry_index(receiver_shape).ok()
    }

    #[inline]
    fn search_entry_index(&self, receiver_shape: ShapeId) -> Result<usize, usize> {
        self.entries[..usize::from(self.entry_count)].binary_search_by(|entry| {
            let Some(entry) = *entry else {
                return Ordering::Greater;
            };
            entry.receiver_shape().cmp(&receiver_shape)
        })
    }

    #[inline]
    fn insert_entry_at(&mut self, index: usize, entry: NamedPropertyCacheEntry) {
        let count = usize::from(self.entry_count);
        if count >= POLYMORPHIC_PROPERTY_CACHE_LIMIT {
            self.promote_to_megamorphic();
            return;
        }
        if index < count {
            self.entries.copy_within(index..count, index + 1);
        }
        self.entries[index] = Some(entry);
        self.entry_count = self.entry_count.saturating_add(1);
        self.cache_state = InlineCacheState::Polymorphic;
        // `refresh_monomorphic_own_data_handler` clears the monomorphic words and
        // repopulates the polymorphic sidecar from the just-grown `entries`.
        self.refresh_monomorphic_own_data_handler();
    }
}

impl KeyedPropertyFeedback {
    #[inline]
    const fn new() -> Self {
        Self {
            execution_count: 0,
            family: None,
            cache_state: InlineCacheState::Uninitialized,
            named_entry_count: 0,
            named_entries: [None; POLYMORPHIC_PROPERTY_CACHE_LIMIT],
            dense_entry_count: 0,
            dense_entries: [None; POLYMORPHIC_PROPERTY_CACHE_LIMIT],
            monomorphic_named_own_data_handler: NamedPropertyHandler::NONE,
            monomorphic_named_atom: 0,
            monomorphic_named_proto_data_handler: NamedPropertyProtoHandler::NONE,
            monomorphic_dense_index_handler: KeyedDenseIndexHandler::NONE,
            polymorphic_named_own_data_handlers: [NamedPropertyHandler::NONE; POLY_LIMIT],
            polymorphic_named_atoms: [0; POLY_LIMIT],
            polymorphic_dense_index_handlers: [KeyedDenseIndexHandler::NONE; POLY_LIMIT],
        }
    }

    /// Recompute every keyed cache-hit sidecar (`monomorphic_named_own_data_handler`,
    /// `monomorphic_named_proto_data_handler`, `monomorphic_dense_index_handler`,
    /// `polymorphic_named_own_data_handlers`, `polymorphic_dense_index_handlers`, plus paired
    /// atom / epoch snapshots) from the current cache state. Called from
    /// every mutation that may change an active entry, the `cache_state`,
    /// or the family. Mirrors
    /// [`NamedPropertyFeedback::refresh_monomorphic_own_data_handler`] but covers both
    /// the named-atom and dense-index families.
    #[inline]
    fn refresh_monomorphic_own_data_handler(&mut self) {
        self.monomorphic_named_own_data_handler = NamedPropertyHandler::NONE;
        self.monomorphic_named_atom = 0;
        self.monomorphic_named_proto_data_handler = NamedPropertyProtoHandler::NONE;
        self.monomorphic_dense_index_handler = KeyedDenseIndexHandler::NONE;
        for slot in 0..POLY_LIMIT {
            self.polymorphic_named_own_data_handlers[slot] = NamedPropertyHandler::NONE;
            self.polymorphic_named_atoms[slot] = 0;
            self.polymorphic_dense_index_handlers[slot] = KeyedDenseIndexHandler::NONE;
        }
        match (self.cache_state, self.family) {
            (InlineCacheState::Monomorphic, Some(KeyedPropertyFamily::NamedAtom)) => {
                if let Some(keyed_entry) = self.named_entries[0] {
                    Self::populate_named_atom_monomorphic(
                        keyed_entry,
                        &mut self.monomorphic_named_own_data_handler,
                        &mut self.monomorphic_named_atom,
                        &mut self.monomorphic_named_proto_data_handler,
                    );
                }
            }
            (InlineCacheState::Monomorphic, Some(KeyedPropertyFamily::DenseIndex)) => {
                if let Some(dense) = self.dense_entries[0] {
                    self.monomorphic_dense_index_handler =
                        KeyedDenseIndexHandler::new(dense.receiver_shape, dense.receiver_flags);
                }
            }
            (InlineCacheState::Polymorphic, Some(KeyedPropertyFamily::NamedAtom)) => {
                let active = usize::from(self.named_entry_count).min(POLY_LIMIT);
                for slot in 0..active {
                    let Some(keyed_entry) = self.named_entries[slot] else {
                        continue;
                    };
                    if !matches!(keyed_entry.entry.path(), NamedPropertyCachePath::OwnData) {
                        continue;
                    }
                    let handler = NamedPropertyHandler::from_entry(keyed_entry.entry);
                    if handler.is_valid() {
                        self.polymorphic_named_own_data_handlers[slot] = handler;
                        self.polymorphic_named_atoms[slot] = keyed_entry.atom.raw();
                    }
                }
            }
            (InlineCacheState::Polymorphic, Some(KeyedPropertyFamily::DenseIndex)) => {
                let active = usize::from(self.dense_entry_count).min(POLY_LIMIT);
                for slot in 0..active {
                    if let Some(dense) = self.dense_entries[slot] {
                        self.polymorphic_dense_index_handlers[slot] =
                            KeyedDenseIndexHandler::new(dense.receiver_shape, dense.receiver_flags);
                    }
                }
            }
            _ => {}
        }
    }

    /// Helper that packs the monomorphic-NamedAtom path into the supplied
    /// sidecar fields. Extracted so `refresh_monomorphic_own_data_handler` can share
    /// the OwnData/PrototypeData branching with the polymorphic walk.
    #[inline]
    fn populate_named_atom_monomorphic(
        keyed_entry: KeyedNamedPropertyCacheEntry,
        mono_handler: &mut NamedPropertyHandler,
        mono_atom: &mut u32,
        proto_handler: &mut NamedPropertyProtoHandler,
    ) {
        match keyed_entry.entry.path() {
            NamedPropertyCachePath::OwnData => {
                let handler = NamedPropertyHandler::from_entry(keyed_entry.entry);
                if handler.is_valid() {
                    *mono_handler = handler;
                    *mono_atom = keyed_entry.atom.raw();
                }
            }
            NamedPropertyCachePath::OwnDataTransition => {}
            NamedPropertyCachePath::PrototypeData => {
                let handler = NamedPropertyProtoHandler::from_entry(keyed_entry.entry);
                if handler.is_valid() {
                    *proto_handler = handler;
                    *mono_atom = keyed_entry.atom.raw();
                }
            }
        }
    }

    #[inline]
    fn try_load(&self, agent: &Agent, receiver: ObjectRef, atom: AtomId) -> Option<Value> {
        if self.family != Some(KeyedPropertyFamily::NamedAtom) {
            return None;
        }
        match self.cache_state {
            InlineCacheState::Monomorphic | InlineCacheState::Polymorphic => {}
            InlineCacheState::Uninitialized | InlineCacheState::Megamorphic => return None,
        }
        let receiver_shape = agent
            .objects()
            .object_header(agent.heap().view(), receiver)?
            .shape();
        let entry = self.named_entries[self.find_named_entry_index(atom, receiver_shape)?]?;
        if let Ok(Some(value)) = agent.objects().load_from_named_property_cache(
            agent.heap().view(),
            receiver,
            entry.entry,
        ) {
            return Some(value);
        }
        None
    }

    #[inline]
    fn try_store(
        &self,
        agent: &mut Agent,
        receiver: ObjectRef,
        atom: AtomId,
        value: Value,
    ) -> Option<bool> {
        if self.family != Some(KeyedPropertyFamily::NamedAtom) {
            return None;
        }
        match self.cache_state {
            InlineCacheState::Monomorphic | InlineCacheState::Polymorphic => {}
            InlineCacheState::Uninitialized | InlineCacheState::Megamorphic => return None,
        }
        let receiver_shape = agent
            .objects()
            .object_header(agent.heap().view(), receiver)?
            .shape();
        let entry = self.named_entries[self.find_named_entry_index(atom, receiver_shape)?]?;
        let result = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.store_to_named_property_cache(
                &mut mutator,
                receiver,
                PropertyKey::from_atom(atom),
                entry.entry,
                value,
            )
        });
        if let Ok(Some(stored)) = result {
            return Some(stored);
        }
        None
    }

    #[inline]
    fn try_dense_index_load(
        &self,
        agent: &Agent,
        receiver: ObjectRef,
        index: u32,
    ) -> Option<Value> {
        let header = self.match_dense_index_header(agent, receiver)?;
        Self::dense_value_from_header(agent, header, index)
    }

    #[inline]
    fn try_dense_index_store(
        &self,
        agent: &mut Agent,
        receiver: ObjectRef,
        index: u32,
        value: Value,
    ) -> Option<bool> {
        if value == Value::array_hole() {
            return None;
        }
        let header = self.match_dense_index_header(agent, receiver)?;
        let elements = header.elements()?;
        let index_usize = usize::try_from(index).expect("u32 index should fit into usize");
        let current = agent
            .heap()
            .view()
            .object_slots(elements.raw())?
            .get(index_usize)
            .copied()
            .unwrap_or(Value::array_hole());
        if current == Value::array_hole() {
            return None;
        }
        let stored = agent.with_heap_and_objects(|heap, _objects| {
            let mut mutator = heap.mutator();
            mutator.mut_store_value(ValueStoreTarget::ObjectSlot(elements.raw(), index), value)
        });
        stored.then_some(true)
    }

    #[inline]
    fn observe_named_atom_slow_path(
        &mut self,
        atom: AtomId,
        plan: Option<NamedPropertyCacheEntry>,
    ) {
        let Some(plan) = plan else {
            self.promote_to_megamorphic(Some(KeyedPropertyFamily::NamedAtom));
            return;
        };
        match self.family {
            None => {
                self.family = Some(KeyedPropertyFamily::NamedAtom);
                self.named_entries[0] = Some(KeyedNamedPropertyCacheEntry { atom, entry: plan });
                self.named_entry_count = 1;
                self.cache_state = InlineCacheState::Monomorphic;
            }
            Some(KeyedPropertyFamily::NamedAtom) => match self.cache_state {
                InlineCacheState::Megamorphic => {}
                InlineCacheState::Uninitialized => {
                    self.named_entries[0] =
                        Some(KeyedNamedPropertyCacheEntry { atom, entry: plan });
                    self.named_entry_count = 1;
                    self.cache_state = InlineCacheState::Monomorphic;
                }
                InlineCacheState::Monomorphic | InlineCacheState::Polymorphic => {
                    let entry = KeyedNamedPropertyCacheEntry { atom, entry: plan };
                    match self.search_named_entry_index(atom, plan.receiver_shape()) {
                        Ok(index) => self.named_entries[index] = Some(entry),
                        Err(index) => self.insert_named_entry_at(index, entry),
                    }
                }
            },
            Some(KeyedPropertyFamily::DenseIndex | KeyedPropertyFamily::Generic) => {
                self.promote_to_megamorphic(Some(KeyedPropertyFamily::Generic));
            }
        }
        self.refresh_monomorphic_own_data_handler();
    }

    #[inline]
    fn observe_dense_index(&mut self, plan: Option<DenseIndexCacheEntry>) -> bool {
        let Some(plan) = plan else {
            return self.observe_uncacheable_dense_index();
        };
        let changed = match self.family {
            None | Some(KeyedPropertyFamily::DenseIndex) => {
                if self.family.is_none() {
                    self.install_first_dense_entry(plan);
                    true
                } else {
                    match self.cache_state {
                        InlineCacheState::Megamorphic => false,
                        InlineCacheState::Uninitialized => {
                            self.install_first_dense_entry(plan);
                            true
                        }
                        InlineCacheState::Monomorphic | InlineCacheState::Polymorphic => {
                            if let Some(index) = self.find_dense_entry_index(plan) {
                                let changed = self.dense_entries[index] != Some(plan);
                                self.dense_entries[index] = Some(plan);
                                changed
                            } else if usize::from(self.dense_entry_count)
                                >= POLYMORPHIC_PROPERTY_CACHE_LIMIT
                            {
                                self.promote_to_megamorphic(Some(KeyedPropertyFamily::DenseIndex));
                                true
                            } else {
                                self.dense_entries[usize::from(self.dense_entry_count)] =
                                    Some(plan);
                                self.dense_entry_count = self.dense_entry_count.saturating_add(1);
                                self.cache_state = if self.dense_entry_count <= 1 {
                                    InlineCacheState::Monomorphic
                                } else {
                                    InlineCacheState::Polymorphic
                                };
                                true
                            }
                        }
                    }
                }
            }
            Some(KeyedPropertyFamily::NamedAtom | KeyedPropertyFamily::Generic) => {
                self.promote_mixed_keyed_family_to_generic()
            }
        };
        self.refresh_monomorphic_own_data_handler();
        changed
    }

    #[inline]
    fn observe_uncacheable_dense_index(&mut self) -> bool {
        match self.family {
            None | Some(KeyedPropertyFamily::DenseIndex) => {
                if self.family == Some(KeyedPropertyFamily::DenseIndex)
                    && self.cache_state == InlineCacheState::Megamorphic
                    && self.dense_entry_count == 0
                {
                    return false;
                }
                self.promote_to_megamorphic(Some(KeyedPropertyFamily::DenseIndex));
                true
            }
            Some(KeyedPropertyFamily::NamedAtom | KeyedPropertyFamily::Generic) => {
                self.promote_mixed_keyed_family_to_generic()
            }
        }
    }

    #[inline]
    fn promote_mixed_keyed_family_to_generic(&mut self) -> bool {
        if self.family == Some(KeyedPropertyFamily::Generic)
            && self.cache_state == InlineCacheState::Megamorphic
            && self.named_entry_count == 0
            && self.dense_entry_count == 0
        {
            return false;
        }
        self.promote_to_megamorphic(Some(KeyedPropertyFamily::Generic));
        true
    }

    #[inline]
    const fn observe_generic(&mut self) {
        self.promote_to_megamorphic(Some(KeyedPropertyFamily::Generic));
    }

    #[inline]
    fn dense_index_plan(
        agent: &Agent,
        receiver: ObjectRef,
        index: u32,
    ) -> Option<DenseIndexCacheEntry> {
        let header = agent
            .objects()
            .object_header(agent.heap().view(), receiver)?;
        if !Self::dense_index_receiver_is_cacheable(agent, receiver, header) {
            return None;
        }
        Self::dense_value_from_header(agent, header, index)?;
        Some(DenseIndexCacheEntry::from_header(header))
    }

    #[inline]
    fn dense_index_receiver_is_cacheable(
        agent: &Agent,
        receiver: ObjectRef,
        header: ObjectHeader,
    ) -> bool {
        matches!(header.kind(), ObjectKind::Ordinary | ObjectKind::Function)
            && !header.flags().is_arguments_object()
            && !agent.objects().is_module_namespace_object(receiver)
            && !agent.objects().is_typed_array_object(receiver)
            && agent.objects().primitive_wrapper_kind(receiver)
                != Some(PrimitiveWrapperKind::String)
    }

    #[inline]
    fn dense_value_from_header(agent: &Agent, header: ObjectHeader, index: u32) -> Option<Value> {
        let elements = header.elements()?;
        let index = usize::try_from(index).expect("u32 index should fit into usize");
        let value = agent
            .heap()
            .view()
            .object_slots(elements.raw())?
            .get(index)
            .copied()
            .unwrap_or(Value::array_hole());
        (value != Value::array_hole()).then_some(value)
    }

    #[inline]
    fn match_dense_index_header(&self, agent: &Agent, receiver: ObjectRef) -> Option<ObjectHeader> {
        if self.family != Some(KeyedPropertyFamily::DenseIndex) {
            return None;
        }
        match self.cache_state {
            InlineCacheState::Monomorphic | InlineCacheState::Polymorphic => {}
            InlineCacheState::Uninitialized | InlineCacheState::Megamorphic => return None,
        }
        let header = agent
            .objects()
            .object_header(agent.heap().view(), receiver)?;
        self.active_dense_entries()
            .any(|entry| entry.matches_header(header))
            .then_some(header)
    }

    #[inline]
    fn active_named_entries(&self) -> impl Iterator<Item = KeyedNamedPropertyCacheEntry> + '_ {
        self.named_entries
            .iter()
            .take(usize::from(self.named_entry_count))
            .filter_map(|entry| *entry)
    }

    #[inline]
    fn active_dense_entries(&self) -> impl Iterator<Item = DenseIndexCacheEntry> + '_ {
        self.dense_entries
            .iter()
            .take(usize::from(self.dense_entry_count))
            .filter_map(|entry| *entry)
    }

    #[inline]
    const fn install_first_dense_entry(&mut self, entry: DenseIndexCacheEntry) {
        self.family = Some(KeyedPropertyFamily::DenseIndex);
        self.dense_entries[0] = Some(entry);
        self.dense_entry_count = 1;
        self.cache_state = InlineCacheState::Monomorphic;
    }

    #[inline]
    fn find_named_entry_index(&self, atom: AtomId, receiver_shape: ShapeId) -> Option<usize> {
        self.search_named_entry_index(atom, receiver_shape).ok()
    }

    #[inline]
    fn search_named_entry_index(
        &self,
        atom: AtomId,
        receiver_shape: ShapeId,
    ) -> Result<usize, usize> {
        self.named_entries[..usize::from(self.named_entry_count)].binary_search_by(|entry| {
            let Some(entry) = *entry else {
                return Ordering::Greater;
            };
            (entry.atom, entry.entry.receiver_shape()).cmp(&(atom, receiver_shape))
        })
    }

    #[inline]
    fn insert_named_entry_at(&mut self, index: usize, entry: KeyedNamedPropertyCacheEntry) {
        let count = usize::from(self.named_entry_count);
        if count >= POLYMORPHIC_PROPERTY_CACHE_LIMIT {
            self.promote_to_megamorphic(Some(KeyedPropertyFamily::NamedAtom));
            return;
        }
        if index < count {
            self.named_entries.copy_within(index..count, index + 1);
        }
        self.named_entries[index] = Some(entry);
        self.named_entry_count = self.named_entry_count.saturating_add(1);
        self.cache_state = InlineCacheState::Polymorphic;
    }

    #[inline]
    fn find_dense_entry_index(&self, plan: DenseIndexCacheEntry) -> Option<usize> {
        self.active_dense_entries()
            .enumerate()
            .find_map(|(index, entry)| (entry == plan).then_some(index))
    }

    #[inline]
    const fn promote_to_megamorphic(&mut self, family: Option<KeyedPropertyFamily>) {
        self.family = family;
        self.cache_state = InlineCacheState::Megamorphic;
        self.named_entry_count = 0;
        self.named_entries = [None; POLYMORPHIC_PROPERTY_CACHE_LIMIT];
        self.dense_entry_count = 0;
        self.dense_entries = [None; POLYMORPHIC_PROPERTY_CACHE_LIMIT];
        self.monomorphic_named_own_data_handler = NamedPropertyHandler::NONE;
        self.monomorphic_named_atom = 0;
        self.monomorphic_named_proto_data_handler = NamedPropertyProtoHandler::NONE;
        self.monomorphic_dense_index_handler = KeyedDenseIndexHandler::NONE;
        self.polymorphic_named_own_data_handlers = [NamedPropertyHandler::NONE; POLY_LIMIT];
        self.polymorphic_named_atoms = [0; POLY_LIMIT];
        self.polymorphic_dense_index_handlers = [KeyedDenseIndexHandler::NONE; POLY_LIMIT];
    }
}

impl CallFeedback {
    #[inline]
    const fn new(expected_arity: Option<u16>) -> Self {
        Self {
            execution_count: 0,
            expected_arity,
            cache_state: InlineCacheState::Uninitialized,
            entry_count: 0,
            entries: [None; POLYMORPHIC_CALL_CACHE_LIMIT],
        }
    }

    #[inline]
    fn observe_target(&mut self, agent: &Agent, callee: ObjectRef) {
        match self.cache_state {
            InlineCacheState::Megamorphic => {}
            InlineCacheState::Uninitialized => {
                let Some(entry) = CallCacheEntry::from_callee(agent, callee) else {
                    self.promote_to_megamorphic();
                    return;
                };
                self.install_first_entry(entry);
            }
            InlineCacheState::Monomorphic => {
                if self.entries[0].is_some_and(|entry| entry.callee == callee) {
                    return;
                }
                let Some(entry) = CallCacheEntry::from_callee(agent, callee) else {
                    self.promote_to_megamorphic();
                    return;
                };
                self.entries[usize::from(self.entry_count)] = Some(entry);
                self.entry_count = self.entry_count.saturating_add(1);
                self.cache_state = InlineCacheState::Polymorphic;
            }
            InlineCacheState::Polymorphic => {
                for index in 0..usize::from(self.entry_count) {
                    if self.entries[index].is_some_and(|entry| entry.callee == callee) {
                        return;
                    }
                }
                if usize::from(self.entry_count) >= POLYMORPHIC_CALL_CACHE_LIMIT {
                    self.promote_to_megamorphic();
                    return;
                }
                let Some(entry) = CallCacheEntry::from_callee(agent, callee) else {
                    self.promote_to_megamorphic();
                    return;
                };
                self.entries[usize::from(self.entry_count)] = Some(entry);
                self.entry_count = self.entry_count.saturating_add(1);
            }
        }
    }

    #[inline]
    fn active_entries(&self) -> impl Iterator<Item = CallCacheEntry> + '_ {
        self.entries
            .iter()
            .take(usize::from(self.entry_count))
            .filter_map(|entry| *entry)
    }

    #[inline]
    fn frame_safe_builtin_target(&self, callee: ObjectRef) -> Option<BuiltinFunctionId> {
        if self.cache_state != InlineCacheState::Monomorphic {
            return None;
        }
        let entry = self.entries[0]?;
        if entry.callee != callee {
            return None;
        }
        entry
            .builtin
            .filter(|builtin| call_feedback_builtin_is_frame_safe(*builtin))
    }

    #[inline]
    const fn install_first_entry(&mut self, entry: CallCacheEntry) {
        self.entries[0] = Some(entry);
        self.entry_count = 1;
        self.cache_state = InlineCacheState::Monomorphic;
    }

    #[inline]
    const fn promote_to_megamorphic(&mut self) {
        self.cache_state = InlineCacheState::Megamorphic;
        self.entry_count = 0;
        self.entries = [None; POLYMORPHIC_CALL_CACHE_LIMIT];
    }
}

impl ConstructFeedback {
    #[inline]
    const fn new(expected_arity: Option<u16>) -> Self {
        Self {
            execution_count: 0,
            expected_arity,
            cache_state: InlineCacheState::Uninitialized,
            entry_count: 0,
            entries: [None; POLYMORPHIC_CALL_CACHE_LIMIT],
        }
    }

    #[inline]
    fn observe_target(
        &mut self,
        agent: &Agent,
        constructor: ObjectRef,
        created: Option<ObjectRef>,
    ) {
        match self.cache_state {
            InlineCacheState::Megamorphic => {}
            InlineCacheState::Uninitialized => {
                let Some(entry) =
                    ConstructCacheEntry::from_constructor(agent, constructor, created)
                else {
                    self.promote_to_megamorphic();
                    return;
                };
                self.install_first_entry(entry);
            }
            InlineCacheState::Monomorphic => {
                if self.refresh_matching_entry_created_shape(agent, 0, constructor, created) {
                    return;
                }
                let Some(entry) =
                    ConstructCacheEntry::from_constructor(agent, constructor, created)
                else {
                    self.promote_to_megamorphic();
                    return;
                };
                self.entries[usize::from(self.entry_count)] = Some(entry);
                self.entry_count = self.entry_count.saturating_add(1);
                self.cache_state = InlineCacheState::Polymorphic;
            }
            InlineCacheState::Polymorphic => {
                for index in 0..usize::from(self.entry_count) {
                    if self.refresh_matching_entry_created_shape(agent, index, constructor, created)
                    {
                        return;
                    }
                }
                if usize::from(self.entry_count) >= POLYMORPHIC_CALL_CACHE_LIMIT {
                    self.promote_to_megamorphic();
                    return;
                }
                let Some(entry) =
                    ConstructCacheEntry::from_constructor(agent, constructor, created)
                else {
                    self.promote_to_megamorphic();
                    return;
                };
                self.entries[usize::from(self.entry_count)] = Some(entry);
                self.entry_count = self.entry_count.saturating_add(1);
            }
        }
    }

    #[inline]
    fn refresh_matching_entry_created_shape(
        &mut self,
        agent: &Agent,
        index: usize,
        constructor: ObjectRef,
        created: Option<ObjectRef>,
    ) -> bool {
        let Some(mut entry) = self.entries[index] else {
            return false;
        };
        if entry.constructor != constructor {
            return false;
        }
        if entry.created_shape.is_none() {
            entry.created_shape = ConstructCacheEntry::created_shape(agent, created);
            self.entries[index] = Some(entry);
        }
        true
    }

    #[inline]
    fn active_entries(&self) -> impl Iterator<Item = ConstructCacheEntry> + '_ {
        self.entries
            .iter()
            .take(usize::from(self.entry_count))
            .filter_map(|entry| *entry)
    }

    #[inline]
    const fn install_first_entry(&mut self, entry: ConstructCacheEntry) {
        self.entries[0] = Some(entry);
        self.entry_count = 1;
        self.cache_state = InlineCacheState::Monomorphic;
    }

    #[inline]
    const fn promote_to_megamorphic(&mut self) {
        self.cache_state = InlineCacheState::Megamorphic;
        self.entry_count = 0;
        self.entries = [None; POLYMORPHIC_CALL_CACHE_LIMIT];
    }
}

impl FeedbackSiteState {
    #[inline]
    const fn for_descriptor(descriptor: FeedbackSiteDescriptor) -> Self {
        match descriptor.kind() {
            FeedbackSiteKind::Arithmetic => {
                Self::Arithmetic(ArithmeticFeedback { execution_count: 0 })
            }
            FeedbackSiteKind::Comparison => {
                Self::Comparison(ComparisonFeedback { execution_count: 0 })
            }
            FeedbackSiteKind::NamedPropertyLoad | FeedbackSiteKind::NamedPropertyStore => {
                Self::NamedProperty(NamedPropertyFeedback::new())
            }
            FeedbackSiteKind::KeyedPropertyAccess => {
                Self::KeyedProperty(KeyedPropertyFeedback::new())
            }
            FeedbackSiteKind::Call => {
                Self::Call(CallFeedback::new(descriptor.metadata().expected_arity()))
            }
            FeedbackSiteKind::Construct => Self::Construct(ConstructFeedback::new(
                descriptor.metadata().expected_arity(),
            )),
        }
    }

    #[inline]
    const fn record_execution(&mut self) {
        self.record_execution_count(1);
    }

    #[inline]
    const fn record_execution_count(&mut self, count: u32) {
        match self {
            Self::Arithmetic(feedback) => {
                feedback.execution_count = feedback.execution_count.saturating_add(count);
            }
            Self::Comparison(feedback) => {
                feedback.execution_count = feedback.execution_count.saturating_add(count);
            }
            Self::NamedProperty(feedback) => {
                feedback.execution_count = feedback.execution_count.saturating_add(count);
            }
            Self::KeyedProperty(feedback) => {
                feedback.execution_count = feedback.execution_count.saturating_add(count);
            }
            Self::Call(feedback) => {
                feedback.execution_count = feedback.execution_count.saturating_add(count);
            }
            Self::Construct(feedback) => {
                feedback.execution_count = feedback.execution_count.saturating_add(count);
            }
        }
    }

    #[inline]
    fn record_call_target(&mut self, agent: &Agent, callee: ObjectRef) {
        match self {
            Self::Call(feedback) => {
                feedback.execution_count = feedback.execution_count.saturating_add(1);
                feedback.observe_target(agent, callee);
            }
            _ => self.record_execution(),
        }
    }

    #[inline]
    fn record_construct_target(
        &mut self,
        agent: &Agent,
        constructor: ObjectRef,
        created: Option<ObjectRef>,
    ) {
        match self {
            Self::Construct(feedback) => {
                feedback.execution_count = feedback.execution_count.saturating_add(1);
                feedback.observe_target(agent, constructor, created);
            }
            _ => self.record_execution(),
        }
    }

    #[inline]
    fn snapshot(&self, descriptor: FeedbackSiteDescriptor) -> FeedbackSiteSnapshot {
        match self {
            Self::Arithmetic(feedback) => FeedbackSiteSnapshot::new(
                descriptor,
                feedback.execution_count,
                FeedbackSiteDetail::Arithmetic,
            ),
            Self::Comparison(feedback) => FeedbackSiteSnapshot::new(
                descriptor,
                feedback.execution_count,
                FeedbackSiteDetail::Comparison,
            ),
            Self::NamedProperty(feedback) => FeedbackSiteSnapshot::new(
                descriptor,
                feedback.execution_count,
                FeedbackSiteDetail::NamedProperty(NamedPropertyFeedbackSnapshot::from_feedback(
                    feedback,
                )),
            ),
            Self::KeyedProperty(feedback) => FeedbackSiteSnapshot::new(
                descriptor,
                feedback.execution_count,
                FeedbackSiteDetail::KeyedProperty(KeyedPropertyFeedbackSnapshot::from_feedback(
                    feedback,
                )),
            ),
            Self::Call(feedback) => FeedbackSiteSnapshot::new(
                descriptor,
                feedback.execution_count,
                FeedbackSiteDetail::Call(CallFeedbackSnapshot::from_feedback(feedback)),
            ),
            Self::Construct(feedback) => FeedbackSiteSnapshot::new(
                descriptor,
                feedback.execution_count,
                FeedbackSiteDetail::Construct(ConstructFeedbackSnapshot::from_feedback(feedback)),
            ),
        }
    }

    #[inline]
    const fn unallocated_snapshot(descriptor: FeedbackSiteDescriptor) -> FeedbackSiteSnapshot {
        let detail = match descriptor.kind() {
            FeedbackSiteKind::Arithmetic => FeedbackSiteDetail::Arithmetic,
            FeedbackSiteKind::Comparison => FeedbackSiteDetail::Comparison,
            FeedbackSiteKind::NamedPropertyLoad | FeedbackSiteKind::NamedPropertyStore => {
                FeedbackSiteDetail::NamedProperty(NamedPropertyFeedbackSnapshot::uninitialized(0))
            }
            FeedbackSiteKind::KeyedPropertyAccess => {
                FeedbackSiteDetail::KeyedProperty(KeyedPropertyFeedbackSnapshot::uninitialized(0))
            }
            FeedbackSiteKind::Call => FeedbackSiteDetail::Call(
                CallFeedbackSnapshot::uninitialized(descriptor.metadata().expected_arity(), 0),
            ),
            FeedbackSiteKind::Construct => FeedbackSiteDetail::Construct(
                ConstructFeedbackSnapshot::uninitialized(descriptor.metadata().expected_arity(), 0),
            ),
        };
        FeedbackSiteSnapshot::new(descriptor, 0, detail)
    }

    #[cfg(test)]
    #[inline]
    const fn execution_count(&self) -> u32 {
        match self {
            Self::Arithmetic(feedback) => feedback.execution_count,
            Self::Comparison(feedback) => feedback.execution_count,
            Self::NamedProperty(feedback) => feedback.execution_count,
            Self::KeyedProperty(feedback) => feedback.execution_count,
            Self::Call(feedback) => feedback.execution_count,
            Self::Construct(feedback) => feedback.execution_count,
        }
    }
}

/// Per-code-object feedback storage.
///
/// Stored contiguously in `Vm::feedback_vectors: Vec<FeedbackVector>` (one entry per installed
/// code object). The default-constructed value is an "unallocated" sentinel — empty `sites` —
/// so the hot path can dispatch through `Vec` indexing with no `Option` discrimination. Once
/// the warmup counter on `TieringState` crosses `FEEDBACK_ALLOCATION_THRESHOLD`,
/// [`allocate_sites`](Self::allocate_sites) populates the slot storage in place;
/// `is_allocated()` flips to `true` from then on. The warmup counter itself lives on
/// `TieringState` (see `Tiering::bump_warmup` / `Tiering::warmup_counter`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct FeedbackVector {
    sites: Vec<Option<FeedbackSiteState>>,
}

impl FeedbackVector {
    /// Populate slot storage on a previously-empty vector. The warmup counter is preserved
    /// so [`feedback_vector_footprint`](Vm::feedback_vector_footprint) keeps reporting it
    /// after the cold-to-warm transition.
    #[inline]
    fn allocate_sites(&mut self, slot_descriptors: &[Option<FeedbackSiteDescriptor>]) {
        debug_assert!(
            !self.is_allocated(),
            "allocate_sites should only run on an unallocated FeedbackVector"
        );
        self.sites = slot_descriptors
            .iter()
            .copied()
            .map(|descriptor| descriptor.map(FeedbackSiteState::for_descriptor))
            .collect();
    }

    /// Returns `true` once `sites` has been populated. Stays `true` for the lifetime of the
    /// vector (sites are never cleared once allocated).
    #[inline]
    const fn is_allocated(&self) -> bool {
        !self.sites.is_empty()
    }

    #[inline]
    const fn sites_capacity_bytes(&self) -> usize {
        self.sites.len() * size_of::<Option<FeedbackSiteState>>()
    }

    #[inline]
    fn site_mut(&mut self, slot: FeedbackSlotId) -> Option<&mut FeedbackSiteState> {
        self.sites
            .get_mut(usize::try_from(slot.get().saturating_sub(1)).ok()?)
            .and_then(Option::as_mut)
    }

    #[inline]
    fn site(&self, slot: FeedbackSlotId) -> Option<&FeedbackSiteState> {
        self.sites
            .get(usize::try_from(slot.get().saturating_sub(1)).ok()?)
            .and_then(Option::as_ref)
    }

    /// Returns the per-slot install generation. Returns `0` if the slot is
    /// absent or is not a `NamedProperty` site (other site kinds do not carry
    /// generations in Phase A).
    #[inline]
    pub(super) fn generation(&self, slot: FeedbackSlotId) -> u32 {
        let Some(index) = usize::try_from(slot.get().saturating_sub(1)).ok() else {
            return 0;
        };
        match self.sites.get(index).and_then(Option::as_ref) {
            Some(FeedbackSiteState::NamedProperty(named)) => named.generation,
            _ => 0,
        }
    }

    /// Bumps the per-slot install generation (wrapping add per §8.2) and
    /// returns the new value. Returns `0` if the slot is absent or is not a
    /// `NamedProperty` site.
    /// Called from the slow-path install (via `Vm::bump_generation_for_install`)
    /// to mint the generation before registering `AdaptiveProtoLoad`
    /// watchpoints.
    #[inline]
    pub(super) fn bump_generation(&mut self, slot: FeedbackSlotId) -> u32 {
        let Some(index) = usize::try_from(slot.get().saturating_sub(1)).ok() else {
            return 0;
        };
        match self.sites.get_mut(index).and_then(Option::as_mut) {
            Some(FeedbackSiteState::NamedProperty(named)) => {
                named.generation = named.generation.wrapping_add(1);
                named.generation
            }
            _ => 0,
        }
    }

    /// Clears the IC slot at `slot` by setting its `Option<FeedbackSiteState>`
    /// to `None` (the "uninitialized" representation for this vector).
    /// Called from `Vm::clear_ic_slot_if_generation_matches` after a
    /// generation match confirms the watchpoint is not stale.
    #[inline]
    pub(super) fn clear_site(&mut self, slot: FeedbackSlotId) {
        let Some(index) = usize::try_from(slot.get().saturating_sub(1)).ok() else {
            return;
        };
        if let Some(entry) = self.sites.get_mut(index) {
            *entry = None;
        }
    }

    /// Restores a cleared (None) named-property IC slot to an `Uninitialized`
    /// [`NamedPropertyFeedback`] so the next slow-path call can re-install the
    /// IC entry against the post-mutation shapes. Called from
    /// `record_named_property_cache_entry` when the vector is already allocated
    /// but the slot was cleared by an `AdaptiveProtoLoad` watchpoint fire
    /// (Phase A, Task A.2). No-ops if the slot is already present (not cleared)
    /// or if the vector is not yet allocated.
    #[inline]
    pub(super) fn reinit_named_property_site_if_cleared(&mut self, slot: FeedbackSlotId) {
        let Some(index) = usize::try_from(slot.get().saturating_sub(1)).ok() else {
            return;
        };
        if let Some(entry @ None) = self.sites.get_mut(index) {
            *entry = Some(FeedbackSiteState::NamedProperty(
                NamedPropertyFeedback::new(),
            ));
        }
    }
}

impl Vm {
    #[inline]
    fn ensure_feedback_capacity(&mut self, code: CodeRef) {
        let index = code_index(code);
        if self.feedback_vectors.len() <= index {
            self.feedback_vectors
                .resize_with(index + 1, FeedbackVector::default);
        }
    }

    #[inline]
    fn feedback_site_for_slot(
        &self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) -> Option<&FeedbackSiteState> {
        self.feedback_vectors.get(code_index(code))?.site(slot)
    }

    fn with_feedback_slot_mut<R>(
        &mut self,
        code: CodeRef,
        slot: FeedbackSlotId,
        f: impl FnOnce(&mut FeedbackSiteState) -> R,
    ) -> Option<R> {
        let result = self
            .feedback_vectors
            .get_mut(code_index(code))?
            .site_mut(slot)
            .map(f);
        // DSL-0b (B17): dual-write — after every legacy mutation,
        // mirror the slot into the flat-array storage so the asm
        // `FV` pin sees the compact IC header projected from the
        // semantic source of truth. The projection runs only when the
        // legacy write actually happened (Some(R) path).
        if result.is_some() {
            self.mirror_flat_slot(code, slot);
        }
        result
    }

    #[inline]
    pub(super) fn mirror_flat_slot(&mut self, code: CodeRef, slot: FeedbackSlotId) {
        let index = code_index(code);
        let vector = self.feedback_vectors.get(index);
        let header = vector
            .and_then(|v| v.site(slot))
            .and_then(Self::named_llint_load_header);
        // Read the generation from the semantic source so the flat entry stays
        // in sync on every install/clear cycle.
        let generation = vector.map_or(0, |v| v.generation(slot));
        let Some(slot_index) = Self::flat_feedback_slot_index(slot) else {
            return;
        };
        let Some(entry) = self
            .feedback_flat_storage
            .get_mut(index)
            .and_then(|entries| entries.get_mut(slot_index))
        else {
            return;
        };
        entry.clear_ic_header();
        // Write generation AFTER clear_ic_header so it is never overwritten by
        // the blanket IC-field clear. `clear_ic_header` only touches mode and
        // the named-property handler fields; generation is metadata, not
        // IC state, and is deliberately excluded from that reset.
        entry.generation = generation;
        match header {
            Some(LlIntNamedPropertyHeader::OwnInline { handler_bits }) => {
                entry.set_named_own_inline_load(handler_bits);
            }
            Some(LlIntNamedPropertyHeader::OwnOutline { handler_bits }) => {
                entry.set_named_own_outline_load(handler_bits);
            }
            Some(LlIntNamedPropertyHeader::ProtoInline {
                receiver_word,
                proto_word,
            }) => entry.set_named_proto_inline_load(receiver_word, proto_word),
            Some(LlIntNamedPropertyHeader::OwnPolymorphic {
                slot0_handler_bits,
                slot1_handler_bits,
            }) => entry.set_named_own_polymorphic(slot0_handler_bits, slot1_handler_bits),
            None => {}
        }
    }

    #[inline]
    fn flat_feedback_slot_index(slot: FeedbackSlotId) -> Option<usize> {
        usize::try_from(slot.get().checked_sub(1)?).ok()
    }

    #[inline]
    const fn named_own_inline_load_header(site: &FeedbackSiteState) -> Option<u64> {
        let FeedbackSiteState::NamedProperty(feedback) = site else {
            return None;
        };
        let handler = feedback.monomorphic_own_data_handler;
        if !handler.is_valid() || !matches!(handler.slot_location(), SlotLocation::Inline(_)) {
            return None;
        }
        Some(handler.bits())
    }

    #[inline]
    const fn named_own_outline_load_header(site: &FeedbackSiteState) -> Option<u64> {
        let FeedbackSiteState::NamedProperty(feedback) = site else {
            return None;
        };
        let handler = feedback.monomorphic_own_data_handler;
        if !handler.is_valid() || !matches!(handler.slot_location(), SlotLocation::OutOfLine(_)) {
            return None;
        }
        Some(handler.bits())
    }

    #[inline]
    const fn named_proto_inline_load_header(site: &FeedbackSiteState) -> Option<(u64, u64)> {
        let FeedbackSiteState::NamedProperty(feedback) = site else {
            return None;
        };
        let handler = feedback.monomorphic_proto_data_handler;
        if !handler.is_valid() || !matches!(handler.slot_location(), SlotLocation::Inline(_)) {
            return None;
        }
        Some((handler.receiver_word(), handler.proto_word()))
    }

    /// Phase 3f polymorphic-OwnData header projection. Reports the two
    /// entries of `polymorphic_own_data_handlers` packed into the asm IC
    /// header. Returns `Some` only when both sidecar slots hold valid
    /// `OwnData` handlers *and* both are inline-slot — the first-cut
    /// asm walk in `op_get_named_property_dsl`'s `.try_poly` label
    /// doesn't handle outline polymorphic. Outline / mixed / partially-
    /// filled poly state stays on the slow path until that walk is
    /// extended.
    #[inline]
    const fn named_own_polymorphic_load_header(site: &FeedbackSiteState) -> Option<(u64, u64)> {
        let FeedbackSiteState::NamedProperty(feedback) = site else {
            return None;
        };
        let handler0 = feedback.polymorphic_own_data_handlers[0];
        let handler1 = feedback.polymorphic_own_data_handlers[1];
        if !handler0.is_valid() || !handler1.is_valid() {
            return None;
        }
        if !matches!(handler0.slot_location(), SlotLocation::Inline(_))
            || !matches!(handler1.slot_location(), SlotLocation::Inline(_))
        {
            return None;
        }
        Some((handler0.bits(), handler1.bits()))
    }

    #[inline]
    fn named_llint_load_header(site: &FeedbackSiteState) -> Option<LlIntNamedPropertyHeader> {
        if let Some(handler_bits) = Self::named_own_inline_load_header(site) {
            return Some(LlIntNamedPropertyHeader::OwnInline { handler_bits });
        }
        if let Some(handler_bits) = Self::named_own_outline_load_header(site) {
            return Some(LlIntNamedPropertyHeader::OwnOutline { handler_bits });
        }
        // Polymorphic OwnData takes precedence over ProtoInline — when the
        // IC has transitioned to polymorphic, the monomorphic-proto handler
        // word is NONE, so the order here is monomorphic-Own → polymorphic-
        // Own → monomorphic-Proto for clarity.
        if let Some((slot0_handler_bits, slot1_handler_bits)) =
            Self::named_own_polymorphic_load_header(site)
        {
            return Some(LlIntNamedPropertyHeader::OwnPolymorphic {
                slot0_handler_bits,
                slot1_handler_bits,
            });
        }
        let (receiver_word, proto_word) = Self::named_proto_inline_load_header(site)?;
        Some(LlIntNamedPropertyHeader::ProtoInline {
            receiver_word,
            proto_word,
        })
    }

    fn ensure_feedback_slot_execution(&mut self, code: CodeRef, slot: FeedbackSlotId) -> bool {
        self.ensure_feedback_capacity(code);
        let index = code_index(code);
        let needs_allocation = !self.feedback_vectors[index].is_allocated()
            && self.tiering.warmup_counter(code).saturating_add(1) >= FEEDBACK_ALLOCATION_THRESHOLD;
        let Some(installed) = self.installed.get(index).and_then(Option::as_ref) else {
            return false;
        };
        if installed.feedback_descriptor_for_slot(slot).is_none() {
            return false;
        }
        let slot_descriptors = if needs_allocation {
            Some(installed.feedback_slot_descriptors().to_vec())
        } else {
            None
        };

        if !self.feedback_vectors[index].is_allocated() {
            self.tiering.bump_warmup(code);
            if let Some(slot_descriptors) = slot_descriptors.filter(|slots| !slots.is_empty()) {
                self.feedback_vectors[index].allocate_sites(&slot_descriptors);
            }
        }

        let mirrored = self.feedback_vectors[index]
            .site_mut(slot)
            .is_some_and(|site| {
                site.record_execution();
                true
            });
        if mirrored {
            self.mirror_flat_slot(code, slot);
        }
        self.tiering.observe_feedback_event(code);
        true
    }

    fn record_allocated_feedback_slot(&mut self, code: CodeRef, slot: FeedbackSlotId) -> bool {
        let index = code_index(code);
        let Some(vector) = self.feedback_vectors.get_mut(index) else {
            return false;
        };
        if !vector.is_allocated() {
            return false;
        }
        let Some(site) = vector.site_mut(slot) else {
            return false;
        };
        site.record_execution();
        self.mirror_flat_slot(code, slot);
        self.tiering.observe_feedback_event(code);
        true
    }

    pub(crate) fn record_feedback_slot(&mut self, code: CodeRef, slot: Option<FeedbackSlotId>) {
        let Some(slot) = slot else {
            return;
        };
        if self.record_allocated_feedback_slot(code, slot) {
            return;
        }
        let _ = self.ensure_feedback_slot_execution(code, slot);
    }

    pub(in crate::vm) fn drain_llint_scalar_feedback(&mut self) {
        let mut pending = Vec::new();
        for (code_index, entries) in self.feedback_flat_storage.iter_mut().enumerate() {
            let Some(code_raw) = u32::try_from(code_index)
                .ok()
                .and_then(|index| index.checked_add(1))
            else {
                continue;
            };
            let Some(code) = CodeRef::from_raw(code_raw) else {
                continue;
            };
            for (slot_index, entry) in entries.iter_mut().enumerate() {
                let Some(update) = entry.take_scalar_feedback() else {
                    continue;
                };
                let Some(slot_raw) = u32::try_from(slot_index)
                    .ok()
                    .and_then(|index| index.checked_add(1))
                else {
                    continue;
                };
                if let Some(slot) = FeedbackSlotId::from_raw(slot_raw) {
                    pending.push((code, slot, update));
                }
            }
        }

        for (code, slot, update) in pending {
            self.record_llint_scalar_feedback_update(code, slot, update);
        }
    }

    fn record_llint_scalar_feedback_update(
        &mut self,
        code: CodeRef,
        slot: FeedbackSlotId,
        update: crate::dsl::feedback_flat::ScalarFeedbackUpdate,
    ) {
        if update.execution_count == 0
            || update.observed_bits & crate::dsl::feedback_flat::LLINT_FEEDBACK_OBSERVED_SMI == 0
        {
            return;
        }
        self.record_feedback_slot_batch(code, slot, update.execution_count);
    }

    fn record_feedback_slot_batch(&mut self, code: CodeRef, slot: FeedbackSlotId, count: u32) {
        if count == 0 {
            return;
        }
        self.ensure_feedback_capacity(code);
        let index = code_index(code);
        let Some(installed) = self.installed.get(index).and_then(Option::as_ref) else {
            return;
        };
        if installed.feedback_descriptor_for_slot(slot).is_none() {
            return;
        }
        let slot_descriptors = if self.feedback_vectors[index].is_allocated() {
            None
        } else {
            Some(installed.feedback_slot_descriptors().to_vec())
        };

        let mut recorded_count = count;
        if !self.feedback_vectors[index].is_allocated() {
            // Read warmup counter from tiering before taking the mutable vector borrow.
            let current_warmup = self.tiering.warmup_counter(code);
            let events_until_allocation = u32::from(
                FEEDBACK_ALLOCATION_THRESHOLD
                    .saturating_sub(current_warmup)
                    .max(1),
            );
            let warmup_events = count.min(events_until_allocation);
            let warmup_increment =
                u16::try_from(warmup_events).expect("feedback warmup threshold fits in u16");
            self.tiering.bump_warmup_by(code, warmup_increment);
            if count < events_until_allocation {
                recorded_count = 0;
            } else {
                let vector = &mut self.feedback_vectors[index];
                if let Some(slot_descriptors) = slot_descriptors.filter(|slots| !slots.is_empty()) {
                    vector.allocate_sites(&slot_descriptors);
                }
                recorded_count = count - events_until_allocation + 1;
            }
        }

        let wrote = if recorded_count == 0 {
            false
        } else if let Some(site) = self.feedback_vectors[index].site_mut(slot) {
            site.record_execution_count(recorded_count);
            true
        } else {
            false
        };
        if wrote {
            self.mirror_flat_slot(code, slot);
        }
        self.tiering.observe_feedback_events(code, count);
    }

    #[inline]
    pub(super) fn observe_call_target(
        &mut self,
        agent: &Agent,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
        callee: ObjectRef,
    ) {
        let Some(slot) = slot else {
            return;
        };
        let index = code_index(code);
        if let Some(site) = self
            .feedback_vectors
            .get_mut(index)
            .and_then(|vector| vector.site_mut(slot))
        {
            site.record_call_target(agent, callee);
            // DSL-0b (B17) dual-write — see `mirror_flat_slot`.
            self.mirror_flat_slot(code, slot);
            self.tiering.observe_feedback_event(code);
            return;
        }

        if !self.ensure_feedback_slot_execution(code, slot) {
            return;
        }
        let _ = self.with_feedback_slot_mut(code, slot, |site| {
            if let FeedbackSiteState::Call(feedback) = site {
                feedback.observe_target(agent, callee);
            }
        });
    }

    #[inline]
    pub(super) fn cached_frame_safe_builtin_call_target(
        &self,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
        callee: ObjectRef,
    ) -> Option<BuiltinFunctionId> {
        match self.feedback_site_for_slot(code, slot?)? {
            FeedbackSiteState::Call(feedback) => feedback.frame_safe_builtin_target(callee),
            _ => None,
        }
    }

    #[inline]
    pub(super) fn observe_construct_target(
        &mut self,
        agent: &Agent,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
        constructor: ObjectRef,
        created: Option<ObjectRef>,
    ) {
        let Some(slot) = slot else {
            return;
        };
        let index = code_index(code);
        if let Some(site) = self
            .feedback_vectors
            .get_mut(index)
            .and_then(|vector| vector.site_mut(slot))
        {
            site.record_construct_target(agent, constructor, created);
            // DSL-0b (B17) dual-write — see `mirror_flat_slot`.
            self.mirror_flat_slot(code, slot);
            self.tiering.observe_feedback_event(code);
            return;
        }

        if !self.ensure_feedback_slot_execution(code, slot) {
            return;
        }
        let _ = self.with_feedback_slot_mut(code, slot, |site| {
            if let FeedbackSiteState::Construct(feedback) = site {
                feedback.observe_target(agent, constructor, created);
            }
        });
    }

    /// Read the bit-packed monomorphic `OwnData` IC handler for one feedback
    /// slot. Returns `None` when the slot is absent, the site isn't a
    /// named-property site, or the cache is in any state other than
    /// monomorphic-OwnData. Phase 3 IC cache hit path entry point.
    ///
    /// Spec 2 Phase A: epoch comparisons are no longer needed because
    /// `AdaptiveProtoLoad` watchpoints registered at IC install time fire
    /// on any proto-chain mutation and clear the IC slot before the next
    /// cache-hit read.
    #[inline(always)]
    pub(super) fn named_property_own_data_handler(
        &self,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
    ) -> Option<NamedPropertyHandler> {
        let site = self.feedback_site_for_slot(code, slot?)?;
        match site {
            FeedbackSiteState::NamedProperty(feedback)
                if feedback.monomorphic_own_data_handler.is_valid() =>
            {
                Some(feedback.monomorphic_own_data_handler)
            }
            _ => None,
        }
    }

    /// Read the bit-packed one-hop `PrototypeData` IC handler for one
    /// feedback slot. Returns `None` when the slot is absent, the site
    /// isn't a named-property site, or the cache is in any state other
    /// than monomorphic one-hop `PrototypeData` (`dependency_count == 2`).
    /// Phase 3e IC cache path entry point — mirrors
    /// [`Self::named_property_own_data_handler`] but for the
    /// prototype-method-dispatch hot path.
    #[inline(always)]
    pub(super) fn named_property_proto_data_handler(
        &self,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
    ) -> Option<NamedPropertyProtoHandler> {
        let site = self.feedback_site_for_slot(code, slot?)?;
        match site {
            FeedbackSiteState::NamedProperty(feedback)
                if feedback.monomorphic_proto_data_handler.is_valid() =>
            {
                Some(feedback.monomorphic_proto_data_handler)
            }
            _ => None,
        }
    }

    /// Side-effect helper for the inlined IC cache hit path: increment the
    /// per-site execution counter and emit a tier feedback event. Mirrors
    /// the trailing two lines of [`Self::try_named_property_load_inline_cache_hit`]
    /// so the inline cache hit path stays semantically identical.
    #[inline(always)]
    pub(super) fn record_named_property_cache_hit(&mut self, code: CodeRef, slot: FeedbackSlotId) {
        let wrote = if let Some(vector) = self.feedback_vectors.get_mut(code_index(code))
            && let Some(site) = vector.site_mut(slot)
        {
            site.record_execution();
            true
        } else {
            false
        };
        if wrote {
            self.mirror_flat_slot(code, slot);
        }
        self.tiering.observe_feedback_event(code);
    }

    /// Phase 3d named-keyed cache handler lookup. Returns the packed
    /// `NamedPropertyHandler` only when:
    ///   - the site is a `KeyedProperty` site,
    ///   - cache state is monomorphic + family is `NamedAtom`,
    ///   - the cached atom equals the runtime `atom`.
    #[inline(always)]
    pub(super) fn keyed_property_named_own_data_handler(
        &self,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
        atom: AtomId,
    ) -> Option<NamedPropertyHandler> {
        let site = self.feedback_site_for_slot(code, slot?)?;
        match site {
            FeedbackSiteState::KeyedProperty(feedback)
                if feedback.monomorphic_named_atom == atom.raw()
                    && feedback.monomorphic_named_own_data_handler.is_valid() =>
            {
                Some(feedback.monomorphic_named_own_data_handler)
            }
            _ => None,
        }
    }

    /// Phase 3e named-keyed proto cache handler lookup. Returns the packed
    /// `NamedPropertyProtoHandler` only when:
    ///   - the site is a `KeyedProperty` site,
    ///   - cache state is monomorphic + family is `NamedAtom`,
    ///   - the cached atom equals the runtime `atom`,
    ///   - the cached entry is one-hop `PrototypeData`.
    #[inline(always)]
    pub(super) fn keyed_property_named_proto_data_handler(
        &self,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
        atom: AtomId,
    ) -> Option<NamedPropertyProtoHandler> {
        let site = self.feedback_site_for_slot(code, slot?)?;
        match site {
            FeedbackSiteState::KeyedProperty(feedback)
                if feedback.monomorphic_named_atom == atom.raw()
                    && feedback.monomorphic_named_proto_data_handler.is_valid() =>
            {
                Some(feedback.monomorphic_named_proto_data_handler)
            }
            _ => None,
        }
    }

    /// Phase 3d dense-keyed cache handler lookup. Returns the packed
    /// `KeyedDenseIndexHandler` only when the site is a `KeyedProperty`
    /// site in the monomorphic `DenseIndex` family.
    #[inline(always)]
    pub(super) fn keyed_property_dense_index_handler(
        &self,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
    ) -> Option<KeyedDenseIndexHandler> {
        let site = self.feedback_site_for_slot(code, slot?)?;
        match site {
            FeedbackSiteState::KeyedProperty(feedback)
                if feedback.monomorphic_dense_index_handler.is_valid() =>
            {
                Some(feedback.monomorphic_dense_index_handler)
            }
            _ => None,
        }
    }

    /// Phase 3f polymorphic-OwnData IC lookup. Walks the
    /// `[NamedPropertyHandler; POLY_LIMIT]` sidecar for a shape match,
    /// returning the matching packed handler on hit. Returns `None` when
    /// the slot is absent, the site isn't a named-property site, the
    /// cache isn't polymorphic, or no cached shape matches. Sibling to
    /// [`Self::named_property_own_data_handler`] for shapes
    /// `2..POLY_LIMIT`; the inline call site checks the monomorphic word
    /// first, then walks here on miss.
    #[inline(always)]
    pub(super) fn named_property_polymorphic_own_data_handler(
        &self,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
        receiver_shape: ShapeId,
    ) -> Option<NamedPropertyHandler> {
        let site = self.feedback_site_for_slot(code, slot?)?;
        let FeedbackSiteState::NamedProperty(feedback) = site else {
            return None;
        };
        let target = Some(receiver_shape);
        for slot_index in 0..POLY_LIMIT {
            let handler = feedback.polymorphic_own_data_handlers[slot_index];
            if handler.is_valid() && handler.receiver_shape() == target {
                return Some(handler);
            }
        }
        None
    }

    /// Phase 3f polymorphic-OwnData keyed-named IC cache-hit lookup.
    /// Walks the named-atom polymorphic sidecar matching both the runtime
    /// `atom` and the receiver shape. Sibling to
    /// [`Self::keyed_property_named_own_data_handler`] for shapes `2..POLY_LIMIT`
    /// of a keyed-atom site.
    #[inline(always)]
    pub(super) fn keyed_property_named_polymorphic_own_data_handler(
        &self,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
        atom: AtomId,
        receiver_shape: ShapeId,
    ) -> Option<NamedPropertyHandler> {
        let site = self.feedback_site_for_slot(code, slot?)?;
        let FeedbackSiteState::KeyedProperty(feedback) = site else {
            return None;
        };
        let target = Some(receiver_shape);
        let atom_raw = atom.raw();
        for slot_index in 0..POLY_LIMIT {
            let handler = feedback.polymorphic_named_own_data_handlers[slot_index];
            if handler.is_valid()
                && feedback.polymorphic_named_atoms[slot_index] == atom_raw
                && handler.receiver_shape() == target
            {
                return Some(handler);
            }
        }
        None
    }

    /// Phase 3f polymorphic dense-index keyed IC cache-hit lookup. Walks
    /// the `[KeyedDenseIndexHandler; POLY_LIMIT]` sidecar for a shape+flags
    /// match. Mirrors `keyed_property_dense_index_handler` for shapes
    /// `2..POLY_LIMIT`.
    #[inline(always)]
    pub(super) fn keyed_property_dense_polymorphic_handlers(
        &self,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
    ) -> Option<&[KeyedDenseIndexHandler; POLY_LIMIT]> {
        let site = self.feedback_site_for_slot(code, slot?)?;
        match site {
            FeedbackSiteState::KeyedProperty(feedback)
                if matches!(feedback.cache_state, InlineCacheState::Polymorphic)
                    && feedback.family == Some(KeyedPropertyFamily::DenseIndex) =>
            {
                Some(&feedback.polymorphic_dense_index_handlers)
            }
            _ => None,
        }
    }

    pub(super) fn try_named_property_load_inline_cache_hit(
        &mut self,
        agent: &Agent,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
    ) -> Option<Value> {
        let slot = slot?;
        let site = self
            .feedback_vectors
            .get_mut(code_index(code))?
            .site_mut(slot)?;
        let value = match site {
            FeedbackSiteState::NamedProperty(feedback) => feedback.try_load(agent, receiver),
            _ => None,
        }?;
        site.record_execution();
        self.tiering.observe_feedback_event(code);
        Some(value)
    }

    pub(super) fn try_named_property_store_inline_cache(
        &self,
        agent: &mut Agent,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
        atom: AtomId,
        value: Value,
    ) -> Option<bool> {
        match self.feedback_site_for_slot(code, slot?) {
            Some(FeedbackSiteState::NamedProperty(feedback)) => {
                feedback.try_store(agent, receiver, atom, value)
            }
            _ => None,
        }
    }

    pub(super) fn observe_named_property_slow_path(
        &mut self,
        agent: &mut Agent,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
        atom: AtomId,
        purpose: NamedPropertyCachePurpose,
    ) {
        let Some(slot) = slot else {
            return;
        };
        let _ = self.ensure_feedback_slot_execution(code, slot);
        let plan = agent
            .objects()
            .plan_named_property_cache_entry(
                agent.heap().view(),
                receiver,
                PropertyKey::from_atom(atom),
                purpose,
            )
            .ok()
            .flatten();
        self.record_named_property_cache_entry(agent, code, slot, plan);
    }

    pub(super) fn observe_named_property_cache_entry(
        &mut self,
        agent: &mut Agent,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
        plan: Option<NamedPropertyCacheEntry>,
    ) {
        let Some(slot) = slot else {
            return;
        };
        let _ = self.ensure_feedback_slot_execution(code, slot);
        self.record_named_property_cache_entry(agent, code, slot, plan);
    }

    fn record_named_property_cache_entry(
        &mut self,
        agent: &mut Agent,
        code: CodeRef,
        slot: FeedbackSlotId,
        plan: Option<NamedPropertyCacheEntry>,
    ) {
        // Phase A.2 (Task A.2): after a watchpoint fire clears the IC slot to
        // `None`, the slot must be restored to `Uninitialized` state before
        // any watchpoint registration or `observe_slow_path` call so that
        // `with_feedback_slot_mut` can find the site and the generation bump
        // inside `register_proto_chain_watchpoints` operates on a live slot.
        // This no-ops if the slot is already present (not cleared).
        if let Some(vector) = self.feedback_vectors.get_mut(code_index(code)) {
            vector.reinit_named_property_site_if_cleared(slot);
        }

        // Spec 2 Phase A: a `PrototypeData` plan caches a load through a
        // prototype chain. Before committing the entry we register
        // `AdaptiveProtoLoad` watchpoints on every chain shape (the
        // prototype objects' shapes; the receiver's shape is guarded by
        // the IC cache-hit shape compare). If any chain shape is already
        // `Invalidated`, abandon the install — the next slow-path
        // observation will retry. Note that the abandon path calls
        // `clear_ic_slot_if_generation_matches` which sets the slot back to
        // `None`; the `reinit_named_property_site_if_cleared` above will
        // restore it again on the next observation.
        if let Some(plan_entry) = plan
            && plan_entry.path() == NamedPropertyCachePath::PrototypeData
            && !Self::register_proto_chain_watchpoints(self, agent, code, slot, plan_entry)
        {
            return;
        }
        let _ = self.with_feedback_slot_mut(code, slot, |site| {
            if let FeedbackSiteState::NamedProperty(feedback) = site {
                feedback.observe_slow_path(plan);
            }
        });
    }

    /// Returns `true` when the install may proceed (all chain shapes
    /// registered or the chain is empty), `false` when registration was
    /// abandoned because some shape was already `Invalidated`.
    ///
    /// `chain_shapes` are collected from `entry.dependency(1..dependency_count)`:
    /// dependency[0] is the receiver (covered by the IC cache-hit shape
    /// compare), so we skip it. The remaining dependencies are the
    /// prototype objects walked while planning the entry, last being the
    /// holder.
    fn register_proto_chain_watchpoints(
        vm: &mut Self,
        agent: &mut Agent,
        code: CodeRef,
        slot: FeedbackSlotId,
        entry: NamedPropertyCacheEntry,
    ) -> bool {
        // Slow-path install runs once per IC re-cache; tiny Vec allocation
        // is acceptable here (`PROPERTY_CACHE_MAX_DEPENDENCIES == 4`).
        // `dependency(0)` is the receiver (guarded by the IC cache-hit
        // shape compare); the remaining entries are the prototype objects'
        // shapes that we register `AdaptiveProtoLoad` watchpoints on.
        let mut chain: Vec<ShapeId> = Vec::with_capacity(PROPERTY_CACHE_MAX_DEPENDENCIES);
        for index in 1..usize::from(entry.dependency_count()) {
            let Some(dep) = entry.dependency(index) else {
                return false;
            };
            chain.push(dep.shape());
        }
        agent
            .register_adaptive_proto_load_for_chain(code, slot, &chain, vm)
            .is_ok()
    }

    pub(super) fn try_keyed_property_load_inline_cache(
        &self,
        agent: &Agent,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
        atom: AtomId,
    ) -> Option<Value> {
        match self.feedback_site_for_slot(code, slot?) {
            Some(FeedbackSiteState::KeyedProperty(feedback)) => {
                feedback.try_load(agent, receiver, atom)
            }
            _ => None,
        }
    }

    pub(super) fn try_keyed_property_store_inline_cache(
        &self,
        agent: &mut Agent,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
        atom: AtomId,
        value: Value,
    ) -> Option<bool> {
        match self.feedback_site_for_slot(code, slot?) {
            Some(FeedbackSiteState::KeyedProperty(feedback)) => {
                feedback.try_store(agent, receiver, atom, value)
            }
            _ => None,
        }
    }

    pub(super) fn try_keyed_dense_index_load_inline_cache_hit(
        &mut self,
        agent: &Agent,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
        index: u32,
    ) -> Option<Value> {
        let slot = slot?;
        let site = self
            .feedback_vectors
            .get_mut(code_index(code))?
            .site_mut(slot)?;
        let value = match site {
            FeedbackSiteState::KeyedProperty(feedback) => {
                feedback.try_dense_index_load(agent, receiver, index)
            }
            _ => None,
        }?;
        site.record_execution();
        // DSL-0b (B17) dual-write — borrow on `site` is dropped after
        // `record_execution()` so `mirror_flat_slot` can re-borrow.
        self.mirror_flat_slot(code, slot);
        self.tiering.observe_feedback_event(code);
        Some(value)
    }

    pub(super) fn try_keyed_dense_index_store_inline_cache_hit(
        &mut self,
        agent: &mut Agent,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
        index: u32,
        value: Value,
    ) -> Option<bool> {
        let slot = slot?;
        let site = self
            .feedback_vectors
            .get_mut(code_index(code))?
            .site_mut(slot)?;
        let stored = match site {
            FeedbackSiteState::KeyedProperty(feedback) => {
                feedback.try_dense_index_store(agent, receiver, index, value)
            }
            _ => None,
        }?;
        site.record_execution();
        // DSL-0b (B17) dual-write — see paired load helper.
        self.mirror_flat_slot(code, slot);
        self.tiering.observe_feedback_event(code);
        Some(stored)
    }

    pub(super) fn observe_keyed_atom_slow_path(
        &mut self,
        agent: &mut Agent,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
        atom: AtomId,
        purpose: NamedPropertyCachePurpose,
    ) {
        let Some(slot) = slot else {
            return;
        };
        let _ = self.ensure_feedback_slot_execution(code, slot);
        let plan = agent
            .objects()
            .plan_named_property_cache_entry(
                agent.heap().view(),
                receiver,
                PropertyKey::from_atom(atom),
                purpose,
            )
            .ok()
            .flatten();
        // Spec 2 Phase A: same proto-chain registration as the non-keyed
        // named-property slow path — see `record_named_property_cache_entry`.
        if let Some(plan_entry) = plan
            && plan_entry.path() == NamedPropertyCachePath::PrototypeData
            && !Self::register_proto_chain_watchpoints(self, agent, code, slot, plan_entry)
        {
            return;
        }
        let _ = self.with_feedback_slot_mut(code, slot, |site| {
            if let FeedbackSiteState::KeyedProperty(feedback) = site {
                feedback.observe_named_atom_slow_path(atom, plan);
            }
        });
    }

    fn observe_keyed_index_slow_path(
        &mut self,
        code: CodeRef,
        slot: FeedbackSlotId,
        plan: Option<DenseIndexCacheEntry>,
    ) {
        let _ = self.ensure_feedback_slot_execution(code, slot);
        let _ = self.with_feedback_slot_mut(code, slot, |site| {
            if let FeedbackSiteState::KeyedProperty(feedback) = site {
                let _ = feedback.observe_dense_index(plan);
            }
        });
    }

    pub(super) fn observe_keyed_index_access(
        &mut self,
        agent: &Agent,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
        receiver: ObjectRef,
        index: u32,
    ) {
        let Some(slot) = slot else {
            return;
        };
        let plan = KeyedPropertyFeedback::dense_index_plan(agent, receiver, index);
        self.observe_keyed_index_slow_path(code, slot, plan);
    }

    pub(super) fn observe_keyed_generic_slow_path(
        &mut self,
        code: CodeRef,
        slot: Option<FeedbackSlotId>,
    ) {
        let Some(slot) = slot else {
            return;
        };
        let _ = self.ensure_feedback_slot_execution(code, slot);
        let _ = self.with_feedback_slot_mut(code, slot, |site| {
            if let FeedbackSiteState::KeyedProperty(feedback) = site {
                feedback.observe_generic();
            }
        });
    }

    #[cfg(test)]
    pub(crate) fn has_feedback_vector(&self, code: CodeRef) -> bool {
        self.feedback_vectors
            .get(code_index(code))
            .is_some_and(FeedbackVector::is_allocated)
    }

    #[inline]
    pub fn feedback_vector_footprint(&self, code: CodeRef) -> Option<FeedbackVectorFootprint> {
        let index = code_index(code);
        let installed = self.installed.get(index).and_then(Option::as_ref)?;
        let slot_count = installed.feedback_slot_descriptors().len();
        let live_site_count = installed
            .feedback_slot_descriptors()
            .iter()
            .flatten()
            .count();
        let vector = self.feedback_vectors.get(index);
        // Match prior reporting semantics: the "allocated_bytes" total only counts the heap
        // budget for an actually-populated vector (sites populated). The sentinel default
        // value in `Vec<FeedbackVector>` reports 0 bytes so callers continue to use
        // `allocated == allocated_bytes > 0` as the cold-vs-warm signal.
        let allocated_bytes = vector.map_or(0, |vector| {
            if vector.is_allocated() {
                size_of::<FeedbackVector>() + vector.sites_capacity_bytes()
            } else {
                0
            }
        });

        Some(FeedbackVectorFootprint {
            allocated: allocated_bytes > 0,
            slot_count,
            live_site_count,
            allocated_bytes,
            warmup_counter: self.tiering.warmup_counter(code),
        })
    }

    #[inline]
    pub fn feedback_vector_snapshot(&self, code: CodeRef) -> Option<FeedbackVectorSnapshot> {
        let index = code_index(code);
        let installed = self.installed.get(index).and_then(Option::as_ref)?;
        let vector = self.feedback_vectors.get(index);
        let allocated = vector.is_some_and(FeedbackVector::is_allocated);
        let sites = installed
            .feedback_slot_descriptors()
            .iter()
            .flatten()
            .copied()
            .map(|descriptor| {
                vector
                    .and_then(|vector| vector.site(descriptor.slot()))
                    .map_or_else(
                        || FeedbackSiteState::unallocated_snapshot(descriptor),
                        |site| site.snapshot(descriptor),
                    )
            })
            .collect::<Vec<_>>();

        Some(FeedbackVectorSnapshot::new(
            allocated,
            self.tiering.warmup_counter(code),
            installed.feedback_slot_descriptors().len(),
            sites,
        ))
    }

    #[cfg(test)]
    pub(crate) fn feedback_warmup_counter(&self, code: CodeRef) -> Option<u16> {
        // Returns None if the code has no installed slot; otherwise reads warmup from Tiering.
        self.installed
            .get(code_index(code))
            .and_then(Option::as_ref)
            .map(|_| self.tiering.warmup_counter(code))
    }

    /// Assert that each flat LLInt IC header matches the legacy feedback
    /// slot it summarizes. Returns `Ok(())` on full match, or
    /// `Err((slot_index, diff_string))` describing the first divergence.
    #[doc(hidden)]
    #[allow(
        clippy::too_many_lines,
        reason = "debug assertion walks every flat feedback shape in one ordered comparison for readable mismatch reports"
    )]
    pub fn feedback_flat_matches_legacy(&self, code: CodeRef) -> Result<(), (usize, String)> {
        let index = code_index(code);
        let legacy_sites: &[Option<FeedbackSiteState>] = self
            .feedback_vectors
            .get(index)
            .map_or(&[], |vector| vector.sites.as_slice());
        let empty_flat: &[crate::dsl::feedback_flat::FeedbackEntry] = &[];
        let flat: &[crate::dsl::feedback_flat::FeedbackEntry] = self
            .feedback_flat_storage
            .get(index)
            .map_or(empty_flat, std::ops::Deref::deref);
        // Both storages may differ in length only when the legacy vector
        // is still unallocated and the flat array carries install-time
        // capacity. In that case every flat entry must be empty.
        if legacy_sites.is_empty() {
            for (i, entry) in flat.iter().enumerate() {
                if entry.mode() != crate::dsl::feedback_flat::LLINT_IC_MODE_EMPTY
                    || entry.named_handler_bits() != 0
                    || entry.named_aux_bits() != 0
                    || entry.scalar_observed_bits() != 0
                    || entry.scalar_execution_count() != 0
                {
                    return Err((
                        i,
                        format!(
                            "flat slot {i} populated while legacy vector is unallocated: mode={} handler={:#x} aux_bits={:#x} scalar_observed={:#x} scalar_count={}",
                            entry.mode(),
                            entry.named_handler_bits(),
                            entry.named_aux_bits(),
                            entry.scalar_observed_bits(),
                            entry.scalar_execution_count(),
                        ),
                    ));
                }
            }
            return Ok(());
        }
        // Once allocated, lengths must match.
        if legacy_sites.len() != flat.len() {
            return Err((
                0,
                format!(
                    "length mismatch: legacy={} flat={}",
                    legacy_sites.len(),
                    flat.len()
                ),
            ));
        }
        for (i, (legacy, flat_entry)) in legacy_sites.iter().zip(flat.iter()).enumerate() {
            let expected = legacy.as_ref().and_then(Self::named_llint_load_header);
            match expected {
                Some(LlIntNamedPropertyHeader::OwnInline { handler_bits })
                    if flat_entry.mode()
                        == crate::dsl::feedback_flat::LLINT_IC_MODE_NAMED_OWN_INLINE_LOAD
                        && flat_entry.named_handler_bits() == handler_bits
                        && flat_entry.named_aux_bits() == 0
                        && flat_entry.scalar_observed_bits() == 0
                        && flat_entry.scalar_execution_count() == 0 => {}
                Some(LlIntNamedPropertyHeader::OwnOutline { handler_bits })
                    if flat_entry.mode()
                        == crate::dsl::feedback_flat::LLINT_IC_MODE_NAMED_OWN_OUTLINE_LOAD
                        && flat_entry.named_handler_bits() == handler_bits
                        && flat_entry.named_aux_bits() == 0
                        && flat_entry.scalar_observed_bits() == 0
                        && flat_entry.scalar_execution_count() == 0 => {}
                Some(LlIntNamedPropertyHeader::ProtoInline {
                    receiver_word,
                    proto_word,
                }) if flat_entry.mode()
                    == crate::dsl::feedback_flat::LLINT_IC_MODE_NAMED_PROTO_INLINE_LOAD
                    && flat_entry.named_handler_bits() == proto_word
                    && flat_entry.named_aux_bits() == receiver_word
                    && flat_entry.scalar_observed_bits() == 0
                    && flat_entry.scalar_execution_count() == 0 => {}
                Some(expected) => {
                    return Err((
                        i,
                        Self::format_flat_header_divergence(i, expected, flat_entry),
                    ));
                }
                None if flat_entry.mode() == crate::dsl::feedback_flat::LLINT_IC_MODE_EMPTY
                    && flat_entry.named_handler_bits() == 0
                    && flat_entry.named_aux_bits() == 0
                    && flat_entry.scalar_observed_bits() == 0
                    && flat_entry.scalar_execution_count() == 0 => {}
                None => {
                    return Err((
                        i,
                        format!(
                            "slot {i} carried a flat LLInt header for an ineligible legacy slot: mode={} handler={:#x} aux_bits={:#x} scalar_observed={:#x} scalar_count={}",
                            flat_entry.mode(),
                            flat_entry.named_handler_bits(),
                            flat_entry.named_aux_bits(),
                            flat_entry.scalar_observed_bits(),
                            flat_entry.scalar_execution_count()
                        ),
                    ));
                }
            }
        }
        Ok(())
    }

    fn format_flat_header_divergence(
        slot_index: usize,
        expected: LlIntNamedPropertyHeader,
        flat_entry: &crate::dsl::feedback_flat::FeedbackEntry,
    ) -> String {
        let expected_text = match expected {
            LlIntNamedPropertyHeader::OwnInline { handler_bits } => format!(
                "mode={} handler={handler_bits:#x} aux_bits=0x0",
                crate::dsl::feedback_flat::LLINT_IC_MODE_NAMED_OWN_INLINE_LOAD,
            ),
            LlIntNamedPropertyHeader::OwnOutline { handler_bits } => format!(
                "mode={} handler={handler_bits:#x} aux_bits=0x0",
                crate::dsl::feedback_flat::LLINT_IC_MODE_NAMED_OWN_OUTLINE_LOAD,
            ),
            LlIntNamedPropertyHeader::ProtoInline {
                receiver_word,
                proto_word,
            } => format!(
                "mode={} handler={proto_word:#x} aux_bits={receiver_word:#x}",
                crate::dsl::feedback_flat::LLINT_IC_MODE_NAMED_PROTO_INLINE_LOAD,
            ),
            LlIntNamedPropertyHeader::OwnPolymorphic {
                slot0_handler_bits,
                slot1_handler_bits,
            } => format!(
                "mode={} handler={slot0_handler_bits:#x} aux_bits={slot1_handler_bits:#x}",
                crate::dsl::feedback_flat::LLINT_IC_MODE_NAMED_OWN_POLYMORPHIC,
            ),
        };
        format!(
            "slot {slot_index} header diverges: expected {expected_text}, flat mode={} handler={:#x} aux_bits={:#x} scalar_observed={:#x} scalar_count={}",
            flat_entry.mode(),
            flat_entry.named_handler_bits(),
            flat_entry.named_aux_bits(),
            flat_entry.scalar_observed_bits(),
            flat_entry.scalar_execution_count()
        )
    }

    #[cfg(test)]
    pub(crate) fn feedback_execution_count(
        &self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) -> Option<u32> {
        self.feedback_vectors
            .get(code_index(code))?
            .site(slot)
            .map(FeedbackSiteState::execution_count)
    }

    #[cfg(test)]
    pub(crate) fn flat_named_property_header_snapshot(
        &self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) -> Option<(u8, u64)> {
        let slot_index = Self::flat_feedback_slot_index(slot)?;
        let entry = self
            .feedback_flat_storage
            .get(code_index(code))?
            .get(slot_index)?;
        Some((entry.mode(), entry.named_handler_bits()))
    }

    #[cfg(test)]
    pub(crate) fn named_property_cache_snapshot(
        &self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) -> Option<(
        &'static str,
        u8,
        Option<lyng_objects::NamedPropertyCachePath>,
    )> {
        let state = self.feedback_vectors.get(code_index(code))?.site(slot)?;
        match state {
            FeedbackSiteState::NamedProperty(feedback) => Some((
                match feedback.cache_state {
                    InlineCacheState::Uninitialized => "Uninitialized",
                    InlineCacheState::Monomorphic => "Monomorphic",
                    InlineCacheState::Polymorphic => "Polymorphic",
                    InlineCacheState::Megamorphic => "Megamorphic",
                },
                feedback.entry_count,
                feedback.entries[0].map(NamedPropertyCacheEntry::path),
            )),
            _ => None,
        }
    }

    /// Returns the IC slot's current `(cache_state, generation)` tuple
    /// for a NamedProperty site. `None` if the slot is empty or is not a
    /// NamedProperty site. Used by Spec 2 Phase A tests to assert
    /// generation-bump and abandon-on-invalidate behaviours.
    #[cfg(test)]
    pub(crate) fn named_property_generation_snapshot(
        &self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) -> Option<(&'static str, u32)> {
        let state = self.feedback_vectors.get(code_index(code))?.site(slot)?;
        match state {
            FeedbackSiteState::NamedProperty(feedback) => Some((
                match feedback.cache_state {
                    InlineCacheState::Uninitialized => "Uninitialized",
                    InlineCacheState::Monomorphic => "Monomorphic",
                    InlineCacheState::Polymorphic => "Polymorphic",
                    InlineCacheState::Megamorphic => "Megamorphic",
                },
                feedback.generation,
            )),
            _ => None,
        }
    }

    /// Returns `true` iff the slot has a populated `NamedProperty` site
    /// (i.e. is not `None`/Uninitialized). Used by Spec 2 Phase A tests
    /// to assert that abandon-on-invalidated kept the slot clear.
    #[cfg(test)]
    pub(crate) fn named_property_slot_is_present(
        &self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) -> bool {
        self.feedback_vectors
            .get(code_index(code))
            .and_then(|vector| vector.site(slot))
            .is_some()
    }

    #[cfg(test)]
    pub(crate) fn keyed_property_cache_snapshot(
        &self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) -> Option<(&'static str, Option<&'static str>, u8)> {
        let state = self.feedback_vectors.get(code_index(code))?.site(slot)?;
        match state {
            FeedbackSiteState::KeyedProperty(feedback) => Some((
                match feedback.cache_state {
                    InlineCacheState::Uninitialized => "Uninitialized",
                    InlineCacheState::Monomorphic => "Monomorphic",
                    InlineCacheState::Polymorphic => "Polymorphic",
                    InlineCacheState::Megamorphic => "Megamorphic",
                },
                feedback.family.map(|family| match family {
                    KeyedPropertyFamily::DenseIndex => "DenseIndex",
                    KeyedPropertyFamily::NamedAtom => "NamedAtom",
                    KeyedPropertyFamily::Generic => "Generic",
                }),
                match feedback.family {
                    Some(KeyedPropertyFamily::DenseIndex) => feedback.dense_entry_count,
                    Some(KeyedPropertyFamily::NamedAtom) => feedback.named_entry_count,
                    Some(KeyedPropertyFamily::Generic) | None => 0,
                },
            )),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        call_feedback_builtin_is_frame_safe, DenseIndexCacheEntry, FeedbackVector,
        InlineCacheState, KeyedPropertyFamily, KeyedPropertyFeedback,
    };
    use lyng_bytecode::{FeedbackSiteDescriptor, FeedbackSiteKind};
    use lyng_objects::ObjectFlags;
    use lyng_types::{
        eval_builtin, function_builtin, function_call_builtin, string_char_code_at_builtin,
        FeedbackSlotId, ShapeId,
    };

    #[test]
    fn default_feedback_vector_is_unallocated() {
        let vector = FeedbackVector::default();
        assert!(
            !vector.is_allocated(),
            "default FeedbackVector should report unallocated so the hot path skips it"
        );
    }

    #[test]
    fn bump_warmup_on_tiering_state_increments_saturating_and_returns_new_value() {
        use super::super::tiering::TieringState;
        let mut state = TieringState::default();
        assert_eq!(state.warmup_counter(), 0);
        assert_eq!(state.bump_warmup(), 1);
        assert_eq!(state.bump_warmup(), 2);
        assert_eq!(state.warmup_counter(), 2);
    }

    #[test]
    fn bump_warmup_on_tiering_state_saturates_at_u16_max() {
        use super::super::tiering::TieringState;
        let mut state = TieringState::default();
        state.bump_warmup_by(u16::MAX);
        assert_eq!(state.bump_warmup(), u16::MAX);
        assert_eq!(state.warmup_counter(), u16::MAX);
    }

    #[test]
    fn vector_with_populated_sites_reports_allocated() {
        let slot = FeedbackSlotId::from_raw(1).expect("test slot id should be non-zero");
        let descriptor = FeedbackSiteDescriptor::new(slot, 0, FeedbackSiteKind::Arithmetic);
        let mut vector = FeedbackVector::default();
        vector.allocate_sites(&[Some(descriptor)]);
        assert!(
            vector.is_allocated(),
            "vector with non-empty site storage should report allocated"
        );
    }

    #[test]
    fn warmup_counter_lives_on_tiering_state_independent_of_allocation() {
        // After the warmup counter was lifted from FeedbackVector onto TieringState,
        // allocating sites has no effect on the counter — they are now independent.
        use super::super::tiering::TieringState;
        let slot = FeedbackSlotId::from_raw(1).expect("test slot id should be non-zero");
        let descriptor = FeedbackSiteDescriptor::new(slot, 0, FeedbackSiteKind::Arithmetic);
        let mut state = TieringState::default();
        state.bump_warmup();
        state.bump_warmup();
        assert_eq!(state.warmup_counter(), 2);
        let mut vector = FeedbackVector::default();
        vector.allocate_sites(&[Some(descriptor)]);
        assert!(vector.is_allocated());
        // Counter is on TieringState, unaffected by vector allocation.
        assert_eq!(state.warmup_counter(), 2);
    }

    #[test]
    fn dense_index_observation_reports_whether_classification_changed() {
        let mut feedback = KeyedPropertyFeedback::new();
        let plan = DenseIndexCacheEntry::new(
            ShapeId::from_raw(1).expect("test shape id should be non-zero"),
            ObjectFlags::extensible(),
        );

        assert!(feedback.observe_dense_index(Some(plan)));
        assert!(!feedback.observe_dense_index(Some(plan)));
        assert_eq!(feedback.family, Some(KeyedPropertyFamily::DenseIndex));
        assert_eq!(feedback.cache_state, InlineCacheState::Monomorphic);
        assert_eq!(feedback.dense_entry_count, 1);

        assert!(feedback.observe_dense_index(None));
        assert!(!feedback.observe_dense_index(None));
        assert_eq!(feedback.cache_state, InlineCacheState::Megamorphic);
        assert_eq!(feedback.dense_entry_count, 0);
    }

    #[test]
    fn frame_safe_builtin_classification_excludes_frame_observers() {
        assert!(call_feedback_builtin_is_frame_safe(
            string_char_code_at_builtin()
        ));
        assert!(!call_feedback_builtin_is_frame_safe(eval_builtin()));
        assert!(!call_feedback_builtin_is_frame_safe(function_builtin()));
        assert!(!call_feedback_builtin_is_frame_safe(function_call_builtin()));
    }
}
