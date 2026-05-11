use std::collections::{HashMap, HashSet};

use lyng_js_types::{ObjectRef, SymbolRef, Value};

/// Heap handle classes that ECMAScript permits to be held weakly.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum WeakHeapRef {
    Object(ObjectRef),
    Symbol(SymbolRef),
}

impl WeakHeapRef {
    #[inline]
    pub const fn from_value(value: Value) -> Option<Self> {
        if let Some(object) = value.as_object_ref() {
            return Some(Self::Object(object));
        }
        if let Some(symbol) = value.as_symbol_ref() {
            return Some(Self::Symbol(symbol));
        }
        None
    }

    #[inline]
    pub const fn as_object(self) -> Option<ObjectRef> {
        match self {
            Self::Object(object) => Some(object),
            Self::Symbol(_) => None,
        }
    }

    #[inline]
    pub const fn as_symbol(self) -> Option<SymbolRef> {
        match self {
            Self::Object(_) => None,
            Self::Symbol(symbol) => Some(symbol),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WeakMapEntry {
    key: WeakHeapRef,
    value: Value,
}

impl WeakMapEntry {
    #[inline]
    pub(crate) const fn new(key: WeakHeapRef, value: Value) -> Self {
        Self { key, value }
    }

    #[inline]
    pub(crate) const fn key(self) -> WeakHeapRef {
        self.key
    }

    #[inline]
    pub(crate) const fn value(self) -> Value {
        self.value
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WeakMapState {
    entries: HashMap<WeakHeapRef, Value>,
}

impl WeakMapState {
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    #[inline]
    pub(crate) fn entries(&self) -> impl Iterator<Item = WeakMapEntry> + '_ {
        self.entries
            .iter()
            .map(|(key, value)| WeakMapEntry::new(*key, *value))
    }

    pub(crate) fn get(&self, key: WeakHeapRef) -> Option<Value> {
        self.entries.get(&key).copied()
    }

    pub(crate) fn set(&mut self, key: WeakHeapRef, value: Value) {
        self.entries.insert(key, value);
    }

    pub(crate) fn delete(&mut self, key: WeakHeapRef) -> bool {
        self.entries.remove(&key).is_some()
    }

    pub(crate) fn retain_live_keys(&mut self, mut is_live: impl FnMut(WeakHeapRef) -> bool) {
        self.entries.retain(|key, _| is_live(*key));
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WeakSetState {
    entries: HashSet<WeakHeapRef>,
}

impl WeakSetState {
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            entries: HashSet::new(),
        }
    }

    #[inline]
    pub(crate) fn contains(&self, value: WeakHeapRef) -> bool {
        self.entries.contains(&value)
    }

    pub(crate) fn insert(&mut self, value: WeakHeapRef) {
        self.entries.insert(value);
    }

    pub(crate) fn delete(&mut self, value: WeakHeapRef) -> bool {
        self.entries.remove(&value)
    }

    pub(crate) fn retain_live_values(&mut self, mut is_live: impl FnMut(WeakHeapRef) -> bool) {
        self.entries.retain(|entry| is_live(*entry));
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WeakRefState {
    target: Option<WeakHeapRef>,
}

impl WeakRefState {
    #[inline]
    pub(crate) const fn new(target: WeakHeapRef) -> Self {
        Self {
            target: Some(target),
        }
    }

    #[inline]
    pub(crate) const fn target(self) -> Option<WeakHeapRef> {
        self.target
    }

    #[inline]
    pub(crate) fn clear_if_dead(&mut self, mut is_live: impl FnMut(WeakHeapRef) -> bool) -> bool {
        let Some(target) = self.target else {
            return false;
        };
        if is_live(target) {
            return false;
        }
        self.target = None;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn object_key(raw: u32) -> WeakHeapRef {
        WeakHeapRef::Object(ObjectRef::from_raw(raw).expect("test object handle"))
    }

    #[test]
    fn weak_set_tracks_insert_duplicate_contains_and_delete() {
        let first = object_key(1);
        let second = object_key(2);
        let mut state = WeakSetState::new();

        assert!(!state.contains(first));
        state.insert(first);
        state.insert(first);
        assert!(state.contains(first));
        assert!(!state.contains(second));

        let mut retained_count = 0;
        state.retain_live_values(|value| {
            retained_count += 1;
            value == first
        });
        assert_eq!(retained_count, 1);
        assert!(state.contains(first));

        state.insert(second);
        assert!(state.delete(first));
        assert!(!state.delete(first));
        assert!(!state.contains(first));
        assert!(state.contains(second));
    }

    #[test]
    fn weak_set_retain_live_values_filters_dead_members() {
        let mut state = WeakSetState::new();
        for raw in 1..=5 {
            state.insert(object_key(raw));
        }

        state.retain_live_values(
            |value| matches!(value, WeakHeapRef::Object(object) if object.get() % 2 == 1),
        );

        assert!(state.contains(object_key(1)));
        assert!(!state.contains(object_key(2)));
        assert!(state.contains(object_key(3)));
        assert!(!state.contains(object_key(4)));
        assert!(state.contains(object_key(5)));
    }

    #[test]
    fn weak_set_membership_scales_for_deep_sets() {
        const ENTRY_COUNT: u32 = 12_000;

        let mut state = WeakSetState::new();
        let started = Instant::now();
        for raw in 1..=ENTRY_COUNT {
            state.insert(object_key(raw));
        }
        let mut observed = 0_u32;
        for raw in 1..=ENTRY_COUNT {
            if state.contains(object_key(raw)) {
                observed += 1;
            }
        }
        let elapsed = started.elapsed();

        assert_eq!(observed, ENTRY_COUNT);
        assert!(
            elapsed < Duration::from_millis(50),
            "weak set insert and lookup over {ENTRY_COUNT} entries took {elapsed:?}"
        );
    }

    #[test]
    fn weak_map_lookup_scales_for_deep_chains() {
        const ENTRY_COUNT: u32 = 12_000;

        let mut state = WeakMapState::new();
        for raw in 1..=ENTRY_COUNT {
            state.set(object_key(raw), Value::from_smi(raw.cast_signed()));
        }

        let started = Instant::now();
        let mut observed = 0_i64;
        for raw in 1..=ENTRY_COUNT {
            let value = state
                .get(object_key(raw))
                .and_then(Value::as_smi)
                .expect("weak map entry");
            observed += i64::from(value);
        }
        let elapsed = started.elapsed();

        assert_eq!(observed, i64::from(ENTRY_COUNT * (ENTRY_COUNT + 1) / 2));
        assert!(
            elapsed < Duration::from_millis(50),
            "weak map lookup over {ENTRY_COUNT} entries took {elapsed:?}"
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FinalizationCell {
    target: WeakHeapRef,
    holdings: Value,
    unregister_token: Option<WeakHeapRef>,
}

impl FinalizationCell {
    #[inline]
    pub(crate) const fn new(
        target: WeakHeapRef,
        holdings: Value,
        unregister_token: Option<WeakHeapRef>,
    ) -> Self {
        Self {
            target,
            holdings,
            unregister_token,
        }
    }

    #[inline]
    pub(crate) const fn target(self) -> WeakHeapRef {
        self.target
    }

    #[inline]
    pub(crate) const fn holdings(self) -> Value {
        self.holdings
    }

    #[inline]
    pub(crate) const fn unregister_token(self) -> Option<WeakHeapRef> {
        self.unregister_token
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FinalizationRegistryState {
    cells: Vec<FinalizationCell>,
    pending_holdings: Vec<Value>,
    cleanup_pending: bool,
    cleanup_active: bool,
}

impl FinalizationRegistryState {
    #[inline]
    pub(crate) const fn new() -> Self {
        Self {
            cells: Vec::new(),
            pending_holdings: Vec::new(),
            cleanup_pending: false,
            cleanup_active: false,
        }
    }

    #[inline]
    pub(crate) fn cells(&self) -> &[FinalizationCell] {
        &self.cells
    }

    #[inline]
    pub(crate) fn pending_holdings(&self) -> &[Value] {
        &self.pending_holdings
    }

    #[inline]
    pub(crate) const fn cleanup_pending(&self) -> bool {
        self.cleanup_pending
    }

    #[inline]
    pub(crate) const fn cleanup_active(&self) -> bool {
        self.cleanup_active
    }

    pub(crate) fn register(
        &mut self,
        target: WeakHeapRef,
        holdings: Value,
        unregister_token: Option<WeakHeapRef>,
    ) {
        self.cells
            .push(FinalizationCell::new(target, holdings, unregister_token));
    }

    pub(crate) fn unregister(&mut self, unregister_token: WeakHeapRef) -> bool {
        let original_len = self.cells.len();
        self.cells
            .retain(|cell| cell.unregister_token() != Some(unregister_token));
        original_len != self.cells.len()
    }

    pub(crate) fn queue_dead_targets(
        &mut self,
        mut is_live: impl FnMut(WeakHeapRef) -> bool,
    ) -> usize {
        let mut queued = 0;
        let mut survivors = Vec::with_capacity(self.cells.len());
        for cell in self.cells.drain(..) {
            if is_live(cell.target()) {
                survivors.push(cell);
            } else {
                self.pending_holdings.push(cell.holdings());
                queued += 1;
            }
        }
        self.cells = survivors;
        self.cleanup_pending = !self.pending_holdings.is_empty();
        queued
    }

    pub(crate) fn take_pending_holdings(&mut self) -> Vec<Value> {
        self.cleanup_pending = false;
        std::mem::take(&mut self.pending_holdings)
    }

    #[inline]
    pub(crate) const fn set_cleanup_active(&mut self, active: bool) {
        self.cleanup_active = active;
    }
}
