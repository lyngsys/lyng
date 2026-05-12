use super::{
    code_index, Agent, AtomId, CodeRef, FeedbackVectorFootprint, ObjectRef, RealmRef, Value, Vm,
};
use lyng_js_bytecode::{FeedbackSiteDescriptor, FeedbackSiteKind};
use lyng_js_gc::ValueStoreTarget;
use lyng_js_objects::{
    FunctionEntryIdentity, NamedPropertyCacheEntry, NamedPropertyCachePath,
    NamedPropertyCachePurpose, ObjectFlags, ObjectHeader, ObjectKind, PrimitiveWrapperKind,
    PropertyCacheDependency,
};
use lyng_js_types::{BuiltinFunctionId, FeedbackSlotId, PropertyKey, ShapeId};
use std::mem::size_of;

const FEEDBACK_ALLOCATION_THRESHOLD: u16 = 2;
const POLYMORPHIC_PROPERTY_CACHE_LIMIT: usize = 8;
const POLYMORPHIC_CALL_CACHE_LIMIT: usize = 8;

#[inline]
pub(super) fn call_feedback_builtin_is_frame_safe(entry: BuiltinFunctionId) -> bool {
    // Keep this whitelist narrow: these direct-call targets do not inspect caller
    // strictness, dynamically compile source, or re-enter through Function.prototype
    // call helpers, so dispatching from a monomorphic feedback entry preserves the
    // general call path's caller frame and callee realm behavior.
    entry == lyng_js_types::regexp_exec_builtin()
        || entry == lyng_js_types::regexp_symbol_replace_builtin()
        || entry == lyng_js_types::regexp_test_builtin()
        || entry == lyng_js_types::string_char_code_at_builtin()
        || entry == lyng_js_types::string_from_char_code_builtin()
        || entry == lyng_js_types::string_replace_builtin()
        || entry == lyng_js_types::string_to_upper_case_builtin()
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
struct ArithmeticFeedback {
    execution_count: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ComparisonFeedback {
    execution_count: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct NamedPropertyFeedback {
    execution_count: u32,
    cache_state: InlineCacheState,
    entry_count: u8,
    entries: [Option<NamedPropertyCacheEntry>; POLYMORPHIC_PROPERTY_CACHE_LIMIT],
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
struct KeyedPropertyFeedback {
    execution_count: u32,
    family: Option<KeyedPropertyFamily>,
    cache_state: InlineCacheState,
    named_entry_count: u8,
    named_entries: [Option<KeyedNamedPropertyCacheEntry>; POLYMORPHIC_PROPERTY_CACHE_LIMIT],
    dense_entry_count: u8,
    dense_entries: [Option<DenseIndexCacheEntry>; POLYMORPHIC_PROPERTY_CACHE_LIMIT],
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
        let realm = function.and_then(lyng_js_objects::FunctionObjectData::realm);
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
struct CallFeedback {
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
            .and_then(lyng_js_objects::FunctionObjectData::realm);
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
                .map(lyng_js_objects::ObjectHeader::shape)
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ConstructFeedback {
    execution_count: u32,
    expected_arity: Option<u16>,
    cache_state: InlineCacheState,
    entry_count: u8,
    entries: [Option<ConstructCacheEntry>; POLYMORPHIC_CALL_CACHE_LIMIT],
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum FeedbackSiteState {
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
        for entry in self.active_entries() {
            if entry.receiver_shape() != receiver_shape {
                continue;
            }
            if let Ok(Some(value)) =
                agent
                    .objects()
                    .load_from_named_property_cache(agent.heap().view(), receiver, entry)
            {
                return Some(value);
            }
        }
        None
    }

    #[inline]
    fn try_store(&self, agent: &mut Agent, receiver: ObjectRef, value: Value) -> Option<bool> {
        match self.cache_state {
            InlineCacheState::Monomorphic | InlineCacheState::Polymorphic => {}
            InlineCacheState::Uninitialized | InlineCacheState::Megamorphic => return None,
        }
        let receiver_shape = agent
            .objects()
            .object_header(agent.heap().view(), receiver)?
            .shape();
        for entry in self.active_entries() {
            if entry.receiver_shape() != receiver_shape {
                continue;
            }
            let result = agent.with_heap_and_objects(|heap, objects| {
                let mut mutator = heap.mutator();
                objects.store_to_named_property_cache(&mut mutator, receiver, entry, value)
            });
            if let Ok(Some(stored)) = result {
                return Some(stored);
            }
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
                if let Some(index) = self.find_entry_index(plan.receiver_shape()) {
                    self.entries[index] = Some(plan);
                    return;
                }
                if usize::from(self.entry_count) >= POLYMORPHIC_PROPERTY_CACHE_LIMIT {
                    self.promote_to_megamorphic();
                    return;
                }
                self.entries[usize::from(self.entry_count)] = Some(plan);
                self.entry_count = self.entry_count.saturating_add(1);
                self.cache_state = if self.entry_count <= 1 {
                    InlineCacheState::Monomorphic
                } else {
                    InlineCacheState::Polymorphic
                };
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
    }

    #[inline]
    const fn promote_to_megamorphic(&mut self) {
        self.cache_state = InlineCacheState::Megamorphic;
        self.entry_count = 0;
        self.entries = [None; POLYMORPHIC_PROPERTY_CACHE_LIMIT];
    }

    #[inline]
    fn find_entry_index(&self, receiver_shape: ShapeId) -> Option<usize> {
        self.active_entries()
            .enumerate()
            .find_map(|(index, entry)| (entry.receiver_shape() == receiver_shape).then_some(index))
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
        for entry in self.active_named_entries() {
            if entry.atom != atom || entry.entry.receiver_shape() != receiver_shape {
                continue;
            }
            if let Ok(Some(value)) = agent.objects().load_from_named_property_cache(
                agent.heap().view(),
                receiver,
                entry.entry,
            ) {
                return Some(value);
            }
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
        for entry in self.active_named_entries() {
            if entry.atom != atom || entry.entry.receiver_shape() != receiver_shape {
                continue;
            }
            let result = agent.with_heap_and_objects(|heap, objects| {
                let mut mutator = heap.mutator();
                objects.store_to_named_property_cache(&mut mutator, receiver, entry.entry, value)
            });
            if let Ok(Some(stored)) = result {
                return Some(stored);
            }
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
                    if let Some(index) = self.find_named_entry_index(atom, plan.receiver_shape()) {
                        self.named_entries[index] =
                            Some(KeyedNamedPropertyCacheEntry { atom, entry: plan });
                        return;
                    }
                    if usize::from(self.named_entry_count) >= POLYMORPHIC_PROPERTY_CACHE_LIMIT {
                        self.promote_to_megamorphic(Some(KeyedPropertyFamily::NamedAtom));
                        return;
                    }
                    self.named_entries[usize::from(self.named_entry_count)] =
                        Some(KeyedNamedPropertyCacheEntry { atom, entry: plan });
                    self.named_entry_count = self.named_entry_count.saturating_add(1);
                    self.cache_state = if self.named_entry_count <= 1 {
                        InlineCacheState::Monomorphic
                    } else {
                        InlineCacheState::Polymorphic
                    };
                }
            },
            Some(KeyedPropertyFamily::DenseIndex | KeyedPropertyFamily::Generic) => {
                self.promote_to_megamorphic(Some(KeyedPropertyFamily::Generic));
            }
        }
    }

    #[inline]
    fn observe_dense_index(&mut self, plan: Option<DenseIndexCacheEntry>) -> bool {
        let Some(plan) = plan else {
            return self.observe_uncacheable_dense_index();
        };
        match self.family {
            None | Some(KeyedPropertyFamily::DenseIndex) => {
                if self.family.is_none() {
                    self.install_first_dense_entry(plan);
                    return true;
                }
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
                            return changed;
                        }
                        if usize::from(self.dense_entry_count) >= POLYMORPHIC_PROPERTY_CACHE_LIMIT {
                            self.promote_to_megamorphic(Some(KeyedPropertyFamily::DenseIndex));
                            return true;
                        }
                        self.dense_entries[usize::from(self.dense_entry_count)] = Some(plan);
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
            Some(KeyedPropertyFamily::NamedAtom | KeyedPropertyFamily::Generic) => {
                self.promote_mixed_keyed_family_to_generic()
            }
        }
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
        self.active_named_entries()
            .enumerate()
            .find_map(|(index, entry)| {
                (entry.atom == atom && entry.entry.receiver_shape() == receiver_shape)
                    .then_some(index)
            })
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
        match self {
            Self::Arithmetic(feedback) => {
                feedback.execution_count = feedback.execution_count.saturating_add(1);
            }
            Self::Comparison(feedback) => {
                feedback.execution_count = feedback.execution_count.saturating_add(1);
            }
            Self::NamedProperty(feedback) => {
                feedback.execution_count = feedback.execution_count.saturating_add(1);
            }
            Self::KeyedProperty(feedback) => {
                feedback.execution_count = feedback.execution_count.saturating_add(1);
            }
            Self::Call(feedback) => {
                feedback.execution_count = feedback.execution_count.saturating_add(1);
            }
            Self::Construct(feedback) => {
                feedback.execution_count = feedback.execution_count.saturating_add(1);
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct FeedbackVector {
    sites: Vec<Option<FeedbackSiteState>>,
}

impl FeedbackVector {
    #[inline]
    fn from_slot_descriptors(slot_descriptors: &[Option<FeedbackSiteDescriptor>]) -> Self {
        let sites = slot_descriptors
            .iter()
            .copied()
            .map(|descriptor| descriptor.map(FeedbackSiteState::for_descriptor))
            .collect();
        Self { sites }
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
}

impl Vm {
    #[inline]
    fn ensure_feedback_capacity(&mut self, code: CodeRef) {
        let index = code_index(code);
        if self.feedback_warmup.len() <= index {
            self.feedback_warmup.resize(index + 1, 0);
        }
        if self.feedback_vectors.len() <= index {
            self.feedback_vectors.resize_with(index + 1, || None);
        }
    }

    #[inline]
    fn feedback_descriptor_for_site(
        &self,
        code: CodeRef,
        instruction_offset: u32,
    ) -> Option<FeedbackSiteDescriptor> {
        self.installed
            .get(code_index(code))
            .and_then(Option::as_ref)?
            .feedback_descriptor(instruction_offset)
    }

    #[inline]
    fn feedback_site_for_site(
        &self,
        code: CodeRef,
        instruction_offset: u32,
    ) -> Option<&FeedbackSiteState> {
        let descriptor = self.feedback_descriptor_for_site(code, instruction_offset)?;
        self.feedback_vectors
            .get(code_index(code))
            .and_then(Option::as_ref)?
            .site(descriptor.slot())
    }

    fn with_feedback_site_mut<R>(
        &mut self,
        code: CodeRef,
        instruction_offset: u32,
        f: impl FnOnce(&mut FeedbackSiteState) -> R,
    ) -> Option<R> {
        let descriptor = self.feedback_descriptor_for_site(code, instruction_offset)?;
        self.feedback_vectors
            .get_mut(code_index(code))
            .and_then(Option::as_mut)?
            .site_mut(descriptor.slot())
            .map(f)
    }

    fn ensure_feedback_site_execution(
        &mut self,
        code: CodeRef,
        instruction_offset: u32,
    ) -> Option<FeedbackSiteDescriptor> {
        self.ensure_feedback_capacity(code);
        let index = code_index(code);
        let needs_allocation = self.feedback_vectors[index].is_none()
            && self.feedback_warmup[index].saturating_add(1) >= FEEDBACK_ALLOCATION_THRESHOLD;
        let (descriptor, slot_descriptors) = {
            let installed = self.installed.get(index).and_then(Option::as_ref)?;
            let descriptor = installed.feedback_descriptor(instruction_offset)?;
            let slot_descriptors =
                needs_allocation.then(|| installed.feedback_slot_descriptors().to_vec());
            (descriptor, slot_descriptors)
        };

        if self.feedback_vectors[index].is_none() {
            self.feedback_warmup[index] = self.feedback_warmup[index].saturating_add(1);
            if let Some(slot_descriptors) = slot_descriptors.filter(|slots| !slots.is_empty()) {
                self.feedback_vectors[index] =
                    Some(FeedbackVector::from_slot_descriptors(&slot_descriptors));
            }
        }

        if let Some(vector) = self.feedback_vectors[index].as_mut()
            && let Some(site) = vector.site_mut(descriptor.slot())
        {
            site.record_execution();
        }
        self.observe_tier_feedback_event(code);

        Some(descriptor)
    }

    fn record_allocated_feedback_site(&mut self, code: CodeRef, instruction_offset: u32) -> bool {
        let index = code_index(code);
        if self
            .feedback_vectors
            .get(index)
            .and_then(Option::as_ref)
            .is_none()
        {
            return false;
        }
        let Some(descriptor) = self.feedback_descriptor_for_site(code, instruction_offset) else {
            return false;
        };
        let Some(site) = self
            .feedback_vectors
            .get_mut(index)
            .and_then(Option::as_mut)
            .and_then(|vector| vector.site_mut(descriptor.slot()))
        else {
            return false;
        };
        site.record_execution();
        self.observe_tier_feedback_event(code);
        true
    }

    pub(super) fn record_feedback_site(&mut self, code: CodeRef, instruction_offset: u32) {
        if self.record_allocated_feedback_site(code, instruction_offset) {
            return;
        }
        let _ = self.ensure_feedback_site_execution(code, instruction_offset);
    }

    #[inline]
    pub(super) fn observe_call_target(
        &mut self,
        agent: &Agent,
        code: CodeRef,
        instruction_offset: u32,
        callee: ObjectRef,
    ) {
        let index = code_index(code);
        if let Some(descriptor) = self.feedback_descriptor_for_site(code, instruction_offset)
            && let Some(site) = self
                .feedback_vectors
                .get_mut(index)
                .and_then(Option::as_mut)
                .and_then(|vector| vector.site_mut(descriptor.slot()))
        {
            site.record_call_target(agent, callee);
            self.observe_tier_feedback_event(code);
            return;
        }

        if self
            .ensure_feedback_site_execution(code, instruction_offset)
            .is_none()
        {
            return;
        }
        let _ = self.with_feedback_site_mut(code, instruction_offset, |site| {
            if let FeedbackSiteState::Call(feedback) = site {
                feedback.observe_target(agent, callee);
            }
        });
    }

    #[inline]
    pub(super) fn cached_frame_safe_builtin_call_target(
        &self,
        code: CodeRef,
        instruction_offset: u32,
        callee: ObjectRef,
    ) -> Option<BuiltinFunctionId> {
        match self.feedback_site_for_site(code, instruction_offset)? {
            FeedbackSiteState::Call(feedback) => feedback.frame_safe_builtin_target(callee),
            _ => None,
        }
    }

    #[inline]
    pub(super) fn observe_construct_target(
        &mut self,
        agent: &Agent,
        code: CodeRef,
        instruction_offset: u32,
        constructor: ObjectRef,
        created: Option<ObjectRef>,
    ) {
        let index = code_index(code);
        if let Some(descriptor) = self.feedback_descriptor_for_site(code, instruction_offset)
            && let Some(site) = self
                .feedback_vectors
                .get_mut(index)
                .and_then(Option::as_mut)
                .and_then(|vector| vector.site_mut(descriptor.slot()))
        {
            site.record_construct_target(agent, constructor, created);
            self.observe_tier_feedback_event(code);
            return;
        }

        if self
            .ensure_feedback_site_execution(code, instruction_offset)
            .is_none()
        {
            return;
        }
        let _ = self.with_feedback_site_mut(code, instruction_offset, |site| {
            if let FeedbackSiteState::Construct(feedback) = site {
                feedback.observe_target(agent, constructor, created);
            }
        });
    }

    pub(super) fn try_named_property_load_inline_cache_hit(
        &mut self,
        agent: &Agent,
        code: CodeRef,
        instruction_offset: u32,
        receiver: ObjectRef,
    ) -> Option<Value> {
        let descriptor = self.feedback_descriptor_for_site(code, instruction_offset)?;
        let site = self
            .feedback_vectors
            .get_mut(code_index(code))
            .and_then(Option::as_mut)?
            .site_mut(descriptor.slot())?;
        let value = match site {
            FeedbackSiteState::NamedProperty(feedback) => feedback.try_load(agent, receiver),
            _ => None,
        }?;
        site.record_execution();
        self.observe_tier_feedback_event(code);
        Some(value)
    }

    pub(super) fn try_named_property_store_inline_cache(
        &self,
        agent: &mut Agent,
        code: CodeRef,
        instruction_offset: u32,
        receiver: ObjectRef,
        value: Value,
    ) -> Option<bool> {
        match self.feedback_site_for_site(code, instruction_offset) {
            Some(FeedbackSiteState::NamedProperty(feedback)) => {
                feedback.try_store(agent, receiver, value)
            }
            _ => None,
        }
    }

    pub(super) fn observe_named_property_slow_path(
        &mut self,
        agent: &Agent,
        code: CodeRef,
        instruction_offset: u32,
        receiver: ObjectRef,
        atom: AtomId,
        purpose: NamedPropertyCachePurpose,
    ) {
        let _ = self.ensure_feedback_site_execution(code, instruction_offset);
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
        let _ = self.with_feedback_site_mut(code, instruction_offset, |site| {
            if let FeedbackSiteState::NamedProperty(feedback) = site {
                feedback.observe_slow_path(plan);
            }
        });
    }

    pub(super) fn try_keyed_property_load_inline_cache(
        &self,
        agent: &Agent,
        code: CodeRef,
        instruction_offset: u32,
        receiver: ObjectRef,
        atom: AtomId,
    ) -> Option<Value> {
        match self.feedback_site_for_site(code, instruction_offset) {
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
        instruction_offset: u32,
        receiver: ObjectRef,
        atom: AtomId,
        value: Value,
    ) -> Option<bool> {
        match self.feedback_site_for_site(code, instruction_offset) {
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
        instruction_offset: u32,
        receiver: ObjectRef,
        index: u32,
    ) -> Option<Value> {
        let descriptor = self.feedback_descriptor_for_site(code, instruction_offset)?;
        let site = self
            .feedback_vectors
            .get_mut(code_index(code))
            .and_then(Option::as_mut)?
            .site_mut(descriptor.slot())?;
        let value = match site {
            FeedbackSiteState::KeyedProperty(feedback) => {
                feedback.try_dense_index_load(agent, receiver, index)
            }
            _ => None,
        }?;
        site.record_execution();
        self.observe_tier_feedback_event(code);
        Some(value)
    }

    pub(super) fn try_keyed_dense_index_store_inline_cache_hit(
        &mut self,
        agent: &mut Agent,
        code: CodeRef,
        instruction_offset: u32,
        receiver: ObjectRef,
        index: u32,
        value: Value,
    ) -> Option<bool> {
        let descriptor = self.feedback_descriptor_for_site(code, instruction_offset)?;
        let site = self
            .feedback_vectors
            .get_mut(code_index(code))
            .and_then(Option::as_mut)?
            .site_mut(descriptor.slot())?;
        let stored = match site {
            FeedbackSiteState::KeyedProperty(feedback) => {
                feedback.try_dense_index_store(agent, receiver, index, value)
            }
            _ => None,
        }?;
        site.record_execution();
        self.observe_tier_feedback_event(code);
        Some(stored)
    }

    pub(super) fn observe_keyed_atom_slow_path(
        &mut self,
        agent: &Agent,
        code: CodeRef,
        instruction_offset: u32,
        receiver: ObjectRef,
        atom: AtomId,
        purpose: NamedPropertyCachePurpose,
    ) {
        let _ = self.ensure_feedback_site_execution(code, instruction_offset);
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
        let _ = self.with_feedback_site_mut(code, instruction_offset, |site| {
            if let FeedbackSiteState::KeyedProperty(feedback) = site {
                feedback.observe_named_atom_slow_path(atom, plan);
            }
        });
    }

    fn observe_keyed_index_slow_path(
        &mut self,
        code: CodeRef,
        instruction_offset: u32,
        plan: Option<DenseIndexCacheEntry>,
    ) {
        let _ = self.ensure_feedback_site_execution(code, instruction_offset);
        let _ = self.with_feedback_site_mut(code, instruction_offset, |site| {
            if let FeedbackSiteState::KeyedProperty(feedback) = site {
                let _ = feedback.observe_dense_index(plan);
            }
        });
    }

    pub(super) fn observe_keyed_index_access(
        &mut self,
        agent: &Agent,
        code: CodeRef,
        instruction_offset: u32,
        receiver: ObjectRef,
        index: u32,
    ) {
        let plan = KeyedPropertyFeedback::dense_index_plan(agent, receiver, index);
        self.observe_keyed_index_slow_path(code, instruction_offset, plan);
    }

    pub(super) fn observe_keyed_generic_slow_path(
        &mut self,
        code: CodeRef,
        instruction_offset: u32,
    ) {
        let _ = self.ensure_feedback_site_execution(code, instruction_offset);
        let _ = self.with_feedback_site_mut(code, instruction_offset, |site| {
            if let FeedbackSiteState::KeyedProperty(feedback) = site {
                feedback.observe_generic();
            }
        });
    }

    #[cfg(test)]
    pub(crate) fn has_feedback_vector(&self, code: CodeRef) -> bool {
        self.feedback_vectors
            .get(code_index(code))
            .and_then(Option::as_ref)
            .is_some()
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
        let allocated_bytes = self
            .feedback_vectors
            .get(index)
            .and_then(Option::as_ref)
            .map_or(0, |vector| {
                size_of::<FeedbackVector>()
                    + vector.sites.len() * size_of::<Option<FeedbackSiteState>>()
            });

        Some(FeedbackVectorFootprint {
            allocated: allocated_bytes > 0,
            slot_count,
            live_site_count,
            allocated_bytes,
            warmup_counter: self.feedback_warmup.get(index).copied().unwrap_or(0),
        })
    }

    #[inline]
    pub fn feedback_vector_snapshot(&self, code: CodeRef) -> Option<FeedbackVectorSnapshot> {
        let index = code_index(code);
        let installed = self.installed.get(index).and_then(Option::as_ref)?;
        let vector = self.feedback_vectors.get(index).and_then(Option::as_ref);
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
            vector.is_some(),
            self.feedback_warmup.get(index).copied().unwrap_or(0),
            installed.feedback_slot_descriptors().len(),
            sites,
        ))
    }

    #[cfg(test)]
    pub(crate) fn feedback_warmup_counter(&self, code: CodeRef) -> Option<u16> {
        self.feedback_warmup.get(code_index(code)).copied()
    }

    #[cfg(test)]
    pub(crate) fn feedback_execution_count(
        &self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) -> Option<u32> {
        self.feedback_vectors
            .get(code_index(code))
            .and_then(Option::as_ref)?
            .site(slot)
            .map(FeedbackSiteState::execution_count)
    }

    #[cfg(test)]
    pub(crate) fn named_property_cache_snapshot(
        &self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) -> Option<(
        &'static str,
        u8,
        Option<lyng_js_objects::NamedPropertyCachePath>,
    )> {
        let state = self
            .feedback_vectors
            .get(code_index(code))
            .and_then(Option::as_ref)?
            .site(slot)?;
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

    #[cfg(test)]
    pub(crate) fn keyed_property_cache_snapshot(
        &self,
        code: CodeRef,
        slot: FeedbackSlotId,
    ) -> Option<(&'static str, Option<&'static str>, u8)> {
        let state = self
            .feedback_vectors
            .get(code_index(code))
            .and_then(Option::as_ref)?
            .site(slot)?;
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
        call_feedback_builtin_is_frame_safe, DenseIndexCacheEntry, InlineCacheState,
        KeyedPropertyFamily, KeyedPropertyFeedback,
    };
    use lyng_js_objects::ObjectFlags;
    use lyng_js_types::{
        eval_builtin, function_builtin, function_call_builtin, string_char_code_at_builtin, ShapeId,
    };

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
