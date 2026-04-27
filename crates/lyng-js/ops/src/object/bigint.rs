use crate::{
    convert::{
        bigint_parts_to_radix_string, bigint_view_to_radix_string, bigint_view_to_string,
        integral_number_to_bigint, lossy_string_from_view, parse_string_to_bigint,
    },
    errors::{throw_range_error, throw_syntax_error, throw_type_error},
};
use lyng_js_env::Agent;
use lyng_js_gc::AllocationLifetime;
use lyng_js_types::{Completion, Value};

/// Converts one already-primitive ECMAScript value into a BigInt.
///
/// # Errors
/// Returns `TypeError` for unsupported logical types, `SyntaxError` for invalid
/// BigInt strings, and `RangeError` for non-integral number input.
pub fn primitive_to_bigint(agent: &mut Agent, value: Value) -> Completion<Value> {
    if value.is_bigint() {
        return Ok(value);
    }
    if let Some(boolean) = value.as_bool() {
        let bigint = agent.heap_mut().mutator().alloc_bigint(
            lyng_js_gc::BigIntSign::NonNegative,
            &[u64::from(boolean)],
            AllocationLifetime::Default,
        );
        return Ok(Value::from_bigint_ref(bigint));
    }
    if let Some(number) = value.as_f64() {
        let Some((sign, limbs)) = integral_number_to_bigint(number) else {
            return Err(throw_range_error(agent));
        };
        let bigint =
            agent
                .heap_mut()
                .mutator()
                .alloc_bigint(sign, &limbs, AllocationLifetime::Default);
        return Ok(Value::from_bigint_ref(bigint));
    }
    if let Some(string) = value.as_string_ref() {
        let text = agent
            .heap()
            .view()
            .string_view(string)
            .map(lossy_string_from_view)
            .ok_or_else(|| throw_type_error(agent))?;
        let Some((sign, limbs)) = parse_string_to_bigint(&text) else {
            return Err(throw_syntax_error(agent));
        };
        let bigint =
            agent
                .heap_mut()
                .mutator()
                .alloc_bigint(sign, &limbs, AllocationLifetime::Default);
        return Ok(Value::from_bigint_ref(bigint));
    }
    Err(throw_type_error(agent))
}

/// ECMAScript `StringToBigInt` for relational BigInt/string comparisons.
///
/// # Errors
/// Returns `TypeError` if the input is not a live string handle.
pub fn string_to_bigint_value(agent: &mut Agent, value: Value) -> Completion<Option<Value>> {
    let string = value
        .as_string_ref()
        .ok_or_else(|| throw_type_error(agent))?;
    let text = agent
        .heap()
        .view()
        .string_view(string)
        .map(lossy_string_from_view)
        .ok_or_else(|| throw_type_error(agent))?;
    let Some((sign, limbs)) = parse_string_to_bigint(&text) else {
        return Ok(None);
    };
    let bigint = agent
        .heap_mut()
        .mutator()
        .alloc_bigint(sign, &limbs, AllocationLifetime::Default);
    Ok(Some(Value::from_bigint_ref(bigint)))
}

/// Formats one BigInt value using the selected radix.
///
/// # Errors
/// Returns `TypeError` when the value is not a live BigInt handle.
pub fn bigint_to_string(agent: &mut Agent, value: Value, radix: u32) -> Completion<String> {
    let bigint = value
        .as_bigint_ref()
        .ok_or_else(|| throw_type_error(agent))?;
    let heap_view = agent.heap().view();
    let Some(view) = heap_view.bigint_view(bigint) else {
        return Err(throw_type_error(agent));
    };
    Ok(if radix == 10 {
        bigint_view_to_string(view)
    } else {
        bigint_view_to_radix_string(view, radix)
    })
}

/// Formats one integral ECMAScript number using the selected non-decimal radix.
#[allow(clippy::cast_possible_truncation)]
pub fn integral_number_to_radix_string(number: f64, radix: u32) -> Option<String> {
    let (sign, limbs) = integral_number_to_bigint(number)?;
    Some(bigint_parts_to_radix_string(sign, &limbs, radix))
}
