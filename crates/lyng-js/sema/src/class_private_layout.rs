//! Stable class-private layout metadata keyed by class body.

use lyng_js_ast::{ClassElementId, NodeList};
use lyng_js_common::{AtomId, Span};
use std::collections::HashMap;

use crate::ids::ScopeId;

/// Runtime-relevant kind for one private class element.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ClassPrivateElementKind {
    Field,
    Method,
    Getter,
    Setter,
}

/// One source-ordered private element entry within a class body.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ClassPrivateElementRecord {
    name: AtomId,
    is_static: bool,
    kind: ClassPrivateElementKind,
    span: Span,
}

impl ClassPrivateElementRecord {
    #[inline]
    pub const fn new(
        name: AtomId,
        is_static: bool,
        kind: ClassPrivateElementKind,
        span: Span,
    ) -> Self {
        Self {
            name,
            is_static,
            kind,
            span,
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

    #[inline]
    pub const fn span(self) -> Span {
        self.span
    }
}

/// Frozen private layout metadata for one class body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassPrivateLayoutRecord {
    body: NodeList<ClassElementId>,
    span: Span,
    scope: ScopeId,
    entries: Vec<ClassPrivateElementRecord>,
}

impl ClassPrivateLayoutRecord {
    #[inline]
    pub const fn new(
        body: NodeList<ClassElementId>,
        span: Span,
        scope: ScopeId,
        entries: Vec<ClassPrivateElementRecord>,
    ) -> Self {
        Self {
            body,
            span,
            scope,
            entries,
        }
    }

    #[inline]
    pub const fn body(&self) -> NodeList<ClassElementId> {
        self.body
    }

    #[inline]
    pub const fn span(&self) -> Span {
        self.span
    }

    #[inline]
    pub const fn scope(&self) -> ScopeId {
        self.scope
    }

    #[inline]
    pub fn entries(&self) -> &[ClassPrivateElementRecord] {
        &self.entries
    }
}

/// Table of class-private layout records keyed by class body.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ClassPrivateLayoutTable {
    records: Vec<ClassPrivateLayoutRecord>,
    by_body: HashMap<NodeList<ClassElementId>, usize>,
    by_body_and_span: HashMap<(NodeList<ClassElementId>, Span), usize>,
    by_scope: HashMap<ScopeId, usize>,
}

impl ClassPrivateLayoutTable {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn alloc(&mut self, record: ClassPrivateLayoutRecord) {
        let index = self.records.len();
        self.by_body.insert(record.body(), index);
        self.by_body_and_span
            .insert((record.body(), record.span()), index);
        self.by_scope.insert(record.scope(), index);
        self.records.push(record);
    }

    pub fn alloc_imported(
        &mut self,
        scope: ScopeId,
        span: Span,
        entries: Vec<ClassPrivateElementRecord>,
    ) {
        let index = self.records.len();
        self.by_scope.insert(scope, index);
        self.records.push(ClassPrivateLayoutRecord::new(
            NodeList::EMPTY,
            span,
            scope,
            entries,
        ));
    }

    #[inline]
    pub fn get(&self, body: NodeList<ClassElementId>) -> Option<&ClassPrivateLayoutRecord> {
        let index = self.by_body.get(&body).copied()?;
        self.records.get(index)
    }

    #[inline]
    pub fn get_with_span(
        &self,
        body: NodeList<ClassElementId>,
        span: Span,
    ) -> Option<&ClassPrivateLayoutRecord> {
        let index = self.by_body_and_span.get(&(body, span)).copied()?;
        self.records.get(index)
    }

    #[inline]
    pub fn get_by_scope(&self, scope: ScopeId) -> Option<&ClassPrivateLayoutRecord> {
        let index = self.by_scope.get(&scope).copied()?;
        self.records.get(index)
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
    pub fn as_slice(&self) -> &[ClassPrivateLayoutRecord] {
        &self.records
    }
}
