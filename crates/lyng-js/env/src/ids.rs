use super::{EnvironmentLayoutId, EnvironmentRef, RealmRef};
use std::fmt;
use std::num::NonZeroU32;

macro_rules! define_runtime_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[repr(transparent)]
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(NonZeroU32);

        impl $name {
            #[inline]
            pub const fn new(raw: NonZeroU32) -> Self {
                Self(raw)
            }

            #[inline]
            pub const fn from_raw(raw: u32) -> Option<Self> {
                match NonZeroU32::new(raw) {
                    Some(raw) => Some(Self(raw)),
                    None => None,
                }
            }

            #[inline]
            pub const fn raw(self) -> NonZeroU32 {
                self.0
            }

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
    /// Engine-local identifier for one agent in an `AgentCluster`.
    AgentId
);
define_runtime_id!(
    /// Engine-local identifier for one queued runtime job.
    JobId
);
define_runtime_id!(
    /// Engine-local identifier for one promise side-table record.
    PromiseId
);
define_runtime_id!(
    /// Engine-local identifier for one promise reaction record.
    PromiseReactionId
);
define_runtime_id!(
    /// Engine-local identifier for one promise capability record.
    PromiseCapabilityId
);
define_runtime_id!(
    /// Engine-local identifier for one promise resolving-function record.
    PromiseResolvingFunctionId
);
define_runtime_id!(
    /// Engine-local identifier for one promise finally-function record.
    PromiseFinallyFunctionId
);
define_runtime_id!(
    /// Engine-local identifier for one promise combinator shared-state record.
    PromiseCombinatorId
);
define_runtime_id!(
    /// Engine-local identifier for one promise combinator element-function record.
    PromiseCombinatorElementId
);
define_runtime_id!(
    /// Engine-local identifier for one disposal-capability side-table record.
    DisposalCapabilityId
);
define_runtime_id!(
    /// Engine-local identifier for one async-disposal continuation record.
    AsyncDisposalOperationId
);
define_runtime_id!(
    /// Engine-local identifier for one async-disposal resume-function record.
    AsyncDisposalResumeId
);

#[inline]
pub const fn agent_index(id: AgentId) -> usize {
    (id.get() - 1) as usize
}

#[inline]
pub const fn layout_index(id: EnvironmentLayoutId) -> usize {
    (id.get() - 1) as usize
}

#[inline]
pub const fn environment_index(id: EnvironmentRef) -> usize {
    (id.get() - 1) as usize
}

#[inline]
pub const fn realm_index(id: RealmRef) -> usize {
    (id.get() - 1) as usize
}
