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
pub(crate) struct WeakMapEntry {
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

    #[inline]
    pub(crate) fn set_value(&mut self, value: Value) {
        self.value = value;
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct WeakMapState {
    entries: Vec<WeakMapEntry>,
}

impl WeakMapState {
    #[inline]
    pub(crate) const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    #[inline]
    pub(crate) fn entries(&self) -> &[WeakMapEntry] {
        &self.entries
    }

    pub(crate) fn get(&self, key: WeakHeapRef) -> Option<Value> {
        self.entries
            .iter()
            .find_map(|entry| (entry.key() == key).then_some(entry.value()))
    }

    pub(crate) fn set(&mut self, key: WeakHeapRef, value: Value) {
        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.key() == key) {
            entry.set_value(value);
            return;
        }
        self.entries.push(WeakMapEntry::new(key, value));
    }

    pub(crate) fn delete(&mut self, key: WeakHeapRef) -> bool {
        let original_len = self.entries.len();
        self.entries.retain(|entry| entry.key() != key);
        original_len != self.entries.len()
    }

    pub(crate) fn retain_live_keys(&mut self, mut is_live: impl FnMut(WeakHeapRef) -> bool) {
        self.entries.retain(|entry| is_live(entry.key()));
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct WeakSetState {
    entries: Vec<WeakHeapRef>,
}

impl WeakSetState {
    #[inline]
    pub(crate) const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    #[inline]
    pub(crate) fn contains(&self, value: WeakHeapRef) -> bool {
        self.entries.contains(&value)
    }

    pub(crate) fn insert(&mut self, value: WeakHeapRef) {
        if !self.contains(value) {
            self.entries.push(value);
        }
    }

    pub(crate) fn delete(&mut self, value: WeakHeapRef) -> bool {
        let original_len = self.entries.len();
        self.entries.retain(|entry| *entry != value);
        original_len != self.entries.len()
    }

    pub(crate) fn retain_live_values(&mut self, mut is_live: impl FnMut(WeakHeapRef) -> bool) {
        self.entries.retain(|entry| is_live(*entry));
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct WeakRefState {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FinalizationCell {
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
pub(crate) struct FinalizationRegistryState {
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
    pub(crate) fn set_cleanup_active(&mut self, active: bool) {
        self.cleanup_active = active;
    }
}
