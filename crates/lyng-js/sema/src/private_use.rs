//! Resolved private-name uses keyed by expression site.

use lyng_js_ast::ExprId;
use lyng_js_common::AtomId;

use crate::ids::ScopeId;

/// One resolved private-name use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrivateUseRecord {
    expr: ExprId,
    name: AtomId,
    defining_scope: ScopeId,
    class_depth: u16,
}

impl PrivateUseRecord {
    #[inline]
    pub const fn new(
        expr: ExprId,
        name: AtomId,
        defining_scope: ScopeId,
        class_depth: u16,
    ) -> Self {
        Self {
            expr,
            name,
            defining_scope,
            class_depth,
        }
    }

    #[inline]
    pub const fn expr(self) -> ExprId {
        self.expr
    }

    #[inline]
    pub const fn name(self) -> AtomId {
        self.name
    }

    #[inline]
    pub const fn defining_scope(self) -> ScopeId {
        self.defining_scope
    }

    #[inline]
    pub const fn class_depth(self) -> u16 {
        self.class_depth
    }
}

/// Table of resolved private-name uses.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PrivateUseTable {
    records: Vec<PrivateUseRecord>,
}

impl PrivateUseTable {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn alloc(&mut self, record: PrivateUseRecord) {
        self.records.push(record);
    }

    #[inline]
    pub fn for_expr(&self, expr: ExprId) -> Option<&PrivateUseRecord> {
        self.records.iter().find(|record| record.expr() == expr)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.records.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    #[inline]
    pub fn as_slice(&self) -> &[PrivateUseRecord] {
        &self.records
    }
}
