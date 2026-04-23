//! Scope table: records the hierarchical scope structure of the program.
//!
//! Each scope has a kind, a parent link, an optional owning function,
//! strictness flag, and lists of child scopes and bindings.

use crate::ids::{FunctionSemaId, ScopeId, SemanticBindingId};

/// The kind of scope.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ScopeKind {
    /// The global/script top-level scope.
    Global,
    /// A module top-level scope (always strict).
    Module,
    /// A function body scope.
    Function,
    /// A block scope (`{ }`, `if`, etc.).
    Block,
    /// A `catch` clause scope.
    Catch,
    /// A `with` statement scope.
    With,
    /// A class body scope (for private names and static elements).
    ClassBody,
    /// A `for`/`for-in`/`for-of` loop head scope (for `let`/`const` bindings).
    ForLoop,
    /// A `switch` body scope.
    Switch,
    /// A parameter scope, distinct from the body when parameters are non-simple.
    Parameter,
}

/// A single scope record in the scope table.
///
/// Bindings and children use `Vec` because sema adds to them incrementally
/// during the walk (e.g., var hoisting adds bindings to ancestor scopes).
/// A `ListArena`-backed design would require a freeze step, which is complex
/// given that scopes can receive bindings after their subtree has been walked.
/// The per-scope Vec allocation cost is modest (typical programs have < 200
/// scopes) and acceptable for Phase 1.
#[derive(Clone, Debug)]
pub struct ScopeRecord {
    /// The parent scope, or `None` for the root.
    pub parent: Option<ScopeId>,
    /// The kind of this scope.
    pub kind: ScopeKind,
    /// The function that owns this scope, if any.
    pub owning_function: Option<FunctionSemaId>,
    /// Whether this scope is in strict mode.
    pub strict: bool,
    /// Whether a direct `eval()` call exists in this scope.
    pub has_eval: bool,
    /// Whether a `with` statement exists in this scope.
    pub has_with: bool,
    /// Whether this scope needs an environment allocation at runtime.
    pub needs_environment: bool,
    /// Bindings declared in this scope.
    pub bindings: Vec<SemanticBindingId>,
    /// Child scopes.
    pub children: Vec<ScopeId>,
}

/// The scope table: a vec of scope records indexed by `ScopeId`.
#[derive(Clone, Debug, Default)]
pub struct ScopeTable {
    scopes: Vec<ScopeRecord>,
}

impl ScopeTable {
    pub fn new() -> Self {
        Self { scopes: Vec::new() }
    }

    /// Allocates a new scope and returns its ID.
    pub fn alloc(&mut self, record: ScopeRecord) -> ScopeId {
        let id = ScopeId::new(self.scopes.len() as u32);
        self.scopes.push(record);
        id
    }

    /// Returns a reference to the scope record.
    #[inline]
    pub fn get(&self, id: ScopeId) -> &ScopeRecord {
        &self.scopes[id.raw() as usize]
    }

    /// Returns a mutable reference to the scope record.
    #[inline]
    pub fn get_mut(&mut self, id: ScopeId) -> &mut ScopeRecord {
        &mut self.scopes[id.raw() as usize]
    }

    /// Returns the number of scopes.
    #[inline]
    pub fn len(&self) -> usize {
        self.scopes.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.scopes.is_empty()
    }

    /// Returns a slice of all scope records.
    pub fn as_slice(&self) -> &[ScopeRecord] {
        &self.scopes
    }
}
