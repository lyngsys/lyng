use std::fmt;
use std::num::NonZeroU32;

macro_rules! define_nonzero_id {
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

define_nonzero_id!(
    /// Compact identifier for one bytecode template in a compiled unit graph.
    BytecodeFunctionId
);

define_nonzero_id!(
    /// Runtime-owned environment-layout reference recorded in bytecode metadata.
    EnvironmentLayoutRef
);

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn bytecode_function_id_stays_compact() {
        assert_eq!(size_of::<BytecodeFunctionId>(), size_of::<u32>());
        assert_eq!(size_of::<Option<BytecodeFunctionId>>(), size_of::<u32>());
        assert_eq!(BytecodeFunctionId::from_raw(0), None);
    }

    #[test]
    fn environment_layout_ref_stays_compact() {
        assert_eq!(size_of::<EnvironmentLayoutRef>(), size_of::<u32>());
        assert_eq!(size_of::<Option<EnvironmentLayoutRef>>(), size_of::<u32>());
        assert_eq!(EnvironmentLayoutRef::from_raw(0), None);
    }
}
