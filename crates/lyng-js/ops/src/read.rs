use crate::convert::{
    bigint_equals_integral_number, bigint_view_equals_parts, encode_number, logical_type,
    lossy_string_from_view, parse_string_to_bigint, primitive_type_error, same_logical_type,
    string_view_to_number, LogicalType,
};
use crate::pure;
use lyng_js_common::AtomId;
use lyng_js_env::Agent;
use lyng_js_gc::{BigIntSign, PrimitiveHeapView};
use lyng_js_types::{BigIntRef, Completion, StringRef, Value};

#[inline]
fn borrowed_strings_equal(
    heap: PrimitiveHeapView<'_>,
    left: StringRef,
    right: StringRef,
) -> Completion<bool> {
    let left_view = heap.string_view(left).ok_or_else(primitive_type_error)?;
    if left == right {
        return Ok(true);
    }
    let right_view = heap.string_view(right).ok_or_else(primitive_type_error)?;
    Ok(left_view.equals(&right_view))
}

#[inline]
fn borrowed_bigint_equal(
    heap: PrimitiveHeapView<'_>,
    left: BigIntRef,
    right: BigIntRef,
) -> Completion<bool> {
    let left_view = heap.bigint_view(left).ok_or_else(primitive_type_error)?;
    if left == right {
        return Ok(true);
    }
    let right_view = heap.bigint_view(right).ok_or_else(primitive_type_error)?;
    Ok(left_view.sign() == right_view.sign()
        && left_view.limb_count() == right_view.limb_count()
        && left_view.limb_bytes_le() == right_view.limb_bytes_le())
}

fn loosely_equal_same_type(
    heap: PrimitiveHeapView<'_>,
    left: Value,
    right: Value,
) -> Completion<bool> {
    is_strictly_equal(heap, left, right)
}

/// Read-only string equality over the shared primitive heap view.
#[inline]
pub fn strings_equal(
    heap: PrimitiveHeapView<'_>,
    left: StringRef,
    right: StringRef,
) -> Option<bool> {
    heap.strings_equal(left, right)
}

/// Read-only string length inspection in UTF-16 code units.
#[inline]
pub fn string_code_unit_len(heap: PrimitiveHeapView<'_>, value: StringRef) -> Option<u32> {
    Some(heap.string_view(value)?.code_unit_len())
}

/// Read-only string code-unit access.
#[inline]
pub fn string_code_unit_at(
    heap: PrimitiveHeapView<'_>,
    value: StringRef,
    index: usize,
) -> Option<u16> {
    heap.string_view(value)?.code_unit_at(index)
}

/// Read-only string hash inspection that falls back to deterministic computation.
#[inline]
pub fn string_hash(heap: PrimitiveHeapView<'_>, value: StringRef) -> Option<u32> {
    let view = heap.string_view(value)?;
    Some(view.cached_hash().unwrap_or_else(|| view.compute_hash()))
}

/// Read-only inspection of whether a runtime string has already been atomized.
#[inline]
pub fn string_cached_atom(heap: PrimitiveHeapView<'_>, value: StringRef) -> Option<Option<AtomId>> {
    Some(heap.string_view(value)?.cached_atom())
}

/// Read-only bigint sign inspection over the shared primitive heap view.
#[inline]
pub fn bigint_sign(heap: PrimitiveHeapView<'_>, value: BigIntRef) -> Option<BigIntSign> {
    Some(heap.bigint_view(value)?.sign())
}

/// Read-only bigint zero check over the shared primitive heap view.
#[inline]
pub fn bigint_is_zero(heap: PrimitiveHeapView<'_>, value: BigIntRef) -> Option<bool> {
    Some(heap.bigint_view(value)?.is_zero())
}

/// Read-only bigint limb-count inspection over the shared primitive heap view.
#[inline]
pub fn bigint_limb_count(heap: PrimitiveHeapView<'_>, value: BigIntRef) -> Option<u32> {
    Some(heap.bigint_view(value)?.limb_count())
}

/// Read-only bigint limb access over the shared primitive heap view.
#[inline]
pub fn bigint_limb_at(heap: PrimitiveHeapView<'_>, value: BigIntRef, index: usize) -> Option<u64> {
    heap.bigint_view(value)?.limb_at(index)
}

/// Read-only borrowed little-endian bigint limb bytes.
#[inline]
pub fn bigint_limb_bytes_le(heap: PrimitiveHeapView<'_>, value: BigIntRef) -> Option<&[u8]> {
    Some(heap.bigint_view(value)?.limb_bytes_le())
}

/// ECMAScript `ToBoolean` over the primitive-runtime surface.
///
/// # Errors
///
/// Returns a primitive type error if a string or bigint handle does not resolve
/// in the shared primitive heap.
pub fn to_boolean(heap: PrimitiveHeapView<'_>, value: Value) -> Completion<bool> {
    if let Some(result) = pure::to_boolean(value) {
        return Ok(result);
    }
    if let Some(string) = value.as_string_ref() {
        return Ok(heap
            .string_view(string)
            .ok_or_else(primitive_type_error)?
            .code_unit_len()
            != 0);
    }
    if let Some(bigint) = value.as_bigint_ref() {
        return Ok(!heap
            .bigint_view(bigint)
            .ok_or_else(primitive_type_error)?
            .is_zero());
    }
    Ok(true)
}

/// ECMAScript `ToBoolean` with Annex B `[[IsHTMLDDA]]` object support.
///
/// # Errors
///
/// Returns a primitive type error if a string or bigint handle does not resolve
/// in the shared primitive heap.
pub fn to_boolean_agent(agent: &Agent, value: Value) -> Completion<bool> {
    if value
        .as_object_ref()
        .is_some_and(|object| agent.objects().is_html_dda_object(object))
    {
        return Ok(false);
    }
    to_boolean(agent.heap().view(), value)
}

/// ECMAScript `IsStrictlyEqual` over the shared primitive heap view.
///
/// # Errors
///
/// Returns a primitive type error if a string or bigint handle does not resolve
/// in the shared primitive heap.
///
/// # Panics
///
/// Panics if a value classified as a number, string, or bigint fails to expose
/// the matching payload, which indicates a corrupted internal `Value`.
pub fn is_strictly_equal(
    heap: PrimitiveHeapView<'_>,
    left: Value,
    right: Value,
) -> Completion<bool> {
    if let (Some(left), Some(right)) = (left.as_string_ref(), right.as_string_ref()) {
        return borrowed_strings_equal(heap, left, right);
    }
    if let (Some(left), Some(right)) = (left.as_bigint_ref(), right.as_bigint_ref()) {
        return borrowed_bigint_equal(heap, left, right);
    }
    if let Some(result) = pure::is_strictly_equal(left, right) {
        return Ok(result);
    }

    match (logical_type(left), logical_type(right)) {
        (LogicalType::String, LogicalType::String) => borrowed_strings_equal(
            heap,
            left.as_string_ref().unwrap(),
            right.as_string_ref().unwrap(),
        ),
        (LogicalType::BigInt, LogicalType::BigInt) => borrowed_bigint_equal(
            heap,
            left.as_bigint_ref().unwrap(),
            right.as_bigint_ref().unwrap(),
        ),
        _ => unreachable!("pure fast path should handle non-heap strict equality"),
    }
}

/// ECMAScript `SameValue` over the shared primitive heap view.
///
/// # Errors
///
/// Returns a primitive type error if a string or bigint handle does not resolve
/// in the shared primitive heap.
///
/// # Panics
///
/// Panics if a value classified as a number, string, or bigint fails to expose
/// the matching payload, which indicates a corrupted internal `Value`.
pub fn same_value(heap: PrimitiveHeapView<'_>, left: Value, right: Value) -> Completion<bool> {
    if let (Some(left), Some(right)) = (left.as_string_ref(), right.as_string_ref()) {
        return borrowed_strings_equal(heap, left, right);
    }
    if let (Some(left), Some(right)) = (left.as_bigint_ref(), right.as_bigint_ref()) {
        return borrowed_bigint_equal(heap, left, right);
    }
    if let Some(result) = pure::same_value(left, right) {
        return Ok(result);
    }

    match (logical_type(left), logical_type(right)) {
        (LogicalType::String, LogicalType::String) => borrowed_strings_equal(
            heap,
            left.as_string_ref().unwrap(),
            right.as_string_ref().unwrap(),
        ),
        (LogicalType::BigInt, LogicalType::BigInt) => borrowed_bigint_equal(
            heap,
            left.as_bigint_ref().unwrap(),
            right.as_bigint_ref().unwrap(),
        ),
        _ => unreachable!("pure fast path should handle non-heap SameValue"),
    }
}

/// ECMAScript `SameValueZero` over the shared primitive heap view.
///
/// # Errors
///
/// Returns a primitive type error if a string or bigint handle does not resolve
/// in the shared primitive heap.
///
/// # Panics
///
/// Panics if a value classified as a number, string, or bigint fails to expose
/// the matching payload, which indicates a corrupted internal `Value`.
pub fn same_value_zero(heap: PrimitiveHeapView<'_>, left: Value, right: Value) -> Completion<bool> {
    if let (Some(left), Some(right)) = (left.as_string_ref(), right.as_string_ref()) {
        return borrowed_strings_equal(heap, left, right);
    }
    if let (Some(left), Some(right)) = (left.as_bigint_ref(), right.as_bigint_ref()) {
        return borrowed_bigint_equal(heap, left, right);
    }
    if let Some(result) = pure::same_value_zero(left, right) {
        return Ok(result);
    }

    match (logical_type(left), logical_type(right)) {
        (LogicalType::String, LogicalType::String) => borrowed_strings_equal(
            heap,
            left.as_string_ref().unwrap(),
            right.as_string_ref().unwrap(),
        ),
        (LogicalType::BigInt, LogicalType::BigInt) => borrowed_bigint_equal(
            heap,
            left.as_bigint_ref().unwrap(),
            right.as_bigint_ref().unwrap(),
        ),
        _ => unreachable!("pure fast path should handle non-heap SameValueZero"),
    }
}

/// ECMAScript `ToNumber` over the primitive-runtime surface.
///
/// This covers primitive values only. Object coercion remains an object-aware
/// operation layered on top of the same entrypoint.
///
/// # Errors
///
/// Returns a primitive type error for symbols, bigints, objects, and internal
/// sentinels, or if a string handle does not resolve in the shared primitive
/// heap.
pub fn to_number(heap: PrimitiveHeapView<'_>, value: Value) -> Completion<Value> {
    if value.is_undefined() {
        return Ok(Value::from_f64(f64::NAN));
    }
    if value.is_null() {
        return Ok(Value::from_smi(0));
    }
    if let Some(boolean) = value.as_bool() {
        return Ok(Value::from_smi(i32::from(boolean)));
    }
    if value.is_number() {
        return Ok(value);
    }
    if let Some(string) = value.as_string_ref() {
        let view = heap.string_view(string).ok_or_else(primitive_type_error)?;
        return Ok(encode_number(string_view_to_number(&view)));
    }
    if value.is_symbol() || value.is_bigint() {
        return Err(primitive_type_error());
    }
    if value.is_object() || value.is_sentinel() {
        return Err(primitive_type_error());
    }

    Err(primitive_type_error())
}

/// ECMAScript `ToNumeric` over the primitive-runtime surface.
///
/// # Errors
///
/// Returns a primitive type error for objects and internal sentinels.
pub fn to_numeric(heap: PrimitiveHeapView<'_>, value: Value) -> Completion<Value> {
    if value.is_object() || value.is_sentinel() {
        return Err(primitive_type_error());
    }
    if value.is_bigint() {
        return Ok(value);
    }

    to_number(heap, value)
}

/// Primitive-only ECMAScript `IsLooselyEqual` over the shared primitive heap view.
///
/// Same-type string and bigint comparisons are content-based rather than handle
/// identity. Object-to-primitive coercion is intentionally deferred.
///
/// # Errors
///
/// Returns a primitive type error for objects, internal sentinels, and invalid
/// runtime handles.
///
/// # Panics
///
/// Panics if a value classified as a number, string, or bigint fails to expose
/// the matching payload, which indicates a corrupted internal `Value`.
pub fn is_loosely_equal(
    heap: PrimitiveHeapView<'_>,
    left: Value,
    right: Value,
) -> Completion<bool> {
    if same_logical_type(left, right) {
        return loosely_equal_same_type(heap, left, right);
    }

    if (left.is_null() && right.is_undefined()) || (left.is_undefined() && right.is_null()) {
        return Ok(true);
    }

    if left.is_number() && right.is_string() {
        let right_number = to_number(heap, right)?;
        return Ok(pure::is_strictly_equal(left, right_number).unwrap());
    }
    if left.is_string() && right.is_number() {
        let left_number = to_number(heap, left)?;
        return Ok(pure::is_strictly_equal(left_number, right).unwrap());
    }

    if left.is_bigint() && right.is_string() {
        let right_view = heap
            .string_view(right.as_string_ref().unwrap())
            .ok_or_else(primitive_type_error)?;
        let right_text = lossy_string_from_view(&right_view);
        let Some((sign, limbs)) = parse_string_to_bigint(&right_text) else {
            return Ok(false);
        };
        let left_view = heap
            .bigint_view(left.as_bigint_ref().unwrap())
            .ok_or_else(primitive_type_error)?;
        return Ok(bigint_view_equals_parts(left_view, sign, &limbs));
    }
    if left.is_string() && right.is_bigint() {
        return is_loosely_equal(heap, right, left);
    }

    if left.is_bool() {
        return is_loosely_equal(heap, to_number(heap, left)?, right);
    }
    if right.is_bool() {
        return is_loosely_equal(heap, left, to_number(heap, right)?);
    }

    if left.is_bigint() && right.is_number() {
        let left_view = heap
            .bigint_view(left.as_bigint_ref().unwrap())
            .ok_or_else(primitive_type_error)?;
        return Ok(bigint_equals_integral_number(
            left_view,
            right.as_f64().unwrap(),
        ));
    }
    if left.is_number() && right.is_bigint() {
        return is_loosely_equal(heap, right, left);
    }

    if left.is_object() || right.is_object() || left.is_sentinel() || right.is_sentinel() {
        return Err(primitive_type_error());
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lyng_js_gc::{AllocationLifetime, PrimitiveHeap, StringEncoding};
    use lyng_js_types::{AbruptCompletion, ObjectRef, SymbolRef};

    #[test]
    fn read_only_helpers_take_shared_heap_views() {
        let mut heap = PrimitiveHeap::new();
        let (latin1, utf16, bigint) = {
            let mut mutator = heap.mutator();
            let latin1 = mutator.alloc_string(
                StringEncoding::Latin1,
                4,
                &[0x63, 0x61, 0x66, 0xE9],
                None,
                AllocationLifetime::Default,
            );
            let utf16 = mutator.alloc_string(
                StringEncoding::Utf16,
                4,
                &[0x63, 0x00, 0x61, 0x00, 0x66, 0x00, 0xE9, 0x00],
                None,
                AllocationLifetime::Default,
            );
            let bigint = mutator.alloc_bigint(
                BigIntSign::Negative,
                &[0x0102_0304_0506_0708, 0],
                AllocationLifetime::Default,
            );
            (latin1, utf16, bigint)
        };

        let view = heap.view();
        assert_eq!(strings_equal(view, latin1, utf16), Some(true));
        assert_eq!(string_code_unit_len(view, latin1), Some(4));
        assert_eq!(string_code_unit_at(view, latin1, 3), Some(0x00E9));
        assert_eq!(string_code_unit_at(view, utf16, 3), Some(0x00E9));
        assert_eq!(string_hash(view, latin1), string_hash(view, utf16));
        assert_eq!(string_cached_atom(view, latin1), Some(None));
        assert_eq!(bigint_sign(view, bigint), Some(BigIntSign::Negative));
        assert_eq!(bigint_is_zero(view, bigint), Some(false));
        assert_eq!(bigint_limb_count(view, bigint), Some(1));
        assert_eq!(bigint_limb_at(view, bigint, 0), Some(0x0102_0304_0506_0708));
        assert_eq!(bigint_limb_at(view, bigint, 1), None);
        assert_eq!(
            bigint_limb_bytes_le(view, bigint),
            Some(&[0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01][..])
        );
    }

    #[test]
    fn heap_aware_boolean_and_equality_cover_string_and_bigint_content() {
        let mut heap = PrimitiveHeap::new();
        let (empty, same_a, same_b, zero, bigint_a, bigint_b) = {
            let mut mutator = heap.mutator();
            let empty = mutator.alloc_string(
                StringEncoding::Latin1,
                0,
                b"",
                None,
                AllocationLifetime::Default,
            );
            let same_a = mutator.alloc_string(
                StringEncoding::Latin1,
                3,
                b"abc",
                None,
                AllocationLifetime::Default,
            );
            let same_b = mutator.alloc_string(
                StringEncoding::Utf16,
                3,
                &[0x61, 0x00, 0x62, 0x00, 0x63, 0x00],
                None,
                AllocationLifetime::Default,
            );
            let zero =
                mutator.alloc_bigint(BigIntSign::Negative, &[0, 0], AllocationLifetime::Default);
            let bigint_a = mutator.alloc_bigint(
                BigIntSign::Negative,
                &[9, 1, 0],
                AllocationLifetime::Default,
            );
            let bigint_b =
                mutator.alloc_bigint(BigIntSign::Negative, &[9, 1], AllocationLifetime::Default);
            (empty, same_a, same_b, zero, bigint_a, bigint_b)
        };

        let view = heap.view();
        assert_eq!(to_boolean(view, Value::from_string_ref(empty)), Ok(false));
        assert_eq!(to_boolean(view, Value::from_string_ref(same_a)), Ok(true));
        assert_eq!(to_boolean(view, Value::from_bigint_ref(zero)), Ok(false));
        assert_eq!(to_boolean(view, Value::from_bigint_ref(bigint_a)), Ok(true));
        assert_eq!(
            is_strictly_equal(
                view,
                Value::from_string_ref(same_a),
                Value::from_string_ref(same_b)
            ),
            Ok(true)
        );
        assert_eq!(
            same_value(
                view,
                Value::from_bigint_ref(bigint_a),
                Value::from_bigint_ref(bigint_b)
            ),
            Ok(true)
        );
        assert_eq!(
            same_value_zero(
                view,
                Value::from_string_ref(same_a),
                Value::from_string_ref(same_b)
            ),
            Ok(true)
        );
    }

    #[test]
    fn invalid_runtime_handles_return_type_errors() {
        let heap = PrimitiveHeap::new();
        let view = heap.view();
        let invalid_string = StringRef::from_raw(99).unwrap();
        let invalid_bigint = BigIntRef::from_raw(101).unwrap();

        assert_eq!(
            to_boolean(view, Value::from_string_ref(invalid_string)),
            Err(AbruptCompletion::Throw(Value::undefined()))
        );
        assert_eq!(
            to_boolean(view, Value::from_bigint_ref(invalid_bigint)),
            Err(AbruptCompletion::Throw(Value::undefined()))
        );
        assert_eq!(
            is_strictly_equal(
                view,
                Value::from_string_ref(invalid_string),
                Value::from_string_ref(invalid_string)
            ),
            Err(AbruptCompletion::Throw(Value::undefined()))
        );
        assert_eq!(
            same_value(
                view,
                Value::from_bigint_ref(invalid_bigint),
                Value::from_bigint_ref(invalid_bigint)
            ),
            Err(AbruptCompletion::Throw(Value::undefined()))
        );
        assert_eq!(
            same_value_zero(
                view,
                Value::from_string_ref(invalid_string),
                Value::from_string_ref(invalid_string)
            ),
            Err(AbruptCompletion::Throw(Value::undefined()))
        );
        assert_eq!(
            to_number(view, Value::from_string_ref(invalid_string)),
            Err(AbruptCompletion::Throw(Value::undefined()))
        );
        assert_eq!(
            is_loosely_equal(
                view,
                Value::from_bigint_ref(invalid_bigint),
                Value::from_string_ref(invalid_string),
            ),
            Err(AbruptCompletion::Throw(Value::undefined()))
        );
    }

    #[test]
    fn to_number_and_to_numeric_cover_primitive_cases() {
        let mut heap = PrimitiveHeap::new();
        let (decimal, negative_zero, invalid, large_hex, bigint) = {
            let mut mutator = heap.mutator();
            let decimal = mutator.alloc_string(
                StringEncoding::Latin1,
                4,
                b" 42 ",
                None,
                AllocationLifetime::Default,
            );
            let negative_zero = mutator.alloc_string(
                StringEncoding::Latin1,
                2,
                b"-0",
                None,
                AllocationLifetime::Default,
            );
            let invalid = mutator.alloc_string(
                StringEncoding::Latin1,
                3,
                b"foo",
                None,
                AllocationLifetime::Default,
            );
            let large_hex = mutator.alloc_string(
                StringEncoding::Latin1,
                19,
                b"0x10000000000000000",
                None,
                AllocationLifetime::Default,
            );
            let bigint =
                mutator.alloc_bigint(BigIntSign::Negative, &[17], AllocationLifetime::Default);
            (decimal, negative_zero, invalid, large_hex, bigint)
        };

        let view = heap.view();
        assert!(to_number(view, Value::undefined()).unwrap().is_nan());
        assert_eq!(to_number(view, Value::null()), Ok(Value::from_smi(0)));
        assert_eq!(
            to_number(view, Value::from_bool(true)),
            Ok(Value::from_smi(1))
        );
        assert_eq!(
            to_number(view, Value::from_string_ref(decimal)),
            Ok(Value::from_smi(42))
        );
        assert_eq!(
            to_number(view, Value::from_string_ref(large_hex))
                .unwrap()
                .as_f64(),
            Some(18_446_744_073_709_552_000.0)
        );
        assert_eq!(
            to_number(view, Value::from_string_ref(negative_zero))
                .unwrap()
                .as_f64(),
            Some(-0.0)
        );
        assert!(to_number(view, Value::from_string_ref(invalid))
            .unwrap()
            .is_nan());
        assert_eq!(
            to_numeric(view, Value::from_bigint_ref(bigint)),
            Ok(Value::from_bigint_ref(bigint))
        );
        assert_eq!(
            to_number(
                view,
                Value::from_symbol_ref(SymbolRef::from_raw(7).unwrap())
            ),
            Err(AbruptCompletion::Throw(Value::undefined()))
        );
        assert_eq!(
            to_number(view, Value::from_bigint_ref(bigint)),
            Err(AbruptCompletion::Throw(Value::undefined()))
        );
        assert_eq!(
            to_number(view, Value::array_hole()),
            Err(AbruptCompletion::Throw(Value::undefined()))
        );
        assert_eq!(
            to_number(
                view,
                Value::from_object_ref(ObjectRef::from_raw(8).unwrap())
            ),
            Err(AbruptCompletion::Throw(Value::undefined()))
        );
        assert_eq!(
            to_numeric(view, Value::array_hole()),
            Err(AbruptCompletion::Throw(Value::undefined()))
        );
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn loose_equality_covers_primitive_mixed_type_matrix() {
        let mut heap = PrimitiveHeap::new();
        let (
            left_string,
            right_string,
            numeric_string,
            prefixed_bigint_string,
            invalid_bigint_string,
            bigint_a,
            bigint_b,
        ) = {
            let mut mutator = heap.mutator();
            let left_string = mutator.alloc_string(
                StringEncoding::Latin1,
                3,
                b"abc",
                None,
                AllocationLifetime::Default,
            );
            let right_string = mutator.alloc_string(
                StringEncoding::Latin1,
                3,
                b"abc",
                None,
                AllocationLifetime::Default,
            );
            let numeric_string = mutator.alloc_string(
                StringEncoding::Latin1,
                1,
                b"1",
                None,
                AllocationLifetime::Default,
            );
            let prefixed_bigint_string = mutator.alloc_string(
                StringEncoding::Latin1,
                4,
                b"0x10",
                None,
                AllocationLifetime::Default,
            );
            let invalid_bigint_string = mutator.alloc_string(
                StringEncoding::Latin1,
                3,
                b"1.5",
                None,
                AllocationLifetime::Default,
            );
            let bigint_a =
                mutator.alloc_bigint(BigIntSign::NonNegative, &[16], AllocationLifetime::Default);
            let bigint_b =
                mutator.alloc_bigint(BigIntSign::NonNegative, &[16], AllocationLifetime::Default);
            (
                left_string,
                right_string,
                numeric_string,
                prefixed_bigint_string,
                invalid_bigint_string,
                bigint_a,
                bigint_b,
            )
        };

        let view = heap.view();
        assert_eq!(
            is_loosely_equal(
                view,
                Value::from_string_ref(left_string),
                Value::from_string_ref(right_string),
            ),
            Ok(true)
        );
        assert_eq!(
            is_loosely_equal(
                view,
                Value::from_smi(1),
                Value::from_string_ref(numeric_string),
            ),
            Ok(true)
        );
        assert_eq!(
            is_loosely_equal(view, Value::from_bool(true), Value::from_smi(1)),
            Ok(true)
        );
        assert_eq!(
            is_loosely_equal(view, Value::null(), Value::undefined()),
            Ok(true)
        );
        assert_eq!(
            is_loosely_equal(
                view,
                Value::from_bigint_ref(bigint_a),
                Value::from_bigint_ref(bigint_b),
            ),
            Ok(true)
        );
        assert_eq!(
            is_loosely_equal(
                view,
                Value::from_bigint_ref(bigint_a),
                Value::from_string_ref(prefixed_bigint_string),
            ),
            Ok(true)
        );
        assert_eq!(
            is_loosely_equal(
                view,
                Value::from_bigint_ref(bigint_a),
                Value::from_f64(16.5),
            ),
            Ok(false)
        );
        assert_eq!(
            is_loosely_equal(
                view,
                Value::from_bigint_ref(bigint_a),
                Value::from_string_ref(invalid_bigint_string),
            ),
            Ok(false)
        );
        assert_eq!(
            is_loosely_equal(
                view,
                Value::from_object_ref(ObjectRef::from_raw(11).unwrap()),
                Value::null(),
            ),
            Err(AbruptCompletion::Throw(Value::undefined()))
        );
    }
}
