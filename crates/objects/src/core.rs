use lyng_gc::ObjectSlotsRef;

/// Coarse object kinds frozen by the runtime-substrate design.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ObjectKind {
    Ordinary,
    Function,
    Proxy,
}

/// Compact flag summary for the object hot header.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct ObjectFlags(pub(crate) u16);

impl ObjectFlags {
    pub const EXTENSIBLE: Self = Self(1 << 0);
    pub const SEALED: Self = Self(1 << 1);
    pub const FROZEN: Self = Self(1 << 2);
    pub const NAMED_PROPERTIES_DICTIONARY: Self = Self(1 << 3);
    pub const ENGINE_ARRAY: Self = Self(1 << 4);
    pub const ARGUMENTS_OBJECT: Self = Self(1 << 5);
    pub const IMMUTABLE_PROTOTYPE: Self = Self(1 << 6);
    pub const ERROR_OBJECT: Self = Self(1 << 7);
    pub const IS_HTMLDDA: Self = Self(1 << 8);
    pub const CELL_BACKED_DICTIONARY: Self = Self(1 << 9);

    #[inline]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn extensible() -> Self {
        Self::EXTENSIBLE
    }

    /// Returns the raw 16-bit packed representation. Used by the Phase 3d
    /// `KeyedDenseIndexHandler` IC shortcut to compare against a cached
    /// flags snapshot in a single u32 cmp.
    #[inline]
    pub const fn bits(self) -> u16 {
        self.0
    }

    /// Construct from a raw 16-bit packed representation. Pairs with
    /// [`Self::bits`].
    #[inline]
    pub const fn from_bits(bits: u16) -> Self {
        Self(bits)
    }

    #[inline]
    pub const fn contains(self, flags: Self) -> bool {
        self.0 & flags.0 == flags.0
    }

    #[inline]
    pub const fn union(self, flags: Self) -> Self {
        Self(self.0 | flags.0)
    }

    #[inline]
    pub const fn without(self, flags: Self) -> Self {
        Self(self.0 & !flags.0)
    }

    #[inline]
    pub const fn is_extensible(self) -> bool {
        self.contains(Self::EXTENSIBLE)
    }

    #[inline]
    pub const fn is_sealed_summary(self) -> bool {
        self.contains(Self::SEALED)
    }

    #[inline]
    pub const fn is_frozen_summary(self) -> bool {
        self.contains(Self::FROZEN)
    }

    #[inline]
    pub const fn uses_named_property_dictionary(self) -> bool {
        self.contains(Self::NAMED_PROPERTIES_DICTIONARY)
    }

    #[inline]
    pub const fn uses_cell_backed_dictionary(self) -> bool {
        self.contains(Self::CELL_BACKED_DICTIONARY)
    }

    #[inline]
    pub const fn is_engine_array(self) -> bool {
        self.contains(Self::ENGINE_ARRAY)
    }

    #[inline]
    pub const fn is_arguments_object(self) -> bool {
        self.contains(Self::ARGUMENTS_OBJECT)
    }

    #[inline]
    pub const fn has_immutable_prototype(self) -> bool {
        self.contains(Self::IMMUTABLE_PROTOTYPE)
    }

    #[inline]
    pub const fn is_error_object(self) -> bool {
        self.contains(Self::ERROR_OBJECT)
    }

    #[inline]
    pub const fn is_html_dda(self) -> bool {
        self.contains(Self::IS_HTMLDDA)
    }
}

/// Named-slot storage reference in the object hot header.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NamedSlotStorageRef(ObjectSlotsRef);

impl NamedSlotStorageRef {
    #[inline]
    pub const fn new(raw: ObjectSlotsRef) -> Self {
        Self(raw)
    }

    #[inline]
    pub const fn raw(self) -> ObjectSlotsRef {
        self.0
    }
}

/// Indexed-element storage reference in the object hot header.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ElementStorageRef(ObjectSlotsRef);

impl ElementStorageRef {
    #[inline]
    pub const fn new(raw: ObjectSlotsRef) -> Self {
        Self(raw)
    }

    #[inline]
    pub const fn raw(self) -> ObjectSlotsRef {
        self.0
    }
}

/// Public error surface for centralized object internal-method dispatch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InternalMethodError {
    MissingObject,
    MissingClassRecord,
    CorruptObjectState,
    ReferenceError,
    InvalidDescriptor,
    InvalidPrivateElement,
    InvalidPrivateBrand,
    DuplicatePrivateElement,
    ObjectNotExtensible,
    RangeError,
    AccessorCallPending,
    MissingFunctionPayload,
    MissingNativeHandler,
    NotCallable,
    NotConstructible,
    RevokedProxy,
    BytecodeDispatchPending,
}

/// Result type used by internal-method entrypoints.
pub type InternalMethodResult<T> = Result<T, InternalMethodError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_backed_dictionary_flag_roundtrips() {
        let flags = ObjectFlags::empty().union(ObjectFlags::CELL_BACKED_DICTIONARY);
        assert!(flags.uses_cell_backed_dictionary());
        assert!(!ObjectFlags::empty().uses_cell_backed_dictionary());
        // Independent from the dictionary flag:
        assert!(
            !ObjectFlags::CELL_BACKED_DICTIONARY.contains(ObjectFlags::NAMED_PROPERTIES_DICTIONARY)
        );
    }
}
