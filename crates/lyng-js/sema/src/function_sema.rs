//! Function-level semantic records: one per `FunctionId` in the AST.
//!
//! Tracks per-function metadata such as strictness, scope linkage,
//! environment needs, and special-name references.

use lyng_js_ast::FunctionId;

use crate::ids::{FunctionSemaId, ScopeId, SemanticBindingId};

/// A function-level semantic record.
#[derive(Clone, Debug)]
pub struct FunctionSemaRecord {
    /// The AST function node this record corresponds to.
    pub function_id: FunctionId,
    /// Whether this function is in strict mode.
    pub strict: bool,
    /// The root scope of this function's body.
    pub scope_root: ScopeId,
    /// When parameters are non-simple, a distinct parameter scope.
    pub param_scope: Option<ScopeId>,
    /// Whether this function needs an environment allocation.
    pub needs_environment: bool,
    /// Whether this function contains a direct `eval()` call.
    pub has_eval: bool,
    /// Whether this function contains a `with` statement.
    pub has_with: bool,
    /// Whether this function uses the `arguments` object.
    /// Note: detection deferred to Phase 4 (requires runtime knowledge of
    /// whether `arguments` is shadowed by a binding).
    pub needs_arguments: bool,
    /// Whether this function references `super`.
    pub references_super: bool,
    /// Whether this function references `new.target`.
    pub references_new_target: bool,
    /// Whether this function references `this`.
    pub references_this: bool,
    /// Whether this function contains `await`.
    pub has_await: bool,
    /// Whether this function contains `yield`.
    pub has_yield: bool,
    /// Bindings captured from outer scopes by this function.
    pub captures: Vec<SemanticBindingId>,
}

/// The function sema table: indexed by `FunctionSemaId`.
#[derive(Clone, Debug, Default)]
pub struct FunctionSemaTable {
    records: Vec<FunctionSemaRecord>,
}

impl FunctionSemaTable {
    pub const fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    /// Allocates a new function sema record and returns its ID.
    pub fn alloc(&mut self, record: FunctionSemaRecord) -> FunctionSemaId {
        let id = FunctionSemaId::new(self.records.len() as u32);
        self.records.push(record);
        id
    }

    /// Returns a reference to the function sema record.
    #[inline]
    pub fn get(&self, id: FunctionSemaId) -> &FunctionSemaRecord {
        &self.records[id.raw() as usize]
    }

    /// Returns a mutable reference to the function sema record.
    #[inline]
    pub fn get_mut(&mut self, id: FunctionSemaId) -> &mut FunctionSemaRecord {
        &mut self.records[id.raw() as usize]
    }

    /// Returns the number of function records.
    #[inline]
    pub const fn len(&self) -> usize {
        self.records.len()
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Returns a slice of all function sema records.
    pub fn as_slice(&self) -> &[FunctionSemaRecord] {
        &self.records
    }
}
