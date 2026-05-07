//! Binding table: records every declaration-level binding and its metadata.

use lyng_js_common::AtomId;

use crate::ids::{ScopeId, SemanticBindingId};

/// The kind of declaration that introduced a binding.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DeclarationKind {
    /// `var` declaration.
    Var,
    /// `let` declaration.
    Let,
    /// `const` declaration.
    Const,
    /// `using` declaration.
    Using,
    /// `await using` declaration.
    AwaitUsing,
    /// Function declaration (`function f() {}`).
    Function,
    /// Class declaration (`class C {}`).
    Class,
    /// Function parameter.
    Parameter,
    /// `catch` clause parameter.
    CatchParam,
    /// Import binding (`import { x } from "mod"`).
    Import,
    /// Function name binding (the name of a function expression bound in its own scope).
    FunctionName,
    /// Class name binding (the name of a class expression bound in its own scope).
    ClassName,
}

impl DeclarationKind {
    /// Returns true if this binding kind has a temporal dead zone.
    pub const fn has_tdz(self) -> bool {
        matches!(
            self,
            Self::Let
                | Self::Const
                | Self::Using
                | Self::AwaitUsing
                | Self::Class
                | Self::ClassName
        )
    }

    /// Returns true if this is a lexical (block-scoped) binding kind.
    pub const fn is_lexical(self) -> bool {
        matches!(
            self,
            Self::Let
                | Self::Const
                | Self::Using
                | Self::AwaitUsing
                | Self::Class
                | Self::Import
                | Self::ClassName
        )
    }

    /// Returns true if this binding is hoisted to function scope.
    pub const fn is_hoisted(self) -> bool {
        matches!(self, Self::Var | Self::Function)
    }
}

/// How a binding is stored at runtime.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StorageClass {
    /// Uncaptured local that lives in registers / stack frame.
    FrameLocal,
    /// Captured variable stored in an environment record (heap closure).
    EnvironmentSlot,
    /// A global-scope `var`/`function` stored as a property of the global object.
    GlobalName,
    /// A variable under `with` or direct `eval` poisoning that requires
    /// dynamic lookup at runtime.
    DynamicLookup,
    /// A direct-eval var/function binding that already exists in the caller's
    /// variable environment. Declaration writes target the variable
    /// environment, while identifier references still use dynamic lookup.
    DynamicVariableLookup,
}

/// A single binding record in the binding table.
#[derive(Clone, Debug)]
pub struct BindingRecord {
    /// The name of this binding.
    pub name: AtomId,
    /// What kind of declaration introduced this binding.
    pub kind: DeclarationKind,
    /// The scope that owns this binding.
    pub scope: ScopeId,
    /// Whether this binding is captured by an inner function.
    pub is_captured: bool,
    /// Whether this binding needs to live in an environment record.
    pub needs_environment: bool,
    /// How this binding is stored at runtime.
    pub storage_class: StorageClass,
    /// Whether this binding has a temporal dead zone (let/const/class).
    pub has_tdz: bool,
    /// The environment slot index within the owning scope, if environment-stored.
    pub slot_index: Option<u32>,
}

/// The binding table: a vec of binding records indexed by `SemanticBindingId`.
#[derive(Clone, Debug, Default)]
pub struct BindingTable {
    bindings: Vec<BindingRecord>,
}

impl BindingTable {
    pub const fn new() -> Self {
        Self {
            bindings: Vec::new(),
        }
    }

    /// Allocates a new binding and returns its ID.
    pub fn alloc(&mut self, record: BindingRecord) -> SemanticBindingId {
        let id = SemanticBindingId::new(self.bindings.len() as u32);
        self.bindings.push(record);
        id
    }

    /// Returns a reference to the binding record.
    #[inline]
    pub fn get(&self, id: SemanticBindingId) -> &BindingRecord {
        &self.bindings[id.raw() as usize]
    }

    /// Returns a mutable reference to the binding record.
    #[inline]
    pub fn get_mut(&mut self, id: SemanticBindingId) -> &mut BindingRecord {
        &mut self.bindings[id.raw() as usize]
    }

    /// Returns the number of bindings.
    #[inline]
    pub const fn len(&self) -> usize {
        self.bindings.len()
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// Returns a slice of all binding records.
    pub fn as_slice(&self) -> &[BindingRecord] {
        &self.bindings
    }
}
