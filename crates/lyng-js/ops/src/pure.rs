use lyng_js_types::{PropertyDescriptor, Value};

#[inline]
fn numeric_pair(left: Value, right: Value) -> Option<(f64, f64)> {
    Some((left.as_f64()?, right.as_f64()?))
}

#[inline]
#[allow(clippy::float_cmp)]
fn numeric_strictly_equal(left: f64, right: f64) -> bool {
    !left.is_nan() && !right.is_nan() && left == right
}

#[inline]
#[allow(clippy::float_cmp)]
fn numeric_same_value(left: f64, right: f64) -> bool {
    if left.is_nan() && right.is_nan() {
        return true;
    }
    if left == 0.0 && right == 0.0 {
        return left.to_bits() == right.to_bits();
    }
    left == right
}

#[inline]
#[allow(clippy::float_cmp)]
fn numeric_same_value_zero(left: f64, right: f64) -> bool {
    if left.is_nan() && right.is_nan() {
        return true;
    }
    left == right
}

/// Classification for the current field-shape of a spec-facing property descriptor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DescriptorKind {
    Generic,
    Data,
    Accessor,
    Mixed,
}

impl DescriptorKind {
    #[inline]
    pub const fn is_generic(self) -> bool {
        matches!(self, Self::Generic)
    }

    #[inline]
    pub const fn is_data(self) -> bool {
        matches!(self, Self::Data | Self::Mixed)
    }

    #[inline]
    pub const fn is_accessor(self) -> bool {
        matches!(self, Self::Accessor | Self::Mixed)
    }

    #[inline]
    pub const fn is_mixed(self) -> bool {
        matches!(self, Self::Mixed)
    }
}

/// Returns the current field-shape classification for a property descriptor.
#[inline]
pub const fn descriptor_kind(descriptor: PropertyDescriptor) -> DescriptorKind {
    match (
        descriptor.has_value() || descriptor.has_writable(),
        descriptor.has_get() || descriptor.has_set(),
    ) {
        (false, false) => DescriptorKind::Generic,
        (true, false) => DescriptorKind::Data,
        (false, true) => DescriptorKind::Accessor,
        (true, true) => DescriptorKind::Mixed,
    }
}

/// ECMAScript `IsDataDescriptor` over the Phase 2 property-descriptor surface.
#[inline]
pub const fn is_data_descriptor(descriptor: PropertyDescriptor) -> bool {
    descriptor_kind(descriptor).is_data()
}

/// ECMAScript `IsAccessorDescriptor` over the Phase 2 property-descriptor surface.
#[inline]
pub const fn is_accessor_descriptor(descriptor: PropertyDescriptor) -> bool {
    descriptor_kind(descriptor).is_accessor()
}

/// ECMAScript `IsGenericDescriptor` over the Phase 2 property-descriptor surface.
#[inline]
pub const fn is_generic_descriptor(descriptor: PropertyDescriptor) -> bool {
    descriptor_kind(descriptor).is_generic()
}

/// Returns whether a descriptor simultaneously carries data and accessor fields.
#[inline]
pub const fn is_mixed_descriptor(descriptor: PropertyDescriptor) -> bool {
    descriptor_kind(descriptor).is_mixed()
}

/// ECMAScript `CompletePropertyDescriptor` over the Phase 2 property-descriptor surface.
///
/// The returned descriptor is a normalized copy. Callers that need the raw
/// presence bits can keep the original descriptor alongside the completed one.
pub const fn complete_property_descriptor(descriptor: PropertyDescriptor) -> PropertyDescriptor {
    let mut completed = descriptor;

    if is_generic_descriptor(descriptor) || is_data_descriptor(descriptor) {
        if !completed.has_value() {
            completed.set_value(Value::undefined());
        }
        if !completed.has_writable() {
            completed.set_writable(false);
        }
    } else {
        if !completed.has_get() {
            completed.set_getter(Value::undefined());
        }
        if !completed.has_set() {
            completed.set_setter(Value::undefined());
        }
    }

    if !completed.has_enumerable() {
        completed.set_enumerable(false);
    }
    if !completed.has_configurable() {
        completed.set_configurable(false);
    }

    completed
}

/// Returns whether the value is `undefined`.
#[inline]
pub const fn is_undefined(value: Value) -> bool {
    value.is_undefined()
}

/// Returns whether the value is `null`.
#[inline]
pub const fn is_null(value: Value) -> bool {
    value.is_null()
}

/// Returns whether the value is `null` or `undefined`.
#[inline]
pub const fn is_nullish(value: Value) -> bool {
    is_null(value) || is_undefined(value)
}

/// Returns whether the value is a boolean.
#[inline]
pub const fn is_boolean(value: Value) -> bool {
    value.is_bool()
}

/// Returns whether the value is a number.
#[inline]
pub const fn is_number(value: Value) -> bool {
    value.is_number()
}

/// Returns whether the value is an object handle.
#[inline]
pub const fn is_object(value: Value) -> bool {
    value.is_object()
}

/// Returns whether the value is a string handle.
#[inline]
pub const fn is_string(value: Value) -> bool {
    value.is_string()
}

/// Returns whether the value is a symbol handle.
#[inline]
pub const fn is_symbol(value: Value) -> bool {
    value.is_symbol()
}

/// Returns whether the value is a bigint handle.
#[inline]
pub const fn is_bigint(value: Value) -> bool {
    value.is_bigint()
}

/// Returns whether the value is an internal runtime sentinel.
#[inline]
pub const fn is_sentinel(value: Value) -> bool {
    value.is_sentinel()
}

/// Returns whether the numeric value is `NaN`.
#[inline]
pub fn is_nan(value: Value) -> bool {
    value.as_f64().is_some_and(f64::is_nan)
}

/// Returns whether the numeric value is finite.
#[inline]
pub fn is_finite_number(value: Value) -> bool {
    value.as_f64().is_some_and(f64::is_finite)
}

/// Returns whether the numeric value is an integral ECMAScript number.
#[inline]
#[allow(clippy::float_cmp)]
pub fn is_integral_number(value: Value) -> bool {
    match value.as_f64() {
        Some(number) => number.is_finite() && number.trunc() == number,
        None => false,
    }
}

/// Returns whether the numeric value is positive zero.
#[inline]
#[allow(clippy::float_cmp)]
pub fn is_positive_zero(value: Value) -> bool {
    match value.as_f64() {
        Some(number) => number == 0.0 && !number.is_sign_negative(),
        None => false,
    }
}

/// Returns whether the numeric value is negative zero.
#[inline]
#[allow(clippy::float_cmp)]
pub fn is_negative_zero(value: Value) -> bool {
    match value.as_f64() {
        Some(number) => number == 0.0 && number.is_sign_negative(),
        None => false,
    }
}

/// Heap-free `ToBoolean` fast path.
///
/// Returns `None` for heap-backed strings and bigints, which require the
/// shared primitive heap to inspect their contents.
#[inline]
pub fn to_boolean(value: Value) -> Option<bool> {
    if value.is_undefined() || value.is_null() {
        return Some(false);
    }
    if let Some(boolean) = value.as_bool() {
        return Some(boolean);
    }
    if let Some(number) = value.as_f64() {
        return Some(!number.is_nan() && number != 0.0);
    }
    if value.is_string() || value.is_bigint() {
        None
    } else {
        Some(true)
    }
}

/// Heap-free `IsStrictlyEqual` fast path.
///
/// Returns `None` when both values are heap-backed strings or bigints, which
/// require content comparison through the shared primitive heap.
#[inline]
pub fn is_strictly_equal(left: Value, right: Value) -> Option<bool> {
    match numeric_pair(left, right) {
        Some((left, right)) => Some(numeric_strictly_equal(left, right)),
        None if (left.is_string() && right.is_string())
            || (left.is_bigint() && right.is_bigint()) =>
        {
            None
        }
        None => Some(left == right),
    }
}

/// Heap-free `SameValue` fast path.
///
/// Returns `None` when both values are heap-backed strings or bigints, which
/// require content comparison through the shared primitive heap.
#[inline]
pub fn same_value(left: Value, right: Value) -> Option<bool> {
    match numeric_pair(left, right) {
        Some((left, right)) => Some(numeric_same_value(left, right)),
        None if (left.is_string() && right.is_string())
            || (left.is_bigint() && right.is_bigint()) =>
        {
            None
        }
        None => Some(left == right),
    }
}

/// Heap-free `SameValueZero` fast path.
///
/// Returns `None` when both values are heap-backed strings or bigints, which
/// require content comparison through the shared primitive heap.
#[inline]
pub fn same_value_zero(left: Value, right: Value) -> Option<bool> {
    match numeric_pair(left, right) {
        Some((left, right)) => Some(numeric_same_value_zero(left, right)),
        None if (left.is_string() && right.is_string())
            || (left.is_bigint() && right.is_bigint()) =>
        {
            None
        }
        None => Some(left == right),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lyng_js_types::{
        BigIntRef, InternalSentinel, ObjectRef, PropertyDescriptor, StringRef, SymbolRef,
    };

    #[test]
    fn pure_predicates_cover_the_phase2_value_classes() {
        let undefined = Value::undefined();
        let null = Value::null();
        let boolean = Value::from_bool(false);
        let number = Value::from_smi(7);
        let object = Value::from_object_ref(ObjectRef::from_raw(1).unwrap());
        let string = Value::from_string_ref(StringRef::from_raw(2).unwrap());
        let symbol = Value::from_symbol_ref(SymbolRef::from_raw(3).unwrap());
        let bigint = Value::from_bigint_ref(BigIntRef::from_raw(4).unwrap());
        let sentinel = Value::from_sentinel(InternalSentinel::ArrayHole);

        assert!(is_undefined(undefined));
        assert!(is_null(null));
        assert!(is_nullish(undefined));
        assert!(is_nullish(null));
        assert!(is_boolean(boolean));
        assert!(is_number(number));
        assert!(is_object(object));
        assert!(is_string(string));
        assert!(is_symbol(symbol));
        assert!(is_bigint(bigint));
        assert!(is_sentinel(sentinel));
        assert!(!is_nullish(number));
        assert!(!is_number(string));
        assert!(!is_boolean(object));
    }

    #[test]
    fn numeric_classification_helpers_cover_edge_cases() {
        let smi_zero = Value::from_smi(0);
        let smi_negative = Value::from_smi(-7);
        let positive_zero = Value::from_f64(0.0);
        let negative_zero = Value::from_f64(-0.0);
        let integer = Value::from_f64(42.0);
        let fractional = Value::from_f64(3.5);
        let infinity = Value::from_f64(f64::INFINITY);
        let nan = Value::from_f64(f64::NAN);

        assert!(is_finite_number(smi_zero));
        assert!(is_finite_number(smi_negative));
        assert!(is_finite_number(integer));
        assert!(!is_finite_number(infinity));
        assert!(!is_finite_number(nan));

        assert!(is_integral_number(smi_zero));
        assert!(is_integral_number(negative_zero));
        assert!(is_integral_number(integer));
        assert!(!is_integral_number(fractional));
        assert!(!is_integral_number(infinity));
        assert!(!is_integral_number(nan));

        assert!(is_positive_zero(smi_zero));
        assert!(is_positive_zero(positive_zero));
        assert!(!is_positive_zero(negative_zero));
        assert!(is_negative_zero(negative_zero));
        assert!(!is_negative_zero(positive_zero));
        assert!(!is_negative_zero(smi_zero));

        assert!(is_nan(nan));
        assert!(!is_nan(integer));
    }

    #[test]
    fn to_boolean_exposes_heap_free_fast_paths() {
        assert_eq!(to_boolean(Value::undefined()), Some(false));
        assert_eq!(to_boolean(Value::null()), Some(false));
        assert_eq!(to_boolean(Value::from_bool(false)), Some(false));
        assert_eq!(to_boolean(Value::from_bool(true)), Some(true));
        assert_eq!(to_boolean(Value::from_smi(0)), Some(false));
        assert_eq!(to_boolean(Value::from_smi(1)), Some(true));
        assert_eq!(to_boolean(Value::from_f64(-0.0)), Some(false));
        assert_eq!(to_boolean(Value::from_f64(f64::NAN)), Some(false));
        assert_eq!(to_boolean(Value::from_f64(f64::INFINITY)), Some(true));
        assert_eq!(
            to_boolean(Value::from_object_ref(ObjectRef::from_raw(7).unwrap())),
            Some(true)
        );
        assert_eq!(
            to_boolean(Value::from_string_ref(StringRef::from_raw(8).unwrap())),
            None
        );
        assert_eq!(
            to_boolean(Value::from_symbol_ref(SymbolRef::from_raw(9).unwrap())),
            Some(true)
        );
        assert_eq!(
            to_boolean(Value::from_bigint_ref(BigIntRef::from_raw(10).unwrap())),
            None
        );
        assert_eq!(to_boolean(Value::array_hole()), Some(true));
    }

    #[test]
    fn strict_equality_exposes_heap_free_fast_paths() {
        let object = ObjectRef::from_raw(11).unwrap();
        let string = StringRef::from_raw(12).unwrap();
        let symbol = SymbolRef::from_raw(13).unwrap();
        let bigint = BigIntRef::from_raw(14).unwrap();

        assert_eq!(
            is_strictly_equal(Value::undefined(), Value::undefined()),
            Some(true)
        );
        assert_eq!(
            is_strictly_equal(Value::undefined(), Value::null()),
            Some(false)
        );
        assert_eq!(
            is_strictly_equal(Value::from_bool(true), Value::from_bool(true)),
            Some(true)
        );
        assert_eq!(
            is_strictly_equal(Value::from_bool(true), Value::from_bool(false)),
            Some(false)
        );
        assert_eq!(
            is_strictly_equal(Value::from_smi(5), Value::from_f64(5.0)),
            Some(true)
        );
        assert_eq!(
            is_strictly_equal(Value::from_smi(0), Value::from_f64(-0.0)),
            Some(true)
        );
        assert_eq!(
            is_strictly_equal(Value::from_f64(f64::NAN), Value::from_f64(f64::NAN)),
            Some(false)
        );
        assert_eq!(
            is_strictly_equal(
                Value::from_object_ref(object),
                Value::from_object_ref(object)
            ),
            Some(true)
        );
        assert_eq!(
            is_strictly_equal(
                Value::from_string_ref(string),
                Value::from_string_ref(StringRef::from_raw(15).unwrap())
            ),
            None
        );
        assert_eq!(
            is_strictly_equal(
                Value::from_symbol_ref(symbol),
                Value::from_symbol_ref(symbol)
            ),
            Some(true)
        );
        assert_eq!(
            is_strictly_equal(
                Value::from_bigint_ref(bigint),
                Value::from_bigint_ref(bigint)
            ),
            None
        );
        assert_eq!(
            is_strictly_equal(
                Value::array_hole(),
                Value::from_sentinel(InternalSentinel::ArrayHole)
            ),
            Some(true)
        );
        assert_eq!(
            is_strictly_equal(Value::array_hole(), Value::empty_internal_slot()),
            Some(false)
        );
    }

    #[test]
    fn same_value_exposes_heap_free_fast_paths() {
        let shared_string = StringRef::from_raw(16).unwrap();
        let shared_bigint = BigIntRef::from_raw(17).unwrap();

        assert_eq!(
            same_value(Value::from_f64(f64::NAN), Value::from_f64(f64::NAN)),
            Some(true)
        );
        assert_eq!(
            same_value(Value::from_smi(0), Value::from_f64(-0.0)),
            Some(false)
        );
        assert_eq!(
            same_value(Value::from_f64(0.0), Value::from_f64(-0.0)),
            Some(false)
        );
        assert_eq!(
            same_value(Value::from_smi(9), Value::from_f64(9.0)),
            Some(true)
        );
        assert_eq!(
            same_value(
                Value::from_string_ref(shared_string),
                Value::from_string_ref(shared_string)
            ),
            None
        );
        assert_eq!(
            same_value(
                Value::from_bigint_ref(shared_bigint),
                Value::from_bigint_ref(shared_bigint)
            ),
            None
        );
        assert_eq!(
            same_value(
                Value::from_string_ref(shared_string),
                Value::from_bigint_ref(shared_bigint)
            ),
            Some(false)
        );
    }

    #[test]
    fn same_value_zero_exposes_heap_free_fast_paths() {
        assert_eq!(
            same_value_zero(Value::from_f64(f64::NAN), Value::from_f64(f64::NAN)),
            Some(true)
        );
        assert_eq!(
            same_value_zero(Value::from_smi(0), Value::from_f64(-0.0)),
            Some(true)
        );
        assert_eq!(
            same_value_zero(Value::from_f64(0.0), Value::from_f64(-0.0)),
            Some(true)
        );
        assert_eq!(
            same_value_zero(Value::from_smi(23), Value::from_f64(23.0)),
            Some(true)
        );
        assert_eq!(
            same_value_zero(Value::null(), Value::undefined()),
            Some(false)
        );
    }

    #[test]
    fn descriptor_kind_is_explicit_for_generic_data_accessor_and_mixed_shapes() {
        let generic = PropertyDescriptor::new();
        let mut data = PropertyDescriptor::new();
        let mut accessor = PropertyDescriptor::new();
        let mut mixed = PropertyDescriptor::new();

        data.set_value(Value::from_smi(1));
        accessor.set_getter(Value::undefined());
        mixed.set_writable(true);
        mixed.set_setter(Value::from_bool(true));

        assert_eq!(descriptor_kind(generic), DescriptorKind::Generic);
        assert_eq!(descriptor_kind(data), DescriptorKind::Data);
        assert_eq!(descriptor_kind(accessor), DescriptorKind::Accessor);
        assert_eq!(descriptor_kind(mixed), DescriptorKind::Mixed);

        assert!(is_generic_descriptor(generic));
        assert!(is_data_descriptor(data));
        assert!(is_accessor_descriptor(accessor));
        assert!(is_mixed_descriptor(mixed));
        assert!(is_data_descriptor(mixed));
        assert!(is_accessor_descriptor(mixed));
        assert!(!is_generic_descriptor(mixed));
    }

    #[test]
    fn complete_property_descriptor_normalizes_a_copy_without_clobbering_presence_bits() {
        let mut original = PropertyDescriptor::new();
        original.set_enumerable(true);

        let completed = complete_property_descriptor(original);

        assert_eq!(original.value(), None);
        assert_eq!(original.writable(), None);
        assert_eq!(original.enumerable(), Some(true));
        assert_eq!(original.configurable(), None);

        assert_eq!(completed.value(), Some(Value::undefined()));
        assert_eq!(completed.writable(), Some(false));
        assert_eq!(completed.enumerable(), Some(true));
        assert_eq!(completed.configurable(), Some(false));
        assert_eq!(descriptor_kind(completed), DescriptorKind::Data);
    }

    #[test]
    fn complete_property_descriptor_fills_missing_accessor_fields() {
        let mut descriptor = PropertyDescriptor::new();
        descriptor.set_getter(Value::from_bool(true));

        let completed = complete_property_descriptor(descriptor);

        assert_eq!(descriptor.setter(), None);
        assert_eq!(completed.getter(), Some(Value::from_bool(true)));
        assert_eq!(completed.setter(), Some(Value::undefined()));
        assert_eq!(completed.enumerable(), Some(false));
        assert_eq!(completed.configurable(), Some(false));
        assert_eq!(descriptor_kind(completed), DescriptorKind::Accessor);
    }

    #[test]
    fn complete_property_descriptor_keeps_mixed_descriptors_mixed_while_applying_defaults() {
        let mut descriptor = PropertyDescriptor::new();
        descriptor.set_value(Value::from_smi(9));
        descriptor.set_setter(Value::from_bool(false));

        let completed = complete_property_descriptor(descriptor);

        assert_eq!(completed.value(), Some(Value::from_smi(9)));
        assert_eq!(completed.writable(), Some(false));
        assert_eq!(completed.setter(), Some(Value::from_bool(false)));
        assert_eq!(completed.enumerable(), Some(false));
        assert_eq!(completed.configurable(), Some(false));
        assert_eq!(descriptor_kind(completed), DescriptorKind::Mixed);
    }
}
