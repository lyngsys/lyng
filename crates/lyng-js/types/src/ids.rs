use std::fmt;
use std::num::NonZeroU32;

macro_rules! define_runtime_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[repr(transparent)]
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(NonZeroU32);

        impl $name {
            /// Creates a new typed ID from a non-zero raw payload.
            #[inline]
            pub const fn new(raw: NonZeroU32) -> Self {
                Self(raw)
            }

            /// Attempts to construct a typed ID from a raw payload.
            #[inline]
            pub const fn from_raw(raw: u32) -> Option<Self> {
                match NonZeroU32::new(raw) {
                    Some(raw) => Some(Self(raw)),
                    None => None,
                }
            }

            /// Returns the underlying non-zero payload.
            #[inline]
            pub const fn raw(self) -> NonZeroU32 {
                self.0
            }

            /// Returns the underlying payload as a `u32`.
            #[inline]
            pub const fn get(self) -> u32 {
                self.0.get()
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), self.get())
            }
        }
    };
}

define_runtime_id!(
    /// Typed handle for an object record.
    ObjectRef
);

define_runtime_id!(
    /// Typed handle for a runtime string record.
    StringRef
);

define_runtime_id!(
    /// Typed handle for a symbol record.
    SymbolRef
);

define_runtime_id!(
    /// Typed handle for a bigint record.
    BigIntRef
);

define_runtime_id!(
    /// Typed handle for an environment record.
    EnvironmentRef
);

define_runtime_id!(
    /// Typed handle for a code template or code record.
    CodeRef
);

define_runtime_id!(
    /// Typed handle for one suspended bytecode execution snapshot.
    SuspendedExecutionRef
);

define_runtime_id!(
    /// Typed handle for one binary-data backing-store record.
    BackingStoreRef
);

define_runtime_id!(
    /// Typed handle for a realm record.
    RealmRef
);

define_runtime_id!(
    /// Compact identity for a shape record.
    ShapeId
);

define_runtime_id!(
    /// Compact identity for a feedback slot.
    FeedbackSlotId
);

define_runtime_id!(
    /// Compact identity for a builtin entrypoint.
    BuiltinFunctionId
);

define_runtime_id!(
    /// Compact identity for one embedding-owned native entrypoint.
    EmbeddingFunctionId
);

/// Stable callable-entry identity shared by builtin and embedding native functions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NativeFunctionId {
    Builtin(BuiltinFunctionId),
    Embedding(EmbeddingFunctionId),
}

impl NativeFunctionId {
    #[inline]
    pub const fn builtin(entry: BuiltinFunctionId) -> Self {
        Self::Builtin(entry)
    }

    #[inline]
    pub const fn embedding(entry: EmbeddingFunctionId) -> Self {
        Self::Embedding(entry)
    }

    #[inline]
    pub const fn builtin_entry(self) -> Option<BuiltinFunctionId> {
        match self {
            Self::Builtin(entry) => Some(entry),
            Self::Embedding(_) => None,
        }
    }

    #[inline]
    pub const fn embedding_entry(self) -> Option<EmbeddingFunctionId> {
        match self {
            Self::Builtin(_) => None,
            Self::Embedding(entry) => Some(entry),
        }
    }
}

/// Typed identifier for a well-known symbol slot in the agent or realm-owned symbol set.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WellKnownSymbolId {
    HasInstance,
    IsConcatSpreadable,
    Iterator,
    AsyncIterator,
    Match,
    MatchAll,
    Replace,
    Search,
    Species,
    Split,
    ToPrimitive,
    ToStringTag,
    Unscopables,
    Dispose,
    AsyncDispose,
}

impl WellKnownSymbolId {
    pub const ALL: [Self; 15] = [
        Self::HasInstance,
        Self::IsConcatSpreadable,
        Self::Iterator,
        Self::AsyncIterator,
        Self::Match,
        Self::MatchAll,
        Self::Replace,
        Self::Search,
        Self::Species,
        Self::Split,
        Self::ToPrimitive,
        Self::ToStringTag,
        Self::Unscopables,
        Self::Dispose,
        Self::AsyncDispose,
    ];

    #[inline]
    pub const fn description(self) -> &'static str {
        match self {
            Self::HasInstance => "Symbol.hasInstance",
            Self::IsConcatSpreadable => "Symbol.isConcatSpreadable",
            Self::Iterator => "Symbol.iterator",
            Self::AsyncIterator => "Symbol.asyncIterator",
            Self::Match => "Symbol.match",
            Self::MatchAll => "Symbol.matchAll",
            Self::Replace => "Symbol.replace",
            Self::Search => "Symbol.search",
            Self::Species => "Symbol.species",
            Self::Split => "Symbol.split",
            Self::ToPrimitive => "Symbol.toPrimitive",
            Self::ToStringTag => "Symbol.toStringTag",
            Self::Unscopables => "Symbol.unscopables",
            Self::Dispose => "Symbol.dispose",
            Self::AsyncDispose => "Symbol.asyncDispose",
        }
    }
}
