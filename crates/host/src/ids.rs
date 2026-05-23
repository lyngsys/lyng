use std::fmt;
use std::num::NonZeroU32;

macro_rules! define_host_id {
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

define_host_id!(
    /// Host-assigned identifier for an engine agent.
    HostAgentId
);
define_host_id!(
    /// Host-assigned identifier for a thread that runs an agent.
    HostThreadId
);
define_host_id!(
    /// Host-observable identifier for a queued job.
    HostJobId
);
define_host_id!(
    /// Opaque host boundary handle for a detachable `ArrayBuffer`.
    HostTransferredBufferId
);
define_host_id!(
    /// Opaque host boundary handle for a shareable `SharedArrayBuffer` backing store.
    HostSharedBufferId
);
