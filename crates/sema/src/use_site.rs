//! Use-site resolution: tracks every identifier reference and its resolution.

use std::collections::HashMap;

use lyng_ast::ExprId;
use lyng_common::AtomId;

use crate::ids::{ScopeId, SemanticBindingId, UseSiteId};

/// How a name reference was resolved.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResolutionKind {
    /// Resolved to a local binding within the same function.
    Local,
    /// Resolved to a binding captured from an outer function.
    Captured,
    /// Resolved to a global name (not lexically defined).
    Global,
    /// Cannot be resolved statically due to `eval` or `with` poisoning.
    Dynamic,
    /// Could not be resolved to any binding.
    Unresolved,
}

/// A single use-site record: an identifier reference and how it resolved.
#[derive(Clone, Debug)]
pub struct UseSiteRecord {
    /// The identifier expression node that produced this use site, when the
    /// use originates from an expression rather than a binding pattern.
    pub expr: Option<ExprId>,
    /// The name being referenced.
    pub name: AtomId,
    /// The scope in which this use site appears.
    pub scope: ScopeId,
    /// The binding this use resolves to, or `None` for global/unresolved.
    pub resolved_binding: Option<SemanticBindingId>,
    /// How the name was resolved.
    pub resolution_kind: ResolutionKind,
}

/// The use-site table: indexed by `UseSiteId`.
///
/// `by_expr` mirrors the records whose `expr` is `Some`, giving `for_expr`
/// O(1) lookup. The bytecode lowerer calls `for_expr` once per identifier
/// reference, so a linear scan here makes compilation O(N^2) in the number of
/// references; the index keeps it linear. Each identifier node produces at most
/// one record (see `record_use`), so the mapping is a bijection. Post-analysis
/// mutators (`as_mut_slice`) only rewrite resolution fields, never `expr`, so
/// the index stays valid.
#[derive(Clone, Debug, Default)]
pub struct UseSiteTable {
    records: Vec<UseSiteRecord>,
    by_expr: HashMap<ExprId, UseSiteId>,
}

impl UseSiteTable {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            by_expr: HashMap::new(),
        }
    }

    /// Allocates a new use-site record and returns its ID.
    pub fn alloc(&mut self, record: UseSiteRecord) -> UseSiteId {
        let id = UseSiteId::new(self.records.len() as u32);
        if let Some(expr) = record.expr {
            self.by_expr.insert(expr, id);
        }
        self.records.push(record);
        id
    }

    /// Returns a reference to the use-site record.
    #[inline]
    pub fn get(&self, id: UseSiteId) -> &UseSiteRecord {
        &self.records[id.raw() as usize]
    }

    #[inline]
    pub fn for_expr(&self, expr: ExprId) -> Option<&UseSiteRecord> {
        let id = *self.by_expr.get(&expr)?;
        Some(&self.records[id.raw() as usize])
    }

    /// Returns the number of use sites.
    #[inline]
    pub const fn len(&self) -> usize {
        self.records.len()
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Returns a slice of all use-site records.
    pub fn as_slice(&self) -> &[UseSiteRecord] {
        &self.records
    }

    /// Returns a mutable slice of all use-site records.
    pub fn as_mut_slice(&mut self) -> &mut [UseSiteRecord] {
        &mut self.records
    }
}
