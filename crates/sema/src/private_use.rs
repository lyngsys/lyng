//! Resolved private-name uses keyed by expression site.

use std::collections::HashMap;

use lyng_ast::ExprId;
use lyng_common::AtomId;

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
///
/// `by_expr` indexes `records` by `expr` so `for_expr` is O(1); the bytecode
/// lowerer calls it once per private-name reference, so a linear scan would
/// make compilation O(N^2) in the number of references. Each private-name node
/// produces at most one record, so the mapping is a bijection.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PrivateUseTable {
    records: Vec<PrivateUseRecord>,
    by_expr: HashMap<ExprId, usize>,
}

impl PrivateUseTable {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn alloc(&mut self, record: PrivateUseRecord) {
        self.by_expr.insert(record.expr(), self.records.len());
        self.records.push(record);
    }

    #[inline]
    pub fn for_expr(&self, expr: ExprId) -> Option<&PrivateUseRecord> {
        let index = *self.by_expr.get(&expr)?;
        Some(&self.records[index])
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.records.len()
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    #[inline]
    pub fn as_slice(&self) -> &[PrivateUseRecord] {
        &self.records
    }
}
