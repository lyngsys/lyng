//! Private-name analysis: track definitions and resolve uses of `#name`.

use lyng_js_common::{AtomId, Span};

use crate::ids::{PrivateNameId, ScopeId};

/// A private name definition within a class body.
#[derive(Clone, Debug)]
pub struct PrivateNameRecord {
    /// The name atom (without the `#` prefix).
    pub name: AtomId,
    /// The class-body scope where this private name is defined.
    pub scope: ScopeId,
    /// The span of the definition.
    pub span: Span,
}

/// The private-name table: indexed by `PrivateNameId`.
#[derive(Clone, Debug, Default)]
pub struct PrivateNameTable {
    records: Vec<PrivateNameRecord>,
}

impl PrivateNameTable {
    pub const fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    /// Allocates a new private name record and returns its ID.
    pub fn alloc(&mut self, record: PrivateNameRecord) -> PrivateNameId {
        let id = PrivateNameId::new(self.records.len() as u32);
        self.records.push(record);
        id
    }

    /// Returns a reference to the private name record.
    #[inline]
    pub fn get(&self, id: PrivateNameId) -> &PrivateNameRecord {
        &self.records[id.raw() as usize]
    }

    /// Returns the number of private name records.
    #[inline]
    pub const fn len(&self) -> usize {
        self.records.len()
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Returns a slice of all private name records.
    pub fn as_slice(&self) -> &[PrivateNameRecord] {
        &self.records
    }
}
