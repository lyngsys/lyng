use std::collections::BTreeSet;

pub const CARD_SIZE_BYTES: usize = 512;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CardDomain {
    String,
    Symbol,
    Object,
    FunctionPayload,
    ValueCell,
    ObjectSlots,
    SuspendedExecution,
    SuspendedRegisters,
    Environment,
    EnvironmentSlots,
    CodeSlots,
    Realm,
    Shape,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CardKey {
    domain: CardDomain,
    index: usize,
}

#[derive(Clone, Debug, Default)]
pub struct CardTable {
    dirty: BTreeSet<CardKey>,
    dirtied_since_minor: usize,
}

impl CardKey {
    #[inline]
    pub(crate) const fn new(domain: CardDomain, index: usize) -> Self {
        Self { domain, index }
    }

    #[inline]
    pub(crate) const fn domain(self) -> CardDomain {
        self.domain
    }

    #[inline]
    pub(crate) const fn index(self) -> usize {
        self.index
    }
}

impl CardTable {
    #[inline]
    pub(crate) fn mark(&mut self, key: CardKey) {
        self.dirty.insert(key);
        self.dirtied_since_minor += 1;
    }

    #[inline]
    pub(crate) const fn dirtied_since_minor(&self) -> usize {
        self.dirtied_since_minor
    }

    pub(crate) fn take_dirty(&mut self) -> Vec<CardKey> {
        let dirty = self.dirty.iter().copied().collect();
        self.dirty.clear();
        self.dirtied_since_minor = 0;
        dirty
    }
}
