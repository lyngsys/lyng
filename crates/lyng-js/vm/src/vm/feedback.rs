use super::*;
use lyng_js_bytecode::{FeedbackSiteDescriptor, FeedbackSiteKind};
use lyng_js_objects::{NamedPropertyCacheEntry, NamedPropertyCachePurpose};
use lyng_js_types::{FeedbackSlotId, PropertyKey, ShapeId};
use std::mem::size_of;

const FEEDBACK_ALLOCATION_THRESHOLD: u16 = 2;
const POLYMORPHIC_PROPERTY_CACHE_LIMIT: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum InlineCacheState {
    Uninitialized,
    Monomorphic,
    Polymorphic,
    Megamorphic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum KeyedPropertyFamily {
    DenseIndex,
    NamedAtom,
    Generic,
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
struct KeyedPropertyFeedback {
    execution_count: u32,
    family: Option<KeyedPropertyFamily>,
    cache_state: InlineCacheState,
    entry_count: u8,
    entries: [Option<KeyedNamedPropertyCacheEntry>; POLYMORPHIC_PROPERTY_CACHE_LIMIT],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CallFeedback {
    execution_count: u32,
    expected_arity: Option<u16>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ConstructFeedback {
    execution_count: u32,
    expected_arity: Option<u16>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
        for entry in self.active_entries() {
            match agent.objects().load_from_named_property_cache(
                agent.heap().view(),
                receiver,
                entry,
            ) {
                Ok(Some(value)) => return Some(value),
                Ok(None) | Err(_) => {}
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
        for entry in self.active_entries() {
            let result = agent.with_heap_and_objects(|heap, objects| {
                let mut mutator = heap.mutator();
                objects.store_to_named_property_cache(&mut mutator, receiver, entry, value)
            });
            match result {
                Ok(Some(stored)) => return Some(stored),
                Ok(None) | Err(_) => {}
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
    fn install_first_entry(&mut self, entry: NamedPropertyCacheEntry) {
        self.entries[0] = Some(entry);
        self.entry_count = 1;
        self.cache_state = InlineCacheState::Monomorphic;
    }

    #[inline]
    fn promote_to_megamorphic(&mut self) {
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
            entry_count: 0,
            entries: [None; POLYMORPHIC_PROPERTY_CACHE_LIMIT],
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
        for entry in self.active_entries() {
            if entry.atom != atom {
                continue;
            }
            match agent.objects().load_from_named_property_cache(
                agent.heap().view(),
                receiver,
                entry.entry,
            ) {
                Ok(Some(value)) => return Some(value),
                Ok(None) | Err(_) => {}
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
        for entry in self.active_entries() {
            if entry.atom != atom {
                continue;
            }
            let result = agent.with_heap_and_objects(|heap, objects| {
                let mut mutator = heap.mutator();
                objects.store_to_named_property_cache(&mut mutator, receiver, entry.entry, value)
            });
            match result {
                Ok(Some(stored)) => return Some(stored),
                Ok(None) | Err(_) => {}
            }
        }
        None
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
                self.entries[0] = Some(KeyedNamedPropertyCacheEntry { atom, entry: plan });
                self.entry_count = 1;
                self.cache_state = InlineCacheState::Monomorphic;
            }
            Some(KeyedPropertyFamily::NamedAtom) => match self.cache_state {
                InlineCacheState::Megamorphic => {}
                InlineCacheState::Uninitialized => {
                    self.entries[0] = Some(KeyedNamedPropertyCacheEntry { atom, entry: plan });
                    self.entry_count = 1;
                    self.cache_state = InlineCacheState::Monomorphic;
                }
                InlineCacheState::Monomorphic | InlineCacheState::Polymorphic => {
                    if let Some(index) = self.find_entry_index(atom, plan.receiver_shape()) {
                        self.entries[index] =
                            Some(KeyedNamedPropertyCacheEntry { atom, entry: plan });
                        return;
                    }
                    if usize::from(self.entry_count) >= POLYMORPHIC_PROPERTY_CACHE_LIMIT {
                        self.promote_to_megamorphic(Some(KeyedPropertyFamily::NamedAtom));
                        return;
                    }
                    self.entries[usize::from(self.entry_count)] =
                        Some(KeyedNamedPropertyCacheEntry { atom, entry: plan });
                    self.entry_count = self.entry_count.saturating_add(1);
                    self.cache_state = if self.entry_count <= 1 {
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
    fn observe_dense_index(&mut self) {
        match self.family {
            None | Some(KeyedPropertyFamily::DenseIndex) => {
                self.promote_to_megamorphic(Some(KeyedPropertyFamily::DenseIndex));
            }
            Some(KeyedPropertyFamily::NamedAtom | KeyedPropertyFamily::Generic) => {
                self.promote_to_megamorphic(Some(KeyedPropertyFamily::Generic));
            }
        }
    }

    #[inline]
    fn observe_generic(&mut self) {
        self.promote_to_megamorphic(Some(KeyedPropertyFamily::Generic));
    }

    #[inline]
    fn active_entries(&self) -> impl Iterator<Item = KeyedNamedPropertyCacheEntry> + '_ {
        self.entries
            .iter()
            .take(usize::from(self.entry_count))
            .filter_map(|entry| *entry)
    }

    #[inline]
    fn find_entry_index(&self, atom: AtomId, receiver_shape: ShapeId) -> Option<usize> {
        self.active_entries()
            .enumerate()
            .find_map(|(index, entry)| {
                (entry.atom == atom && entry.entry.receiver_shape() == receiver_shape)
                    .then_some(index)
            })
    }

    #[inline]
    fn promote_to_megamorphic(&mut self, family: Option<KeyedPropertyFamily>) {
        self.family = family;
        self.cache_state = InlineCacheState::Megamorphic;
        self.entry_count = 0;
        self.entries = [None; POLYMORPHIC_PROPERTY_CACHE_LIMIT];
    }
}

impl FeedbackSiteState {
    #[inline]
    fn for_descriptor(descriptor: FeedbackSiteDescriptor) -> Self {
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
            FeedbackSiteKind::Call => Self::Call(CallFeedback {
                execution_count: 0,
                expected_arity: descriptor.metadata().expected_arity(),
            }),
            FeedbackSiteKind::Construct => Self::Construct(ConstructFeedback {
                execution_count: 0,
                expected_arity: descriptor.metadata().expected_arity(),
            }),
        }
    }

    #[inline]
    fn record_execution(&mut self) {
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

    #[cfg(test)]
    #[inline]
    fn execution_count(self) -> u32 {
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
    fn site(&self, slot: FeedbackSlotId) -> Option<FeedbackSiteState> {
        self.sites
            .get(usize::try_from(slot.get().saturating_sub(1)).ok()?)
            .copied()
            .flatten()
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
    fn feedback_state_for_site(
        &self,
        code: CodeRef,
        instruction_offset: u32,
    ) -> Option<FeedbackSiteState> {
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

        if let Some(vector) = self.feedback_vectors[index].as_mut() {
            if let Some(site) = vector.site_mut(descriptor.slot()) {
                site.record_execution();
            }
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
        match self.feedback_state_for_site(code, instruction_offset) {
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
        match self.feedback_state_for_site(code, instruction_offset) {
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
        match self.feedback_state_for_site(code, instruction_offset) {
            Some(FeedbackSiteState::KeyedProperty(feedback)) => {
                feedback.try_store(agent, receiver, atom, value)
            }
            _ => None,
        }
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

    pub(super) fn observe_keyed_index_slow_path(&mut self, code: CodeRef, instruction_offset: u32) {
        let _ = self.ensure_feedback_site_execution(code, instruction_offset);
        let _ = self.with_feedback_site_mut(code, instruction_offset, |site| {
            if let FeedbackSiteState::KeyedProperty(feedback) = site {
                feedback.observe_dense_index();
            }
        });
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
                feedback.entry_count,
            )),
            _ => None,
        }
    }
}
