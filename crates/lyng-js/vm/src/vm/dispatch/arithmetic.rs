use crate::error::VmResult;
use crate::vm::property_access::ToPrimitiveHint;
use crate::vm::values::{
    bigint_bitwise_and_values, bigint_bitwise_or_values, bigint_bitwise_xor_values,
    bigint_shift_left_values, bigint_shift_right_values, compare_numeric_values, encode_number,
};
use crate::{FrameRecord, Vm, VmError};
use lyng_js_bytecode::Opcode;
use lyng_js_env::Agent;
use lyng_js_host::HostHooks;
use lyng_js_objects::NativeFunctionRegistry;
use lyng_js_ops::{errors, object, pure, read};
use lyng_js_types::{AbruptCompletion, Value};

#[inline]
const fn decode_smi_immediate(raw: u16) -> i16 {
    i16::from_le_bytes(raw.to_le_bytes())
}

impl Vm {
    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    pub(super) fn execute_abc_value_opcode(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        opcode: Opcode,
        left: u16,
        right: u16,
    ) -> VmResult<Value> {
        if let Some(value) =
            self.execute_smi_immediate_opcode(agent, host, registry, frame, opcode, left, right)
        {
            return value;
        }
        if let Some(value) = self.try_primitive_number_binary_opcode(frame, opcode, left, right) {
            return Ok(value);
        }
        match opcode {
            Opcode::Add => self.add_values(agent, host, registry, frame, left, right),
            Opcode::Sub => self.sub_values(agent, host, registry, frame, left, right),
            Opcode::Mul => self.mul_values(agent, host, registry, frame, left, right),
            Opcode::Div => self.div_values(agent, host, registry, frame, left, right),
            Opcode::Mod => self.rem_values(agent, host, registry, frame, left, right),
            Opcode::Exp => self.exp_values(agent, host, registry, frame, left, right),
            Opcode::BitOr => self.bitwise_or(agent, host, registry, frame, left, right),
            Opcode::BitAnd => self.bitwise_and(agent, host, registry, frame, left, right),
            Opcode::BitXor => self.bitwise_xor(agent, host, registry, frame, left, right),
            Opcode::ShiftLeft => self.shift_left(agent, host, registry, frame, left, right),
            Opcode::ShiftRight => self.shift_right(agent, host, registry, frame, left, right),
            Opcode::UnsignedShiftRight => {
                self.unsigned_shift_right(agent, host, registry, frame, left, right)
            }
            Opcode::Equal => {
                let left = self.read_register(frame.registers(), left);
                let right = self.read_register(frame.registers(), right);
                Ok(Value::from_bool(self.loosely_equal(
                    agent, host, registry, frame, left, right,
                )?))
            }
            Opcode::StrictEqual => {
                let left = self.read_register(frame.registers(), left);
                let right = self.read_register(frame.registers(), right);
                if let Some(result) = pure::is_strictly_equal(left, right) {
                    return Ok(Value::from_bool(result));
                }
                Ok(Value::from_bool(
                    read::is_strictly_equal(agent.heap().view(), left, right)
                        .map_err(VmError::Abrupt)?,
                ))
            }
            Opcode::LessThan => {
                self.relational_compare(agent, host, registry, frame, left, right, |ordering| {
                    ordering.is_lt()
                })
            }
            Opcode::LessEqual => {
                self.relational_compare(agent, host, registry, frame, left, right, |ordering| {
                    !ordering.is_gt()
                })
            }
            Opcode::GreaterThan => {
                self.relational_compare(agent, host, registry, frame, left, right, |ordering| {
                    ordering.is_gt()
                })
            }
            Opcode::GreaterEqual => {
                self.relational_compare(agent, host, registry, frame, left, right, |ordering| {
                    !ordering.is_lt()
                })
            }
            _ => unreachable!("caller filters supported ABC value opcodes"),
        }
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    fn execute_smi_immediate_opcode(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        opcode: Opcode,
        left: u16,
        immediate: u16,
    ) -> Option<VmResult<Value>> {
        let immediate = decode_smi_immediate(immediate);
        match opcode {
            Opcode::AddSmi => {
                Some(self.add_value_and_smi(agent, host, registry, frame, left, immediate))
            }
            Opcode::SubSmi => {
                Some(self.sub_value_and_smi(agent, host, registry, frame, left, immediate))
            }
            Opcode::MulSmi => {
                Some(self.mul_value_and_smi(agent, host, registry, frame, left, immediate))
            }
            Opcode::DivSmi => {
                Some(self.div_value_and_smi(agent, host, registry, frame, left, immediate))
            }
            Opcode::ModSmi => {
                Some(self.rem_value_and_smi(agent, host, registry, frame, left, immediate))
            }
            Opcode::BitAndSmi => {
                Some(self.bitwise_and_value_and_smi(agent, host, registry, frame, left, immediate))
            }
            Opcode::EqualZero => {
                let value = self.read_register(frame.registers(), left);
                Some(Ok(Value::from_bool(
                    value.as_f64().is_some_and(|number| number == 0.0),
                )))
            }
            _ => None,
        }
    }

    #[allow(
        clippy::float_cmp,
        reason = "ECMAScript Number equality and exponentiation edge cases require exact IEEE-754 comparisons"
    )]
    fn try_primitive_number_binary_opcode(
        &self,
        frame: FrameRecord,
        opcode: Opcode,
        left: u16,
        right: u16,
    ) -> Option<Value> {
        let left = self.read_register(frame.registers(), left);
        let right = self.read_register(frame.registers(), right);
        if let (Some(left), Some(right)) = (left.as_smi(), right.as_smi()) {
            let value = match opcode {
                Opcode::Add => encode_number(f64::from(left) + f64::from(right)),
                Opcode::Sub => encode_number(f64::from(left) - f64::from(right)),
                Opcode::Mul => encode_number(f64::from(left) * f64::from(right)),
                Opcode::Div => encode_number(f64::from(left) / f64::from(right)),
                Opcode::Mod => encode_number(f64::from(left) % f64::from(right)),
                Opcode::Exp => encode_number(f64::from(left).powf(f64::from(right))),
                Opcode::BitOr => Value::from_smi(left | right),
                Opcode::BitAnd => Value::from_smi(left & right),
                Opcode::BitXor => Value::from_smi(left ^ right),
                Opcode::ShiftLeft => {
                    Value::from_smi(left.wrapping_shl((right & 0x1f).cast_unsigned()))
                }
                Opcode::ShiftRight => {
                    Value::from_smi(left.wrapping_shr((right & 0x1f).cast_unsigned()))
                }
                Opcode::UnsignedShiftRight => {
                    let shifted = left
                        .cast_unsigned()
                        .wrapping_shr((right & 0x1f).cast_unsigned());
                    encode_number(f64::from(shifted))
                }
                Opcode::Equal | Opcode::StrictEqual => Value::from_bool(left == right),
                Opcode::LessThan => Value::from_bool(left < right),
                Opcode::LessEqual => Value::from_bool(left <= right),
                Opcode::GreaterThan => Value::from_bool(left > right),
                Opcode::GreaterEqual => Value::from_bool(left >= right),
                _ => return None,
            };
            return Some(value);
        }
        if !left.is_number() || !right.is_number() {
            return None;
        }
        let left = left
            .as_f64()
            .expect("Number value should expose an f64 payload");
        let right = right
            .as_f64()
            .expect("Number value should expose an f64 payload");
        let value = match opcode {
            Opcode::Add => encode_number(left + right),
            Opcode::Sub => encode_number(left - right),
            Opcode::Mul => encode_number(left * right),
            Opcode::Div => encode_number(left / right),
            Opcode::Mod => encode_number(left % right),
            Opcode::Exp => {
                if left.abs() == 1.0 && right.is_infinite() {
                    Value::from_f64(f64::NAN)
                } else {
                    encode_number(left.powf(right))
                }
            }
            Opcode::BitOr => Value::from_smi(number_to_int32(left) | number_to_int32(right)),
            Opcode::BitAnd => Value::from_smi(number_to_int32(left) & number_to_int32(right)),
            Opcode::BitXor => Value::from_smi(number_to_int32(left) ^ number_to_int32(right)),
            Opcode::ShiftLeft => {
                let left = number_to_int32(left);
                let right = number_to_uint32(right) & 0x1f;
                Value::from_smi(left.wrapping_shl(right))
            }
            Opcode::ShiftRight => {
                let left = number_to_int32(left);
                let right = number_to_uint32(right) & 0x1f;
                Value::from_smi(left >> right)
            }
            Opcode::UnsignedShiftRight => {
                let left = number_to_uint32(left);
                let right = number_to_uint32(right) & 0x1f;
                let result = left >> right;
                i32::try_from(result)
                    .map_or_else(|_| Value::from_f64(f64::from(result)), Value::from_smi)
            }
            Opcode::Equal | Opcode::StrictEqual => Value::from_bool(left == right),
            Opcode::LessThan => Value::from_bool(
                left.partial_cmp(&right)
                    .is_some_and(std::cmp::Ordering::is_lt),
            ),
            Opcode::LessEqual => {
                Value::from_bool(left.partial_cmp(&right).is_some_and(|o| !o.is_gt()))
            }
            Opcode::GreaterThan => Value::from_bool(
                left.partial_cmp(&right)
                    .is_some_and(std::cmp::Ordering::is_gt),
            ),
            Opcode::GreaterEqual => {
                Value::from_bool(left.partial_cmp(&right).is_some_and(|o| !o.is_lt()))
            }
            _ => return None,
        };
        Some(value)
    }

    // ECMAScript IsLooselyEqual only converts an Object operand via ToPrimitive
    // when the other side is String/Number/BigInt/Symbol/Boolean. Object vs
    // null/undefined returns false directly, and Object vs Object falls through
    // to strict (reference) equality.
    fn loosely_equal(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        left: Value,
        right: Value,
    ) -> VmResult<bool> {
        if Self::is_html_dda_equal_nullish(agent, left, right) {
            return Ok(true);
        }
        if (left.is_object() && (right.is_null() || right.is_undefined()))
            || (right.is_object() && (left.is_null() || left.is_undefined()))
        {
            return Ok(false);
        }
        if left.is_object() && !right.is_object() {
            let left =
                self.to_primitive(agent, host, registry, frame, left, ToPrimitiveHint::Default)?;
            return self.loosely_equal(agent, host, registry, frame, left, right);
        }
        if right.is_object() && !left.is_object() {
            let right = self.to_primitive(
                agent,
                host,
                registry,
                frame,
                right,
                ToPrimitiveHint::Default,
            )?;
            return self.loosely_equal(agent, host, registry, frame, left, right);
        }

        read::is_loosely_equal(agent.heap().view(), left, right).map_err(VmError::Abrupt)
    }

    fn is_html_dda_equal_nullish(agent: &Agent, left: Value, right: Value) -> bool {
        fn is_html_dda(agent: &Agent, value: Value) -> bool {
            value
                .as_object_ref()
                .is_some_and(|object| agent.objects().is_html_dda_object(object))
        }

        (is_html_dda(agent, left) && (right.is_null() || right.is_undefined()))
            || (is_html_dda(agent, right) && (left.is_null() || left.is_undefined()))
    }

    fn numeric_register_value(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        register: u16,
    ) -> VmResult<Value> {
        let primitive = self.to_primitive(
            agent,
            host,
            registry,
            frame,
            self.read_register(frame.registers(), register),
            ToPrimitiveHint::Number,
        )?;
        Self::to_numeric_primitive(agent, primitive)
    }

    fn to_numeric_primitive(agent: &mut Agent, value: Value) -> VmResult<Value> {
        read::to_numeric(agent.heap().view(), value)
            .map_err(|abrupt| numeric_conversion_error(agent, abrupt))
    }

    pub(super) fn update_register_value(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        register: u16,
        increment: bool,
    ) -> VmResult<(Value, Value)> {
        let numeric = self.numeric_register_value(agent, host, registry, frame, register)?;
        let updated = Self::update_numeric_value(agent, numeric, increment)?;
        Ok((numeric, updated))
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    pub(super) fn relational_compare(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        left_register: u16,
        right_register: u16,
        compare_op: impl FnOnce(std::cmp::Ordering) -> bool,
    ) -> VmResult<Value> {
        let left = self.to_primitive(
            agent,
            host,
            registry,
            frame,
            self.read_register(frame.registers(), left_register),
            ToPrimitiveHint::Number,
        )?;
        let right = self.to_primitive(
            agent,
            host,
            registry,
            frame,
            self.read_register(frame.registers(), right_register),
            ToPrimitiveHint::Number,
        )?;
        if left.is_string() && right.is_string() {
            let left_units = Self::value_to_string_code_units(agent, left)?;
            let right_units = Self::value_to_string_code_units(agent, right)?;
            return Ok(Value::from_bool(compare_op(left_units.cmp(&right_units))));
        }
        if left.is_bigint() && right.is_string() {
            let Some(right) =
                object::string_to_bigint_value(agent, right).map_err(VmError::Abrupt)?
            else {
                return Ok(Value::from_bool(false));
            };
            let ordering = compare_numeric_values(agent, left, right)?
                .expect("BigInt/StringToBigInt comparison must be ordered");
            return Ok(Value::from_bool(compare_op(ordering)));
        }
        if left.is_string() && right.is_bigint() {
            let Some(left) =
                object::string_to_bigint_value(agent, left).map_err(VmError::Abrupt)?
            else {
                return Ok(Value::from_bool(false));
            };
            let ordering = compare_numeric_values(agent, left, right)?
                .expect("StringToBigInt/BigInt comparison must be ordered");
            return Ok(Value::from_bool(compare_op(ordering)));
        }
        let left = Self::to_numeric_primitive(agent, left)?;
        let right = Self::to_numeric_primitive(agent, right)?;
        let Some(ordering) = compare_numeric_values(agent, left, right)? else {
            return Ok(Value::from_bool(false));
        };
        Ok(Value::from_bool(compare_op(ordering)))
    }

    pub(super) fn bitwise_and(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        left_register: u16,
        right_register: u16,
    ) -> VmResult<Value> {
        let left = self.numeric_register_value(agent, host, registry, frame, left_register)?;
        let right = self.numeric_register_value(agent, host, registry, frame, right_register)?;
        if left.is_bigint() || right.is_bigint() {
            if !left.is_bigint() || !right.is_bigint() {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
            return bigint_bitwise_and_values(agent, left, right);
        }
        let left = number_to_int32(numeric_value_to_f64(left));
        let right = number_to_int32(numeric_value_to_f64(right));
        Ok(Value::from_smi(left & right))
    }

    pub(super) fn bitwise_and_value_and_smi(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        left_register: u16,
        immediate: i16,
    ) -> VmResult<Value> {
        let left = self.read_register(frame.registers(), left_register);
        if let Some(left) = left.as_smi() {
            return Ok(Value::from_smi(left & i32::from(immediate)));
        }
        if left.is_number() {
            let left = number_to_int32(numeric_value_to_f64(left));
            return Ok(Value::from_smi(left & i32::from(immediate)));
        }
        let left = self.numeric_register_value(agent, host, registry, frame, left_register)?;
        if left.is_bigint() {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        let left = number_to_int32(numeric_value_to_f64(left));
        Ok(Value::from_smi(left & i32::from(immediate)))
    }

    pub(super) fn div_value_and_smi(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        left_register: u16,
        immediate: i16,
    ) -> VmResult<Value> {
        let left = self.read_register(frame.registers(), left_register);
        if let Some(left) = left.as_smi() {
            return Ok(encode_number(f64::from(left) / f64::from(immediate)));
        }
        if left.is_number() {
            return Ok(encode_number(
                numeric_value_to_f64(left) / f64::from(immediate),
            ));
        }
        let left = self.numeric_register_value(agent, host, registry, frame, left_register)?;
        if left.is_bigint() {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        Ok(encode_number(
            numeric_value_to_f64(left) / f64::from(immediate),
        ))
    }

    pub(super) fn rem_value_and_smi(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        left_register: u16,
        immediate: i16,
    ) -> VmResult<Value> {
        let left = self.read_register(frame.registers(), left_register);
        if let Some(left) = left.as_smi() {
            return Ok(encode_number(f64::from(left) % f64::from(immediate)));
        }
        if left.is_number() {
            return Ok(encode_number(
                numeric_value_to_f64(left) % f64::from(immediate),
            ));
        }
        let left = self.numeric_register_value(agent, host, registry, frame, left_register)?;
        if left.is_bigint() {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        Ok(encode_number(
            numeric_value_to_f64(left) % f64::from(immediate),
        ))
    }

    pub(super) fn bitwise_or(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        left_register: u16,
        right_register: u16,
    ) -> VmResult<Value> {
        let left = self.numeric_register_value(agent, host, registry, frame, left_register)?;
        let right = self.numeric_register_value(agent, host, registry, frame, right_register)?;
        if left.is_bigint() || right.is_bigint() {
            if !left.is_bigint() || !right.is_bigint() {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
            return bigint_bitwise_or_values(agent, left, right);
        }
        let left = number_to_int32(numeric_value_to_f64(left));
        let right = number_to_int32(numeric_value_to_f64(right));
        Ok(Value::from_smi(left | right))
    }

    pub(super) fn bitwise_xor(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        left_register: u16,
        right_register: u16,
    ) -> VmResult<Value> {
        let left = self.numeric_register_value(agent, host, registry, frame, left_register)?;
        let right = self.numeric_register_value(agent, host, registry, frame, right_register)?;
        if left.is_bigint() || right.is_bigint() {
            if !left.is_bigint() || !right.is_bigint() {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
            return bigint_bitwise_xor_values(agent, left, right);
        }
        let left = number_to_int32(numeric_value_to_f64(left));
        let right = number_to_int32(numeric_value_to_f64(right));
        Ok(Value::from_smi(left ^ right))
    }

    pub(super) fn shift_left(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        left_register: u16,
        right_register: u16,
    ) -> VmResult<Value> {
        let left = self.numeric_register_value(agent, host, registry, frame, left_register)?;
        let right = self.numeric_register_value(agent, host, registry, frame, right_register)?;
        if left.is_bigint() || right.is_bigint() {
            if !left.is_bigint() || !right.is_bigint() {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
            return bigint_shift_left_values(agent, left, right);
        }
        let left = number_to_int32(numeric_value_to_f64(left));
        let right = number_to_uint32(numeric_value_to_f64(right)) & 0x1f;
        Ok(Value::from_smi(left.wrapping_shl(right)))
    }

    pub(super) fn shift_right(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        left_register: u16,
        right_register: u16,
    ) -> VmResult<Value> {
        let left = self.numeric_register_value(agent, host, registry, frame, left_register)?;
        let right = self.numeric_register_value(agent, host, registry, frame, right_register)?;
        if left.is_bigint() || right.is_bigint() {
            if !left.is_bigint() || !right.is_bigint() {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
            return bigint_shift_right_values(agent, left, right);
        }
        let left = number_to_int32(numeric_value_to_f64(left));
        let right = number_to_uint32(numeric_value_to_f64(right)) & 0x1f;
        Ok(Value::from_smi(left >> right))
    }

    pub(super) fn unsigned_shift_right(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        left_register: u16,
        right_register: u16,
    ) -> VmResult<Value> {
        let left = self.numeric_register_value(agent, host, registry, frame, left_register)?;
        let right = self.numeric_register_value(agent, host, registry, frame, right_register)?;
        if left.is_bigint() || right.is_bigint() {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        let left = number_to_uint32(numeric_value_to_f64(left));
        let right = number_to_uint32(numeric_value_to_f64(right)) & 0x1f;
        let result = left >> right;
        Ok(i32::try_from(result)
            .map_or_else(|_| Value::from_f64(f64::from(result)), Value::from_smi))
    }
}

fn numeric_conversion_error(agent: &mut Agent, abrupt: AbruptCompletion) -> VmError {
    match abrupt {
        AbruptCompletion::Throw(value) if value.is_undefined() => {
            VmError::Abrupt(errors::throw_type_error(agent))
        }
        abrupt => VmError::Abrupt(abrupt),
    }
}

fn numeric_value_to_f64(value: Value) -> f64 {
    value
        .as_f64()
        .expect("numeric non-BigInt Value should expose an f64 payload")
}

fn number_to_int32(number: f64) -> i32 {
    if !number.is_finite() || number == 0.0 {
        return 0;
    }
    let truncated = number.trunc();
    let modulo = truncated.rem_euclid(4_294_967_296.0);
    if modulo >= 2_147_483_648.0 {
        number_to_i32_after_range_check(modulo - 4_294_967_296.0)
    } else {
        number_to_i32_after_range_check(modulo)
    }
}

fn number_to_uint32(number: f64) -> u32 {
    if !number.is_finite() || number == 0.0 {
        return 0;
    }
    let truncated = number.trunc();
    number_to_u32_after_range_check(truncated.rem_euclid(4_294_967_296.0))
}

const fn number_to_i32_after_range_check(number: f64) -> i32 {
    #[allow(
        clippy::cast_possible_truncation,
        reason = "caller applies ECMA-262 ToInt32 modulo semantics before narrowing to i32"
    )]
    let integer = number as i32;
    integer
}

const fn number_to_u32_after_range_check(number: f64) -> u32 {
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "caller applies ECMA-262 ToUint32 modulo semantics before narrowing to u32"
    )]
    let integer = number as u32;
    integer
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RegisterWindow;
    use lyng_js_env::ExecutionContextKind;
    use lyng_js_types::{CodeRef, EnvironmentRef, RealmRef};

    fn test_frame() -> FrameRecord {
        FrameRecord::new(
            CodeRef::from_raw(1).expect("test code ref should be non-zero"),
            0,
            RegisterWindow::new(0, 2),
            None,
            RealmRef::from_raw(1).expect("test realm ref should be non-zero"),
            EnvironmentRef::from_raw(1).expect("test lexical env should be non-zero"),
            EnvironmentRef::from_raw(1).expect("test variable env should be non-zero"),
            ExecutionContextKind::Script,
        )
    }

    #[test]
    fn number_binary_opcode_fast_path_handles_double_operands() {
        let mut vm = Vm::new();
        vm.register_stack = vec![Value::from_f64(1.5), Value::from_f64(2.25)];
        vm.register_stack_top = vm.register_stack.len();

        let value = vm.try_primitive_number_binary_opcode(test_frame(), Opcode::Add, 0, 1);

        assert_eq!(
            value.and_then(Value::as_f64),
            Some(3.75),
            "primitive number addition should avoid the generic conversion path"
        );
    }
}
