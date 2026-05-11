use super::{
    flattened_property_lookup, DescriptorAttributes, ElementMode, InvalidationEvent,
    NamedPropertyDictionaryEntry, NamedPropertyStorageMode, NamedPropertyValue, ObjectColdData,
    ObjectFlags, ObjectKind, ObjectRef, PropertyKey, ShapeId, ShapeProperty, ShapeTransitionKey,
    SparseElementEntry,
};
use lyng_js_common::AtomId;
use std::collections::HashMap;

const INLINE_SHAPE_TRANSITION_LIMIT: usize = 3;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectMetadata {
    pub(crate) kind: ObjectKind,
    pub(crate) flags: ObjectFlags,
    pub(crate) cold: ObjectColdData,
    pub(crate) private_brands: Vec<InstalledPrivateBrand>,
    pub(crate) named_properties: NamedPropertyStorage,
    pub(crate) named_property_additions: u32,
    pub(crate) named_property_churn: u32,
    pub(crate) element_storage: ElementStorageMetadata,
    pub(crate) last_invalidation: Option<InvalidationEvent>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PrivateBrandId(u32);

impl PrivateBrandId {
    #[inline]
    pub(crate) const fn new(raw: u32) -> Self {
        Self(raw)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstalledPrivateBrand {
    brand: PrivateBrandId,
    slot_base: u32,
    slot_count: u32,
}

impl InstalledPrivateBrand {
    #[inline]
    pub(crate) const fn new(brand: PrivateBrandId, slot_base: u32, slot_count: u32) -> Self {
        Self {
            brand,
            slot_base,
            slot_count,
        }
    }

    #[inline]
    pub(crate) const fn brand(self) -> PrivateBrandId {
        self.brand
    }

    #[inline]
    pub(crate) const fn slot_base(self) -> u32 {
        self.slot_base
    }

    #[inline]
    pub(crate) const fn slot_count(self) -> u32 {
        self.slot_count
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ClassPrivateElementKind {
    Field,
    Method,
    Getter,
    Setter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClassPrivateElementDescriptor {
    name: AtomId,
    is_static: bool,
    kind: ClassPrivateElementKind,
    slot_index: u32,
}

impl ClassPrivateElementDescriptor {
    #[inline]
    pub(crate) const fn new(
        name: AtomId,
        is_static: bool,
        kind: ClassPrivateElementKind,
        slot_index: u32,
    ) -> Self {
        Self {
            name,
            is_static,
            kind,
            slot_index,
        }
    }

    #[inline]
    pub(crate) const fn name(self) -> AtomId {
        self.name
    }

    #[inline]
    pub(crate) const fn is_static(self) -> bool {
        self.is_static
    }

    #[inline]
    pub(crate) const fn kind(self) -> ClassPrivateElementKind {
        self.kind
    }

    #[inline]
    pub(crate) const fn slot_index(self) -> u32 {
        self.slot_index
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrivateDescriptorSummary {
    name: AtomId,
    is_static: bool,
    kind: ClassPrivateElementKind,
}

impl PrivateDescriptorSummary {
    #[inline]
    pub(crate) const fn new(name: AtomId, is_static: bool, kind: ClassPrivateElementKind) -> Self {
        Self {
            name,
            is_static,
            kind,
        }
    }

    #[inline]
    pub const fn name(self) -> AtomId {
        self.name
    }

    #[inline]
    pub const fn is_static(self) -> bool {
        self.is_static
    }

    #[inline]
    pub const fn kind(self) -> ClassPrivateElementKind {
        self.kind
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassRecord {
    pub(crate) instance_brand: Option<PrivateBrandId>,
    pub(crate) static_brand: Option<PrivateBrandId>,
    pub(crate) instance_slot_count: u32,
    pub(crate) static_slot_count: u32,
    pub(crate) instance_shared_slot_count: u32,
    pub(crate) static_shared_slot_count: u32,
    pub(crate) instance_public_field_key_slots: Vec<u32>,
    pub(crate) descriptors: Vec<ClassPrivateElementDescriptor>,
}

impl ClassRecord {
    #[inline]
    pub(crate) const fn new(
        instance_brand: Option<PrivateBrandId>,
        static_brand: Option<PrivateBrandId>,
    ) -> Self {
        Self {
            instance_brand,
            static_brand,
            instance_slot_count: 0,
            static_slot_count: 0,
            instance_shared_slot_count: 0,
            static_shared_slot_count: 0,
            instance_public_field_key_slots: Vec::new(),
            descriptors: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NamedPropertyStorage {
    ShapeStable,
    Dictionary(NamedPropertyDictionary),
}

impl NamedPropertyStorage {
    #[inline]
    pub(crate) const fn mode(&self) -> NamedPropertyStorageMode {
        match self {
            Self::ShapeStable => NamedPropertyStorageMode::ShapeStable,
            Self::Dictionary(_) => NamedPropertyStorageMode::Dictionary,
        }
    }

    #[inline]
    pub(crate) const fn is_dictionary(&self) -> bool {
        matches!(self, Self::Dictionary(_))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamedPropertyDictionary {
    pub(crate) entries: HashMap<PropertyKey, NamedPropertyDictionaryEntry>,
    pub(crate) next_enumeration_index: u32,
}

impl NamedPropertyDictionary {
    pub(crate) const fn new(
        entries: HashMap<PropertyKey, NamedPropertyDictionaryEntry>,
        next_index: u32,
    ) -> Self {
        Self {
            entries,
            next_enumeration_index: next_index,
        }
    }

    pub(crate) fn entry(&self, key: PropertyKey) -> Option<NamedPropertyDictionaryEntry> {
        self.entries.get(&key).copied()
    }

    pub(crate) fn ordered_entries(&self) -> Vec<NamedPropertyDictionaryEntry> {
        let mut entries = self.entries.values().copied().collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.enumeration_index());
        entries
    }

    pub(crate) fn upsert(
        &mut self,
        key: PropertyKey,
        payload: NamedPropertyValue,
        attrs: DescriptorAttributes,
    ) -> NamedPropertyDictionaryEntry {
        if let Some(entry) = self.entries.get_mut(&key) {
            entry.payload = payload;
            entry.attrs = attrs;
            return *entry;
        }

        let entry =
            NamedPropertyDictionaryEntry::new(key, attrs, payload, self.next_enumeration_index);
        self.next_enumeration_index = self.next_enumeration_index.saturating_add(1);
        self.entries.insert(key, entry);
        entry
    }

    pub(crate) fn remove(&mut self, key: PropertyKey) -> Option<NamedPropertyDictionaryEntry> {
        self.entries.remove(&key)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ElementStorageMetadata {
    Empty,
    Dense {
        logical_len: u32,
    },
    Sparse {
        entries: HashMap<u32, SparseElementEntry>,
        logical_len: u32,
    },
}

impl ElementStorageMetadata {
    #[inline]
    pub(crate) const fn mode(&self) -> ElementMode {
        match self {
            Self::Empty => ElementMode::Empty,
            Self::Dense { .. } => ElementMode::Dense,
            Self::Sparse { .. } => ElementMode::Sparse,
        }
    }

    #[inline]
    pub(crate) const fn logical_len(&self) -> u32 {
        match self {
            Self::Empty => 0,
            Self::Dense { logical_len } | Self::Sparse { logical_len, .. } => *logical_len,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShapeMetadata {
    pub(crate) transition_key: Option<ShapeTransitionKey>,
    pub(crate) property: Option<ShapeProperty>,
    pub(crate) properties: Vec<ShapeProperty>,
    pub(crate) property_lookup: Option<HashMap<PropertyKey, usize>>,
    pub(crate) transitions: ShapeTransitionStorage,
}

impl ShapeMetadata {
    pub(crate) const fn bootstrap() -> Self {
        Self {
            transition_key: None,
            property: None,
            properties: Vec::new(),
            property_lookup: None,
            transitions: ShapeTransitionStorage::new(),
        }
    }

    pub(crate) fn derived(
        transition_key: ShapeTransitionKey,
        properties: Vec<ShapeProperty>,
    ) -> Self {
        let property_lookup = flattened_property_lookup(&properties);
        Self {
            transition_key: Some(transition_key),
            property: properties.last().copied(),
            properties,
            property_lookup,
            transitions: ShapeTransitionStorage::new(),
        }
    }

    pub(crate) fn property(&self, key: PropertyKey) -> Option<ShapeProperty> {
        self.property_lookup.as_ref().map_or_else(
            || {
                self.properties
                    .iter()
                    .find(|property| property.key() == key)
                    .copied()
            },
            |lookup| {
                lookup
                    .get(&key)
                    .copied()
                    .and_then(|index| self.properties.get(index).copied())
            },
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShapeTransitionStorage {
    inline: [Option<(ShapeTransitionKey, ShapeId)>; INLINE_SHAPE_TRANSITION_LIMIT],
    inline_len: usize,
    overflow: Option<HashMap<ShapeTransitionKey, ShapeId>>,
}

impl ShapeTransitionStorage {
    #[inline]
    pub(crate) const fn new() -> Self {
        Self {
            inline: [None; INLINE_SHAPE_TRANSITION_LIMIT],
            inline_len: 0,
            overflow: None,
        }
    }

    #[inline]
    pub(crate) fn get(&self, key: &ShapeTransitionKey) -> Option<&ShapeId> {
        self.inline[..self.inline_len]
            .iter()
            .flatten()
            .find_map(|(entry_key, shape)| (entry_key == key).then_some(shape))
            .or_else(|| {
                self.overflow
                    .as_ref()
                    .and_then(|overflow| overflow.get(key))
            })
    }

    #[inline]
    pub(crate) fn insert(&mut self, key: ShapeTransitionKey, shape: ShapeId) -> Option<ShapeId> {
        for slot in &mut self.inline[..self.inline_len] {
            if let Some((entry_key, existing)) = slot
                && *entry_key == key
            {
                return Some(std::mem::replace(existing, shape));
            }
        }

        if let Some(overflow) = &mut self.overflow
            && let Some(existing) = overflow.get_mut(&key)
        {
            return Some(std::mem::replace(existing, shape));
        }

        if self.inline_len < INLINE_SHAPE_TRANSITION_LIMIT {
            self.inline[self.inline_len] = Some((key, shape));
            self.inline_len += 1;
            return None;
        }

        self.overflow
            .get_or_insert_with(HashMap::new)
            .insert(key, shape)
    }

    #[inline]
    pub(crate) fn remove(&mut self, key: &ShapeTransitionKey) -> Option<ShapeId> {
        if let Some(index) = self.inline[..self.inline_len]
            .iter()
            .position(|entry| entry.is_some_and(|(entry_key, _)| entry_key == *key))
        {
            let (_, removed) =
                self.inline[index].expect("inline transition slot should be populated");
            self.remove_inline(index);
            return Some(removed);
        }

        let overflow = self.overflow.as_mut()?;
        let removed = overflow.remove(key);
        if overflow.is_empty() {
            self.overflow = None;
        }
        removed
    }

    #[cfg(test)]
    #[inline]
    pub(crate) fn contains_key(&self, key: &ShapeTransitionKey) -> bool {
        self.get(key).is_some()
    }

    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.inline_len == 0
            && self
                .overflow
                .as_ref()
                .is_none_or(std::collections::HashMap::is_empty)
    }

    #[cfg(test)]
    #[inline]
    pub(crate) const fn inline_len(&self) -> usize {
        self.inline_len
    }

    #[cfg(test)]
    #[inline]
    pub(crate) fn overflow_len(&self) -> usize {
        self.overflow.as_ref().map_or(0, HashMap::len)
    }

    fn remove_inline(&mut self, index: usize) {
        for slot in index..self.inline_len - 1 {
            self.inline[slot] = self.inline[slot + 1];
        }
        self.inline_len -= 1;
        self.inline[self.inline_len] = None;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RootShapeKey {
    pub(crate) prototype_guard: Option<ObjectRef>,
}
