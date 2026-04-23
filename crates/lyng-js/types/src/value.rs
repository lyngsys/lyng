use super::{BigIntRef, ObjectRef, StringRef, SuspendedExecutionRef, SymbolRef};
use std::fmt;

const TAG_HEADER: u64 = 0x7ff8_0000_0000_0000;
const TAG_KIND_SHIFT: u32 = 32;
const TAG_KIND_MASK: u64 = 0x0000_ffff_0000_0000;
const PAYLOAD_MASK: u64 = 0x0000_0000_ffff_ffff;
const CANONICAL_NAN_BITS: u64 = 0x7ff8_0000_0000_0000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
enum TagKind {
    Undefined = 1,
    Null = 2,
    Boolean = 3,
    Smi = 4,
    ObjectRef = 5,
    StringRef = 6,
    SymbolRef = 7,
    BigIntRef = 8,
    Sentinel = 9,
    SuspendedExecutionRef = 10,
}

impl TagKind {
    #[inline]
    const fn from_raw(raw: u16) -> Option<Self> {
        match raw {
            1 => Some(Self::Undefined),
            2 => Some(Self::Null),
            3 => Some(Self::Boolean),
            4 => Some(Self::Smi),
            5 => Some(Self::ObjectRef),
            6 => Some(Self::StringRef),
            7 => Some(Self::SymbolRef),
            8 => Some(Self::BigIntRef),
            9 => Some(Self::Sentinel),
            10 => Some(Self::SuspendedExecutionRef),
            _ => None,
        }
    }
}

/// Internal runtime-only sentinel identities carried by `Value`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum InternalSentinel {
    ArrayHole = 1,
    UninitializedLexical = 2,
    EmptyInternalSlot = 3,
    DeletedEnvironmentBinding = 4,
}

impl InternalSentinel {
    #[inline]
    pub const fn raw(self) -> u32 {
        self as u32
    }

    #[inline]
    pub const fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            1 => Some(Self::ArrayHole),
            2 => Some(Self::UninitializedLexical),
            3 => Some(Self::EmptyInternalSlot),
            4 => Some(Self::DeletedEnvironmentBinding),
            _ => None,
        }
    }
}

/// An 8-byte runtime value using a NaN-tag-space encoding family.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Value(u64);

impl Value {
    #[inline]
    const fn tagged(kind: TagKind, payload: u32) -> Self {
        Self(TAG_HEADER | ((kind as u64) << TAG_KIND_SHIFT) | payload as u64)
    }

    #[inline]
    const fn is_tagged_bits(bits: u64) -> bool {
        ((bits & TAG_HEADER) == TAG_HEADER) && Self::tag_kind_bits(bits).is_some()
    }

    #[inline]
    const fn tag_kind_bits(bits: u64) -> Option<TagKind> {
        TagKind::from_raw(((bits & TAG_KIND_MASK) >> TAG_KIND_SHIFT) as u16)
    }

    #[inline]
    const fn tag_kind(self) -> Option<TagKind> {
        Self::tag_kind_bits(self.0)
    }

    #[inline]
    const fn payload(self) -> u32 {
        (self.0 & PAYLOAD_MASK) as u32
    }

    #[inline]
    const fn has_tag(self, kind: TagKind) -> bool {
        match self.tag_kind() {
            Some(tag) => (tag as u16) == (kind as u16),
            None => false,
        }
    }

    /// Returns the raw 64-bit representation.
    #[inline]
    pub const fn bits(self) -> u64 {
        self.0
    }

    #[inline]
    pub const fn undefined() -> Self {
        Self::tagged(TagKind::Undefined, 0)
    }

    #[inline]
    pub const fn null() -> Self {
        Self::tagged(TagKind::Null, 0)
    }

    #[inline]
    pub const fn from_bool(value: bool) -> Self {
        Self::tagged(TagKind::Boolean, if value { 1 } else { 0 })
    }

    #[inline]
    pub const fn from_smi(value: i32) -> Self {
        Self::tagged(TagKind::Smi, value.cast_unsigned())
    }

    #[inline]
    pub fn from_f64(value: f64) -> Self {
        if value.is_nan() {
            Self(CANONICAL_NAN_BITS)
        } else {
            Self(value.to_bits())
        }
    }

    #[inline]
    pub const fn from_object_ref(value: ObjectRef) -> Self {
        Self::tagged(TagKind::ObjectRef, value.get())
    }

    #[inline]
    pub const fn from_string_ref(value: StringRef) -> Self {
        Self::tagged(TagKind::StringRef, value.get())
    }

    #[inline]
    pub const fn from_symbol_ref(value: SymbolRef) -> Self {
        Self::tagged(TagKind::SymbolRef, value.get())
    }

    #[inline]
    pub const fn from_bigint_ref(value: BigIntRef) -> Self {
        Self::tagged(TagKind::BigIntRef, value.get())
    }

    #[inline]
    pub const fn from_sentinel(value: InternalSentinel) -> Self {
        Self::tagged(TagKind::Sentinel, value.raw())
    }

    #[inline]
    pub const fn from_suspended_execution_ref(value: SuspendedExecutionRef) -> Self {
        Self::tagged(TagKind::SuspendedExecutionRef, value.get())
    }

    #[inline]
    pub const fn array_hole() -> Self {
        Self::from_sentinel(InternalSentinel::ArrayHole)
    }

    #[inline]
    pub const fn uninitialized_lexical() -> Self {
        Self::from_sentinel(InternalSentinel::UninitializedLexical)
    }

    #[inline]
    pub const fn empty_internal_slot() -> Self {
        Self::from_sentinel(InternalSentinel::EmptyInternalSlot)
    }

    #[inline]
    pub const fn deleted_environment_binding() -> Self {
        Self::from_sentinel(InternalSentinel::DeletedEnvironmentBinding)
    }

    #[inline]
    pub const fn is_undefined(self) -> bool {
        self.has_tag(TagKind::Undefined)
    }

    #[inline]
    pub const fn is_null(self) -> bool {
        self.has_tag(TagKind::Null)
    }

    #[inline]
    pub const fn is_bool(self) -> bool {
        self.has_tag(TagKind::Boolean)
    }

    #[inline]
    pub const fn is_smi(self) -> bool {
        self.has_tag(TagKind::Smi)
    }

    #[inline]
    pub const fn is_double(self) -> bool {
        !Self::is_tagged_bits(self.0)
    }

    #[inline]
    pub const fn is_number(self) -> bool {
        self.is_smi() || self.is_double()
    }

    #[inline]
    pub const fn is_object(self) -> bool {
        self.has_tag(TagKind::ObjectRef)
    }

    #[inline]
    pub const fn is_string(self) -> bool {
        self.has_tag(TagKind::StringRef)
    }

    #[inline]
    pub const fn is_symbol(self) -> bool {
        self.has_tag(TagKind::SymbolRef)
    }

    #[inline]
    pub const fn is_bigint(self) -> bool {
        self.has_tag(TagKind::BigIntRef)
    }

    #[inline]
    pub const fn is_sentinel(self) -> bool {
        self.has_tag(TagKind::Sentinel)
    }

    #[inline]
    pub const fn is_suspended_execution_ref(self) -> bool {
        self.has_tag(TagKind::SuspendedExecutionRef)
    }

    #[inline]
    pub const fn as_bool(self) -> Option<bool> {
        if self.is_bool() {
            Some(self.payload() != 0)
        } else {
            None
        }
    }

    #[inline]
    pub const fn as_smi(self) -> Option<i32> {
        if self.is_smi() {
            Some(self.payload().cast_signed())
        } else {
            None
        }
    }

    #[inline]
    pub fn as_f64(self) -> Option<f64> {
        if self.is_double() {
            Some(f64::from_bits(self.0))
        } else {
            self.as_smi().map(f64::from)
        }
    }

    #[inline]
    pub fn is_nan(self) -> bool {
        self.is_double() && f64::from_bits(self.0).is_nan()
    }

    #[inline]
    pub const fn as_object_ref(self) -> Option<ObjectRef> {
        if self.is_object() {
            ObjectRef::from_raw(self.payload())
        } else {
            None
        }
    }

    #[inline]
    pub const fn as_string_ref(self) -> Option<StringRef> {
        if self.is_string() {
            StringRef::from_raw(self.payload())
        } else {
            None
        }
    }

    #[inline]
    pub const fn as_symbol_ref(self) -> Option<SymbolRef> {
        if self.is_symbol() {
            SymbolRef::from_raw(self.payload())
        } else {
            None
        }
    }

    #[inline]
    pub const fn as_bigint_ref(self) -> Option<BigIntRef> {
        if self.is_bigint() {
            BigIntRef::from_raw(self.payload())
        } else {
            None
        }
    }

    #[inline]
    pub const fn as_sentinel(self) -> Option<InternalSentinel> {
        if self.is_sentinel() {
            InternalSentinel::from_raw(self.payload())
        } else {
            None
        }
    }

    #[inline]
    pub const fn as_suspended_execution_ref(self) -> Option<SuspendedExecutionRef> {
        if self.is_suspended_execution_ref() {
            SuspendedExecutionRef::from_raw(self.payload())
        } else {
            None
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_undefined() {
            return f.write_str("Value::Undefined");
        }
        if self.is_null() {
            return f.write_str("Value::Null");
        }
        if let Some(value) = self.as_bool() {
            return write!(f, "Value::Boolean({value})");
        }
        if let Some(value) = self.as_smi() {
            return write!(f, "Value::Smi({value})");
        }
        if self.is_double() {
            return write!(f, "Value::Double({:?})", f64::from_bits(self.0));
        }
        if let Some(value) = self.as_object_ref() {
            return write!(f, "Value::{value:?}");
        }
        if let Some(value) = self.as_string_ref() {
            return write!(f, "Value::{value:?}");
        }
        if let Some(value) = self.as_symbol_ref() {
            return write!(f, "Value::{value:?}");
        }
        if let Some(value) = self.as_bigint_ref() {
            return write!(f, "Value::{value:?}");
        }
        if let Some(value) = self.as_sentinel() {
            return write!(f, "Value::Sentinel({value:?})");
        }
        if let Some(value) = self.as_suspended_execution_ref() {
            return write!(f, "Value::{value:?}");
        }
        write!(f, "Value::Raw(0x{:016x})", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn value_stays_eight_bytes() {
        assert_eq!(size_of::<Value>(), size_of::<u64>());
    }

    #[test]
    fn undefined_null_and_boolean_round_trip() {
        let undefined = Value::undefined();
        let null = Value::null();
        let t = Value::from_bool(true);
        let f = Value::from_bool(false);

        assert!(undefined.is_undefined());
        assert!(!undefined.is_number());
        assert!(null.is_null());
        assert_eq!(t.as_bool(), Some(true));
        assert_eq!(f.as_bool(), Some(false));
        assert!(!t.is_null());
        assert!(!f.is_undefined());
    }

    #[test]
    fn smi_round_trip_and_numeric_helper_work() {
        let positive = Value::from_smi(123);
        let negative = Value::from_smi(-456);

        assert!(positive.is_smi());
        assert!(negative.is_number());
        assert_eq!(positive.as_smi(), Some(123));
        assert_eq!(negative.as_smi(), Some(-456));
        assert_eq!(positive.as_f64(), Some(123.0));
        assert_eq!(negative.as_f64(), Some(-456.0));
    }

    #[test]
    fn non_nan_doubles_are_stored_directly() {
        let values = [0.0, -0.0, 13.5, f64::INFINITY, f64::NEG_INFINITY];

        for value in values {
            let encoded = Value::from_f64(value);
            assert!(encoded.is_double(), "{value:?} should stay a direct double");
            assert_eq!(encoded.bits(), value.to_bits());
            assert_eq!(encoded.as_f64(), Some(value));
        }
    }

    #[test]
    fn nan_is_canonicalized_without_colliding_with_tags() {
        let encoded = Value::from_f64(f64::NAN);

        assert!(encoded.is_double());
        assert!(encoded.is_nan());
        assert_eq!(encoded.bits(), CANONICAL_NAN_BITS);
        assert!(!encoded.is_undefined());
        assert!(!encoded.is_sentinel());
    }

    #[test]
    fn handle_payloads_round_trip() {
        let object = Value::from_object_ref(ObjectRef::from_raw(1).unwrap());
        let string = Value::from_string_ref(StringRef::from_raw(2).unwrap());
        let symbol = Value::from_symbol_ref(SymbolRef::from_raw(3).unwrap());
        let bigint = Value::from_bigint_ref(BigIntRef::from_raw(4).unwrap());
        let suspended =
            Value::from_suspended_execution_ref(SuspendedExecutionRef::from_raw(5).unwrap());

        assert!(object.is_object());
        assert_eq!(object.as_object_ref(), ObjectRef::from_raw(1));
        assert_eq!(string.as_string_ref(), StringRef::from_raw(2));
        assert_eq!(symbol.as_symbol_ref(), SymbolRef::from_raw(3));
        assert_eq!(bigint.as_bigint_ref(), BigIntRef::from_raw(4));
        assert!(suspended.is_suspended_execution_ref());
        assert_eq!(
            suspended.as_suspended_execution_ref(),
            SuspendedExecutionRef::from_raw(5)
        );
        assert_eq!(object.as_string_ref(), None);
        assert_eq!(string.as_symbol_ref(), None);
    }

    #[test]
    fn sentinels_are_explicit_and_not_guest_visible() {
        let hole = Value::array_hole();
        let lexical = Value::uninitialized_lexical();
        let empty = Value::empty_internal_slot();
        let deleted = Value::deleted_environment_binding();

        assert!(hole.is_sentinel());
        assert_eq!(hole.as_sentinel(), Some(InternalSentinel::ArrayHole));
        assert_eq!(
            lexical.as_sentinel(),
            Some(InternalSentinel::UninitializedLexical)
        );
        assert_eq!(
            empty.as_sentinel(),
            Some(InternalSentinel::EmptyInternalSlot)
        );
        assert_eq!(
            deleted.as_sentinel(),
            Some(InternalSentinel::DeletedEnvironmentBinding)
        );
        assert_ne!(hole.bits(), lexical.bits());
        assert_ne!(lexical.bits(), empty.bits());
        assert_ne!(empty.bits(), deleted.bits());
        assert!(!hole.is_null());
        assert!(!hole.is_undefined());
        assert!(!hole.is_number());
    }
}
