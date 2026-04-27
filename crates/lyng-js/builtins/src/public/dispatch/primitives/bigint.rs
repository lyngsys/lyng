use super::super::{
    map_completion, radix_argument, range_error, string_value, to_bigint_for_builtin,
    to_index_for_builtin, type_error, BuiltinToPrimitiveBridge, PublicBuiltinDispatchContext,
};
use crate::BuiltinInvocation;
use lyng_js_env::Agent;
use lyng_js_gc::{AllocationLifetime, BigIntSign};
use lyng_js_objects::PrimitiveWrapperKind;
use lyng_js_ops::object;
use lyng_js_types::{BuiltinFunctionId, Value};

pub(super) fn dispatch_bigint_builtin<Cx: PublicBuiltinDispatchContext>(
    context: &mut Cx,
    entry: BuiltinFunctionId,
    invocation: BuiltinInvocation<'_>,
) -> Result<Option<Value>, Cx::Error> {
    if entry == super::super::bigint_builtin() {
        return bigint_builtin(context, invocation).map(Some);
    }
    if entry == super::super::bigint_as_int_n_builtin() {
        return bigint_as_int_n_builtin(context, invocation).map(Some);
    }
    if entry == super::super::bigint_as_uint_n_builtin() {
        return bigint_as_uint_n_builtin(context, invocation).map(Some);
    }
    if entry == super::super::bigint_to_string_builtin() {
        return bigint_to_string_builtin(context, invocation).map(Some);
    }
    if entry == super::super::bigint_value_of_builtin() {
        return bigint_value_of_builtin(context, invocation).map(Some);
    }
    Ok(None)
}

pub(super) fn bigint_to_number_value<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<Value, Cx::Error> {
    let text = {
        let agent = cx.agent();
        object::bigint_to_string(agent, value, 10)
    };
    let text = map_completion(cx, text)?;
    let number = text.parse::<f64>().unwrap_or_else(|_| {
        if text.starts_with('-') {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        }
    });
    Ok(Value::from_f64(number))
}

fn bigint_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    if invocation.new_target().is_some() {
        return Err(type_error(cx));
    }
    let argument = invocation
        .arguments()
        .first()
        .copied()
        .unwrap_or(Value::undefined());
    let primitive = {
        let mut bridge = BuiltinToPrimitiveBridge { cx };
        object::to_primitive(&mut bridge, argument, object::ToPrimitiveHint::Number)?
    };
    let bigint = {
        let agent = cx.agent();
        object::primitive_to_bigint(agent, primitive)
    };
    map_completion(cx, bigint)
}

const BIGINT_WIDTH_EXACT_LIMIT_BITS: u64 = 4_096;

fn bigint_as_int_n_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let bits = to_index_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let bigint = to_bigint_for_builtin(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    if bits == 0 {
        return Ok(bigint_zero_value(cx));
    }
    let (sign, limbs) = bigint_parts(cx.agent(), bigint).ok_or_else(|| type_error(cx))?;
    if bigint_fits_signed_width(sign, &limbs, bits) {
        return Ok(bigint);
    }
    if bits > BIGINT_WIDTH_EXACT_LIMIT_BITS {
        return Err(range_error(cx));
    }
    let unsigned = bigint_to_uint_n_limbs(sign, &limbs, bits);
    let negative = bigint_width_sign_bit(&unsigned, bits);
    if negative {
        let magnitude = twos_complement_width_magnitude(unsigned, bits);
        Ok(bigint_from_parts(cx, BigIntSign::Negative, &magnitude))
    } else {
        Ok(bigint_from_parts(cx, BigIntSign::NonNegative, &unsigned))
    }
}

fn bigint_as_uint_n_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let bits = to_index_for_builtin(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let bigint = to_bigint_for_builtin(
        cx,
        invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    if bits == 0 {
        return Ok(bigint_zero_value(cx));
    }
    let (sign, limbs) = bigint_parts(cx.agent(), bigint).ok_or_else(|| type_error(cx))?;
    if sign == BigIntSign::NonNegative && bigint_magnitude_bit_length(&limbs) <= bits {
        return Ok(bigint);
    }
    if bits > BIGINT_WIDTH_EXACT_LIMIT_BITS {
        return Err(range_error(cx));
    }
    let unsigned = bigint_to_uint_n_limbs(sign, &limbs, bits);
    Ok(bigint_from_parts(cx, BigIntSign::NonNegative, &unsigned))
}

fn bigint_parts(agent: &Agent, value: Value) -> Option<(BigIntSign, Vec<u64>)> {
    let bigint = value.as_bigint_ref()?;
    let view = agent.heap().view().bigint_view(bigint)?;
    let mut limbs = Vec::with_capacity(view.limb_count() as usize);
    for index in 0..view.limb_count() {
        limbs.push(view.limb_at(index as usize).unwrap_or(0));
    }
    normalize_bigint_limbs(&mut limbs);
    Some((normalize_bigint_sign(view.sign(), &limbs), limbs))
}

fn bigint_zero_value<Cx: PublicBuiltinDispatchContext>(cx: &mut Cx) -> Value {
    bigint_from_parts(cx, BigIntSign::NonNegative, &[])
}

fn bigint_from_parts<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    sign: BigIntSign,
    limbs: &[u64],
) -> Value {
    let mut normalized = limbs.to_vec();
    normalize_bigint_limbs(&mut normalized);
    let sign = normalize_bigint_sign(sign, &normalized);
    let bigint = cx.agent().heap_mut().mutator().alloc_bigint(
        sign,
        &normalized,
        AllocationLifetime::Default,
    );
    Value::from_bigint_ref(bigint)
}

fn bigint_to_uint_n_limbs(sign: BigIntSign, limbs: &[u64], bits: u64) -> Vec<u64> {
    let width = usize::try_from(bits.div_ceil(64)).expect("exact BigInt width should fit");
    let mut result = vec![0; width.max(1)];
    let copied = result.len().min(limbs.len());
    result[..copied].copy_from_slice(&limbs[..copied]);
    if normalize_bigint_sign(sign, limbs) == BigIntSign::Negative {
        for limb in &mut result {
            *limb = !*limb;
        }
        add_one_to_limbs(&mut result);
    }
    mask_bigint_width(&mut result, bits);
    normalize_bigint_limbs(&mut result);
    result
}

fn twos_complement_width_magnitude(mut bits: Vec<u64>, width_bits: u64) -> Vec<u64> {
    for limb in &mut bits {
        *limb = !*limb;
    }
    mask_bigint_width(&mut bits, width_bits);
    add_one_to_limbs(&mut bits);
    mask_bigint_width(&mut bits, width_bits);
    normalize_bigint_limbs(&mut bits);
    bits
}

fn bigint_width_sign_bit(limbs: &[u64], bits: u64) -> bool {
    let bit = bits - 1;
    let limb_index = usize::try_from(bit / 64).expect("exact BigInt width should fit");
    let bit_index = bit % 64;
    limbs
        .get(limb_index)
        .is_some_and(|limb| (limb & (1_u64 << bit_index)) != 0)
}

fn mask_bigint_width(limbs: &mut [u64], bits: u64) {
    let remainder = bits % 64;
    if remainder == 0 {
        return;
    }
    if let Some(last) = limbs.last_mut() {
        *last &= (1_u64 << remainder) - 1;
    }
}

fn add_one_to_limbs(limbs: &mut [u64]) {
    let mut carry = true;
    for limb in limbs {
        if !carry {
            return;
        }
        let (next, overflow) = limb.overflowing_add(1);
        *limb = next;
        carry = overflow;
    }
}

fn normalize_bigint_limbs(limbs: &mut Vec<u64>) {
    while limbs.last() == Some(&0) {
        limbs.pop();
    }
}

fn normalize_bigint_sign(sign: BigIntSign, limbs: &[u64]) -> BigIntSign {
    if limbs.is_empty() {
        BigIntSign::NonNegative
    } else {
        sign
    }
}

fn bigint_magnitude_bit_length(limbs: &[u64]) -> u64 {
    let Some(last) = limbs.last().copied() else {
        return 0;
    };
    let high_bits = 64 - u64::from(last.leading_zeros());
    ((limbs.len() as u64 - 1) * 64) + high_bits
}

fn bigint_fits_signed_width(sign: BigIntSign, limbs: &[u64], bits: u64) -> bool {
    match normalize_bigint_sign(sign, limbs) {
        BigIntSign::NonNegative => bigint_magnitude_bit_length(limbs) < bits,
        BigIntSign::Negative => bigint_magnitude_le_power_of_two(limbs, bits - 1),
    }
}

fn bigint_magnitude_le_power_of_two(limbs: &[u64], exponent: u64) -> bool {
    let limb_index = usize::try_from(exponent / 64).expect("BigInt exponent should fit");
    if limbs.len() > limb_index + 1 {
        return false;
    }
    if limbs.len() <= limb_index {
        return true;
    }
    let bit = exponent % 64;
    let limit = 1_u64 << bit;
    let high = limbs[limb_index];
    high < limit || (high == limit && limbs[..limb_index].iter().all(|limb| *limb == 0))
}

fn bigint_to_string_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_primitive_wrapper_value(
            agent,
            invocation.this_value(),
            PrimitiveWrapperKind::BigInt,
        )
    };
    let bigint_value = map_completion(cx, value)?;
    let radix = radix_argument(
        cx,
        invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined()),
    )?;
    let text = {
        let agent = cx.agent();
        object::bigint_to_string(agent, bigint_value, radix)
    };
    let text = map_completion(cx, text)?;
    Ok(string_value(cx, &text))
}

fn bigint_value_of_builtin<Cx: PublicBuiltinDispatchContext>(
    cx: &mut Cx,
    invocation: BuiltinInvocation<'_>,
) -> Result<Value, Cx::Error> {
    let value = {
        let agent = cx.agent();
        object::require_primitive_wrapper_value(
            agent,
            invocation.this_value(),
            PrimitiveWrapperKind::BigInt,
        )
    };
    map_completion(cx, value)
}
