//! Typed 32-bit IDs for semantic analysis side tables.

use std::fmt;

macro_rules! define_sema_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name(u32);

        impl $name {
            #[inline]
            pub const fn new(raw: u32) -> Self {
                Self(raw)
            }

            #[inline]
            pub const fn raw(self) -> u32 {
                self.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }
    };
}

define_sema_id!(
    /// Identifies a scope in the scope table.
    ScopeId
);

define_sema_id!(
    /// Identifies a binding in the binding table.
    SemanticBindingId
);

define_sema_id!(
    /// Identifies a function-level semantic record (1:1 with FunctionId).
    FunctionSemaId
);

define_sema_id!(
    /// Identifies a name use site.
    UseSiteId
);

define_sema_id!(
    /// Identifies a private name definition.
    PrivateNameId
);
