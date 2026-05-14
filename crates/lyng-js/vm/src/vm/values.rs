use super::{
    Agent, AtomId, BytecodeFunctionId, CodeRef, CompiledAtom, ConstantValue, EnvironmentRef,
    FrameRecord, HostHooks, InstalledFunction, NativeFunctionRegistry, ObjectRef, Opcode, RealmRef,
    Value, Vm, VmError, VmResult,
};
use crate::vm::property_access::ToPrimitiveHint;
use lyng_js_gc::{AllocationLifetime, BigIntSign, PrimitiveStringView, StringEncoding};
use lyng_js_ops::{errors, number_to_string, object, read};
use lyng_js_types::{AbruptCompletion, PropertyKey, StringRef};

impl Vm {
    #[inline]
    pub(super) fn canonical_atom_for_code(&self, code: CodeRef, atom: AtomId) -> AtomId {
        self.installed
            .get(code_index(code))
            .and_then(Option::as_ref)
            .map_or(atom, |installed| installed.canonical_atom(atom))
    }

    fn property_atom_from_text(&mut self, agent: &mut Agent, text: &str) -> AtomId {
        if let Some(atom) = self.preferred_atoms_by_text.get(text).copied() {
            return atom;
        }
        let atom = agent.atoms_mut().intern_collectible(text);
        self.preferred_atoms_by_text.insert(text.into(), atom);
        self.atom_texts.entry(atom).or_insert_with(|| text.into());
        atom
    }

    fn property_atom_from_utf16(&mut self, agent: &mut Agent, units: &[u16]) -> AtomId {
        if let Ok(text) = String::from_utf16(units) {
            return self.property_atom_from_text(agent, &text);
        }
        agent.atoms_mut().intern_collectible_utf16(units)
    }

    fn primitive_property_key_atom(&mut self, agent: &mut Agent, value: Value) -> Option<AtomId> {
        let text = if value.is_undefined() {
            "undefined".to_owned()
        } else if value.is_null() {
            "null".to_owned()
        } else if let Some(boolean) = value.as_bool() {
            if boolean {
                "true".to_owned()
            } else {
                "false".to_owned()
            }
        } else if value.is_number() {
            number_to_string(value.as_f64()?)
        } else {
            return None;
        };
        Some(self.property_atom_from_text(agent, &text))
    }

    pub(super) fn add_values(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        left_register: u16,
        right_register: u16,
    ) -> VmResult<Value> {
        self.add_value_operands(
            agent,
            host,
            registry,
            frame,
            self.read_register(frame.registers(), left_register),
            self.read_register(frame.registers(), right_register),
        )
    }

    pub(super) fn add_value_and_smi(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        left_register: u16,
        immediate: i16,
    ) -> VmResult<Value> {
        let left = self.read_register(frame.registers(), left_register);
        if let Some(left) = left.as_smi() {
            return Ok(encode_number(f64::from(left) + f64::from(immediate)));
        }
        if left.is_number() {
            let left = left
                .as_f64()
                .expect("Number value should expose an f64 payload");
            return Ok(encode_number(left + f64::from(immediate)));
        }
        self.add_value_operands(
            agent,
            host,
            registry,
            frame,
            left,
            Value::from_smi(i32::from(immediate)),
        )
    }

    fn add_value_operands(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        left: Value,
        right: Value,
    ) -> VmResult<Value> {
        let left =
            self.to_primitive(agent, host, registry, frame, left, ToPrimitiveHint::Default)?;
        let right = self.to_primitive(
            agent,
            host,
            registry,
            frame,
            right,
            ToPrimitiveHint::Default,
        )?;
        if left.is_string() || right.is_string() {
            if let (Some(left_string), Some(right_string)) =
                (left.as_string_ref(), right.as_string_ref())
            {
                return Ok(Value::from_string_ref(concat_string_refs(
                    agent,
                    left_string,
                    right_string,
                )?));
            }

            let mut units = Vec::with_capacity(
                Self::value_string_code_unit_len(agent, left)?
                    + Self::value_string_code_unit_len(agent, right)?,
            );
            Self::append_value_string_code_units(agent, left, &mut units)?;
            Self::append_value_string_code_units(agent, right, &mut units)?;
            return Ok(Value::from_string_ref(alloc_code_unit_string(
                agent, &units, None,
            )));
        }
        if left.is_bigint() || right.is_bigint() {
            if !left.is_bigint() || !right.is_bigint() {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
            return bigint_add_values(agent, left, right);
        }

        let left = to_f64_number_or_type_error(agent, left)?;
        let right = to_f64_number_or_type_error(agent, right)?;
        Ok(encode_number(left + right))
    }

    #[inline]
    pub(crate) fn alloc_code_unit_string_value(agent: &mut Agent, units: &[u16]) -> Value {
        Value::from_string_ref(alloc_code_unit_string(agent, units, None))
    }

    pub(super) fn sub_values(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        left_register: u16,
        right_register: u16,
    ) -> VmResult<Value> {
        let left = self.numeric_value(agent, host, registry, frame, left_register)?;
        let right = self.numeric_value(agent, host, registry, frame, right_register)?;
        if left.is_bigint() || right.is_bigint() {
            if !left.is_bigint() || !right.is_bigint() {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
            return bigint_subtract_values(agent, left, right);
        }
        let left = to_f64_number_or_type_error(agent, left)?;
        let right = to_f64_number_or_type_error(agent, right)?;
        Ok(encode_number(left - right))
    }

    pub(super) fn sub_value_and_smi(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        left_register: u16,
        immediate: i16,
    ) -> VmResult<Value> {
        let left = self.read_register(frame.registers(), left_register);
        if let Some(left) = left.as_smi() {
            return Ok(encode_number(f64::from(left) - f64::from(immediate)));
        }
        if left.is_number() {
            let left = left
                .as_f64()
                .expect("Number value should expose an f64 payload");
            return Ok(encode_number(left - f64::from(immediate)));
        }
        let left = self.numeric_value_operand(agent, host, registry, frame, left)?;
        if left.is_bigint() {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        let left = to_f64_number_or_type_error(agent, left)?;
        Ok(encode_number(left - f64::from(immediate)))
    }

    pub(super) fn negate_value(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        register: u16,
    ) -> VmResult<Value> {
        let value = self.to_primitive(
            agent,
            host,
            registry,
            frame,
            self.read_register(frame.registers(), register),
            ToPrimitiveHint::Number,
        )?;
        if value.is_bigint() {
            return bigint_negate_value(agent, value);
        }
        let number = to_f64_number_or_type_error(agent, value)?;
        Ok(encode_number(-number))
    }

    pub(super) fn bitwise_not_value(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        register: u16,
    ) -> VmResult<Value> {
        let value = self.numeric_value(agent, host, registry, frame, register)?;
        if value.is_bigint() {
            return bigint_bitwise_not_value(agent, value);
        }
        let number = number_to_int32(
            value
                .as_f64()
                .expect("numeric non-BigInt Value should expose an f64 payload"),
        );
        Ok(Value::from_smi(!number))
    }

    pub(super) fn update_numeric_value(
        agent: &mut Agent,
        value: Value,
        increment: bool,
    ) -> VmResult<Value> {
        if value.is_bigint() {
            let (sign, limbs) = bigint_value_parts(agent, value)?;
            let unit_sign = if increment {
                BigIntSign::NonNegative
            } else {
                BigIntSign::Negative
            };
            let (sign, limbs) = bigint_add_signed_parts(sign, &limbs, unit_sign, &[1]);
            return Ok(alloc_bigint_value(agent, sign, &limbs));
        }

        let number = value
            .as_f64()
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let updated = if increment {
            number + 1.0
        } else {
            number - 1.0
        };
        Ok(encode_number(updated))
    }

    pub(super) fn mul_values(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        left_register: u16,
        right_register: u16,
    ) -> VmResult<Value> {
        let left = self.numeric_value(agent, host, registry, frame, left_register)?;
        let right = self.numeric_value(agent, host, registry, frame, right_register)?;
        if left.is_bigint() || right.is_bigint() {
            if !left.is_bigint() || !right.is_bigint() {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
            return bigint_multiply_values(agent, left, right);
        }
        let left = to_f64_number_or_type_error(agent, left)?;
        let right = to_f64_number_or_type_error(agent, right)?;
        Ok(encode_number(left * right))
    }

    pub(super) fn mul_value_and_smi(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        left_register: u16,
        immediate: i16,
    ) -> VmResult<Value> {
        let left = self.read_register(frame.registers(), left_register);
        if let Some(left) = left.as_smi() {
            return Ok(encode_number(f64::from(left) * f64::from(immediate)));
        }
        if left.is_number() {
            let left = left
                .as_f64()
                .expect("Number value should expose an f64 payload");
            return Ok(encode_number(left * f64::from(immediate)));
        }
        let left = self.numeric_value_operand(agent, host, registry, frame, left)?;
        if left.is_bigint() {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        let left = to_f64_number_or_type_error(agent, left)?;
        Ok(encode_number(left * f64::from(immediate)))
    }

    pub(super) fn div_values(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        left_register: u16,
        right_register: u16,
    ) -> VmResult<Value> {
        let left = self.numeric_value(agent, host, registry, frame, left_register)?;
        let right = self.numeric_value(agent, host, registry, frame, right_register)?;
        if left.is_bigint() || right.is_bigint() {
            if !left.is_bigint() || !right.is_bigint() {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
            return bigint_divide_values(agent, left, right);
        }
        let left = to_f64_number_or_type_error(agent, left)?;
        let right = to_f64_number_or_type_error(agent, right)?;
        Ok(encode_number(left / right))
    }

    pub(super) fn rem_values(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        left_register: u16,
        right_register: u16,
    ) -> VmResult<Value> {
        let left = self.numeric_value(agent, host, registry, frame, left_register)?;
        let right = self.numeric_value(agent, host, registry, frame, right_register)?;
        if left.is_bigint() || right.is_bigint() {
            if !left.is_bigint() || !right.is_bigint() {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
            return bigint_remainder_values(agent, left, right);
        }
        let left = to_f64_number_or_type_error(agent, left)?;
        let right = to_f64_number_or_type_error(agent, right)?;
        Ok(encode_number(left % right))
    }

    #[allow(
        clippy::float_cmp,
        reason = "ECMAScript exponentiation has an exact ±1 and infinity special case"
    )]
    pub(super) fn exp_values(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        left_register: u16,
        right_register: u16,
    ) -> VmResult<Value> {
        let left = self.numeric_value(agent, host, registry, frame, left_register)?;
        let right = self.numeric_value(agent, host, registry, frame, right_register)?;
        if left.is_bigint() || right.is_bigint() {
            if !left.is_bigint() || !right.is_bigint() {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
            return bigint_exponentiate_values(agent, left, right);
        }
        let left = to_f64_number_or_type_error(agent, left)?;
        let right = to_f64_number_or_type_error(agent, right)?;
        if left.abs() == 1.0 && right.is_infinite() {
            return Ok(Value::from_f64(f64::NAN));
        }
        Ok(encode_number(left.powf(right)))
    }

    fn numeric_value(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        register: u16,
    ) -> VmResult<Value> {
        self.numeric_value_operand(
            agent,
            host,
            registry,
            frame,
            self.read_register(frame.registers(), register),
        )
    }

    fn numeric_value_operand(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: &FrameRecord,
        value: Value,
    ) -> VmResult<Value> {
        let primitive =
            self.to_primitive(agent, host, registry, frame, value, ToPrimitiveHint::Number)?;
        read::to_numeric(agent.heap().view(), primitive)
            .map_err(|abrupt| numeric_conversion_error(agent, abrupt))
    }

    pub(crate) fn value_to_string_text(agent: &mut Agent, value: Value) -> VmResult<String> {
        if value.is_undefined() {
            return Ok("undefined".to_owned());
        }
        if value.is_null() {
            return Ok("null".to_owned());
        }
        if let Some(boolean) = value.as_bool() {
            return Ok(if boolean {
                "true".to_owned()
            } else {
                "false".to_owned()
            });
        }
        if value.is_number() {
            return Ok(number_to_string(
                value
                    .as_f64()
                    .expect("numeric Value should expose an f64 payload"),
            ));
        }
        if value.is_bigint() {
            return object::bigint_to_string(agent, value, 10).map_err(VmError::Abrupt);
        }
        if let Some(string) = value.as_string_ref() {
            let heap_view = agent.heap().view();
            let Some(view) = heap_view.string_view(string) else {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            };
            return Ok(decode_string_view(&view));
        }

        Err(VmError::Abrupt(errors::throw_type_error(agent)))
    }

    pub(super) fn value_to_string_code_units(
        agent: &mut Agent,
        value: Value,
    ) -> VmResult<Vec<u16>> {
        if let Some(string) = value.as_string_ref() {
            let heap_view = agent.heap().view();
            let Some(view) = heap_view.string_view(string) else {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            };
            return utf16_code_units(&view)
                .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)));
        }

        Ok(Self::value_to_string_text(agent, value)?
            .encode_utf16()
            .collect())
    }

    fn value_string_code_unit_len(agent: &mut Agent, value: Value) -> VmResult<usize> {
        if let Some(string) = value.as_string_ref() {
            let heap_view = agent.heap().view();
            let Some(view) = heap_view.string_view(string) else {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            };
            return Ok(view.code_unit_len() as usize);
        }

        Ok(Self::value_to_string_text(agent, value)?
            .encode_utf16()
            .count())
    }

    pub(super) fn append_value_string_code_units(
        agent: &mut Agent,
        value: Value,
        output: &mut Vec<u16>,
    ) -> VmResult<()> {
        if let Some(string) = value.as_string_ref() {
            return append_string_ref_code_units(agent, string, output);
        }

        output.extend(Self::value_to_string_text(agent, value)?.encode_utf16());
        Ok(())
    }

    #[inline]
    pub(super) const fn decode_abc_operands(
        installed: &InstalledFunction,
        instruction_offset: u32,
        opcode: Opcode,
        a: u16,
        b: u16,
        c: u16,
    ) -> (u16, u16, u16) {
        let _ = (installed, instruction_offset, opcode);
        (a, b, c)
    }

    #[inline]
    pub(super) const fn decode_abx_operands(
        installed: &InstalledFunction,
        instruction_offset: u32,
        a: u16,
        bx: u32,
    ) -> (u16, u32) {
        let _ = (installed, instruction_offset);
        (a, bx)
    }

    pub(super) fn object_register(
        &self,
        frame: &FrameRecord,
        register: u16,
    ) -> VmResult<ObjectRef> {
        let value = self.read_register(frame.registers(), register);
        Self::require_object(frame, value)
    }

    pub(super) fn require_object(frame: &FrameRecord, value: Value) -> VmResult<ObjectRef> {
        value
            .as_object_ref()
            .ok_or_else(|| VmError::ExpectedObject {
                code: frame.code(),
                instruction_offset: frame.instruction_offset(),
                value,
            })
    }

    pub(super) fn value_to_property_key(
        &mut self,
        agent: &mut Agent,
        _frame: &FrameRecord,
        code: CodeRef,
        instruction_offset: u32,
        value: Value,
    ) -> VmResult<PropertyKey> {
        if let Some(index) = value.as_smi().and_then(|index| u32::try_from(index).ok()) {
            return Ok(PropertyKey::Index(index));
        }
        if let Some(number) = value.as_f64()
            && let Some(index) = number_to_array_index(number)
        {
            return Ok(PropertyKey::Index(index));
        }
        if let Some(string) = value.as_string_ref() {
            return self.string_ref_to_property_key(agent, code, instruction_offset, value, string);
        }
        if let Some(symbol) = value.as_symbol_ref() {
            return Ok(PropertyKey::from_symbol(symbol));
        }
        if value.is_bigint() {
            let text = object::bigint_to_string(agent, value, 10).map_err(VmError::Abrupt)?;
            if let Some(index) = string_text_array_index(&text) {
                return Ok(PropertyKey::Index(index));
            }
            let atom = self.property_atom_from_text(agent, &text);
            return Ok(PropertyKey::from_atom(atom));
        }
        if let Some(atom) = self.primitive_property_key_atom(agent, value) {
            return Ok(PropertyKey::from_atom(atom));
        }
        Err(VmError::UnsupportedPropertyKey {
            code,
            instruction_offset,
            value,
        })
    }

    pub(super) fn mapped_arguments_get(
        &self,
        agent: &mut Agent,
        object: ObjectRef,
        index: u32,
    ) -> Option<VmResult<Value>> {
        let (environment, slot) = self.activation_tables.mapped_argument_slot(object, index)?;
        Some(Self::read_environment_slot(agent, environment, slot))
    }

    pub(super) fn mapped_arguments_set(
        &mut self,
        agent: &mut Agent,
        object: ObjectRef,
        index: u32,
        value: Value,
    ) -> Option<VmResult<()>> {
        let (environment, slot) = self.activation_tables.mapped_argument_slot(object, index)?;
        Some(self.write_environment_slot(agent, environment, slot, value))
    }

    pub(super) fn read_environment_slot_raw(
        agent: &Agent,
        environment: EnvironmentRef,
        slot: u32,
    ) -> VmResult<Value> {
        agent
            .environment_slot(environment, slot)
            .ok_or(VmError::MissingEnvironment(environment))
    }

    pub(super) fn set_environment_slot_raw(
        agent: &mut Agent,
        environment: EnvironmentRef,
        slot: u32,
        value: Value,
    ) -> VmResult<()> {
        if !agent.set_environment_slot(environment, slot, value) {
            return Err(VmError::MissingEnvironment(environment));
        }
        Ok(())
    }

    pub(super) fn read_environment_slot(
        agent: &mut Agent,
        environment: EnvironmentRef,
        slot: u32,
    ) -> VmResult<Value> {
        let value = Self::read_environment_slot_raw(agent, environment, slot)?;
        if value == Value::uninitialized_lexical() {
            return Err(VmError::Abrupt(errors::throw_reference_error(agent)));
        }
        Ok(value)
    }

    pub(super) fn initialize_environment_slot(
        agent: &mut Agent,
        environment: EnvironmentRef,
        slot: u32,
        value: Value,
    ) -> VmResult<()> {
        Self::set_environment_slot_raw(agent, environment, slot, value)
    }

    pub(super) fn copy_environment_slot(
        agent: &mut Agent,
        source_environment: EnvironmentRef,
        target_environment: EnvironmentRef,
        slot: u32,
    ) -> VmResult<()> {
        let value = Self::read_environment_slot_raw(agent, source_environment, slot)?;
        Self::set_environment_slot_raw(agent, target_environment, slot, value)
    }

    pub(super) fn mirror_environment_slot(
        agent: &mut Agent,
        environment: EnvironmentRef,
        slot: u32,
        value: Value,
    ) -> VmResult<()> {
        Self::set_environment_slot_raw(agent, environment, slot, value)
    }

    pub(super) fn write_environment_slot(
        &mut self,
        agent: &mut Agent,
        environment: EnvironmentRef,
        slot: u32,
        value: Value,
    ) -> VmResult<()> {
        let current = Self::read_environment_slot_raw(agent, environment, slot)?;
        if Self::environment_slot_flags(agent, environment, slot)
            .is_some_and(|flags| !flags.is_mutable() && current != Value::uninitialized_lexical())
        {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        Self::set_environment_slot_raw(agent, environment, slot, value)?;
        self.sync_loop_iteration_slot(agent, environment, slot, value)
    }

    pub(super) fn assign_environment_slot(
        &mut self,
        agent: &mut Agent,
        environment: EnvironmentRef,
        slot: u32,
        value: Value,
        strict: bool,
    ) -> VmResult<()> {
        let current = Self::read_environment_slot_raw(agent, environment, slot)?;
        if current == Value::uninitialized_lexical() {
            return Err(VmError::Abrupt(errors::throw_reference_error(agent)));
        }
        if let Some(flags) = Self::environment_slot_flags(agent, environment, slot)
            && !flags.is_mutable()
        {
            if flags.sloppy_immutable_assign_silent() && !strict {
                return Ok(());
            }
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        Self::set_environment_slot_raw(agent, environment, slot, value)?;
        self.sync_loop_iteration_slot(agent, environment, slot, value)
    }

    fn environment_slot_flags(
        agent: &Agent,
        environment: EnvironmentRef,
        slot: u32,
    ) -> Option<lyng_js_env::EnvironmentSlotFlags> {
        let layout = agent.environment(environment)?.layout()?;
        agent
            .environment_layout(layout)?
            .binding(slot)
            .map(lyng_js_env::EnvironmentBindingLayout::flags)
    }

    pub(super) fn property_key_to_enumeration_value(
        &self,
        agent: &mut Agent,
        key: PropertyKey,
    ) -> Value {
        match key {
            PropertyKey::Index(index) => {
                let text = index.to_string();
                let atom = agent.atoms_mut().intern_collectible(&text);
                Value::from_string_ref(alloc_atom_string(agent, atom, &text))
            }
            PropertyKey::Atom(atom) => {
                if let Some(text) = self
                    .atom_texts
                    .get(&atom)
                    .map(std::string::ToString::to_string)
                {
                    return Value::from_string_ref(alloc_atom_string(agent, atom, &text));
                }
                if let Some(text) = agent.atoms().get(atom).map(ToOwned::to_owned) {
                    return Value::from_string_ref(alloc_atom_string(agent, atom, &text));
                }
                if let Some(units) = agent.atoms().get_utf16(atom).map(<[u16]>::to_vec) {
                    return Value::from_string_ref(alloc_atom_utf16_string(agent, atom, &units));
                }
                let text = format!("<atom:{}>", atom.raw());
                Value::from_string_ref(alloc_atom_string(agent, atom, &text))
            }
            PropertyKey::Symbol(symbol) => Value::from_symbol_ref(symbol),
        }
    }

    pub(super) fn constant_value(
        &mut self,
        agent: &mut Agent,
        realm: RealmRef,
        constant: ConstantValue,
        compiled_atoms_by_id: &[Option<&CompiledAtom>],
        canonical_atoms: &[Option<AtomId>],
    ) -> Option<Value> {
        match constant {
            ConstantValue::Smi(value) => Some(Value::from_smi(value)),
            ConstantValue::Float64Bits(bits) => Some(Value::from_f64(f64::from_bits(bits))),
            ConstantValue::Atom(atom) => {
                let compiled_atom = compiled_atoms_by_id
                    .get(usize::try_from(atom.raw()).unwrap_or(usize::MAX))
                    .copied()
                    .flatten()?;
                let atom = canonical_atoms
                    .get(usize::try_from(atom.raw()).unwrap_or(usize::MAX))
                    .copied()
                    .flatten()
                    .unwrap_or(atom);
                Some(Value::from_string_ref(match compiled_atom {
                    CompiledAtom::Utf8(text) => alloc_atom_string(agent, atom, text),
                    CompiledAtom::Utf16(units) => alloc_atom_utf16_string(agent, atom, units),
                }))
            }
            ConstantValue::Builtin(entry) => {
                self.builtin_cache.builtin_constant(agent, realm, entry)
            }
        }
    }

    pub(super) fn read_constant(
        &self,
        agent: &Agent,
        code: CodeRef,
        index: u32,
    ) -> VmResult<Value> {
        let index_usize = usize::try_from(index).unwrap_or(usize::MAX);
        let installed = self
            .installed
            .get(code_index(code))
            .and_then(Option::as_ref)
            .ok_or(VmError::MissingInstalledCode(code))?;
        let constant = installed
            .function
            .constants()
            .get(index_usize)
            .copied()
            .ok_or(VmError::UnsupportedConstant {
                code,
                index,
                constant: ConstantValue::Smi(0),
            })?;

        if let Some(value) = agent
            .heap()
            .view()
            .code(code)
            .and_then(lyng_js_gc::RuntimeCodeRecord::constants)
            .and_then(|slots| agent.heap().view().code_slots(slots))
            .and_then(|slots| slots.get(index_usize))
            .copied()
            .filter(|value| *value != Value::empty_internal_slot())
        {
            return Ok(value);
        }

        immediate_constant_value(constant).ok_or(VmError::UnsupportedConstant {
            code,
            index,
            constant,
        })
    }

    pub(super) fn read_atom_constant(&self, code: CodeRef, index: u32) -> VmResult<AtomId> {
        let installed = self
            .installed
            .get(code_index(code))
            .and_then(Option::as_ref)
            .ok_or(VmError::MissingInstalledCode(code))?;
        let constant = installed
            .function
            .constants()
            .get(usize::try_from(index).unwrap_or(usize::MAX))
            .copied()
            .ok_or(VmError::InvalidAtomConstant {
                code,
                index,
                constant: ConstantValue::Smi(0),
            })?;
        match constant {
            ConstantValue::Atom(atom) => Ok(self.canonical_atom_for_code(code, atom)),
            _ => Err(VmError::InvalidAtomConstant {
                code,
                index,
                constant,
            }),
        }
    }

    pub(super) fn string_ref_to_property_key(
        &mut self,
        agent: &mut Agent,
        code: CodeRef,
        instruction_offset: u32,
        value: Value,
        string: lyng_js_types::StringRef,
    ) -> VmResult<PropertyKey> {
        let (array_index, cached_atom, latin1_bytes, utf16_units) =
            {
                let view = agent.heap().view().string_view(string).ok_or(
                    VmError::UnsupportedPropertyKey {
                        code,
                        instruction_offset,
                        value,
                    },
                )?;
                let array_index = string_view_array_index(&view);
                let cached_atom = view.cached_atom();
                let (latin1_bytes, utf16_units) = if array_index.is_none() && cached_atom.is_none()
                {
                    if let Some(bytes) = view.latin1_bytes() {
                        (Some(bytes.to_vec()), None)
                    } else {
                        (
                            None,
                            Some(utf16_code_units(&view).ok_or(
                                VmError::UnsupportedPropertyKey {
                                    code,
                                    instruction_offset,
                                    value,
                                },
                            )?),
                        )
                    }
                } else {
                    (None, None)
                };
                (array_index, cached_atom, latin1_bytes, utf16_units)
            };

        if let Some(index) = array_index {
            return Ok(PropertyKey::Index(index));
        }
        if let Some(atom) = cached_atom {
            return Ok(PropertyKey::from_atom(atom));
        }

        let atom = if let Some(bytes) = latin1_bytes {
            let text: String = bytes.into_iter().map(char::from).collect();
            self.property_atom_from_text(agent, &text)
        } else {
            self.property_atom_from_utf16(
                agent,
                utf16_units
                    .as_deref()
                    .ok_or(VmError::UnsupportedPropertyKey {
                        code,
                        instruction_offset,
                        value,
                    })?,
            )
        };
        if !agent.heap_mut().mutator().memoize_string_atom(string, atom) {
            return Err(VmError::UnsupportedPropertyKey {
                code,
                instruction_offset,
                value,
            });
        }
        Ok(PropertyKey::from_atom(atom))
    }
}

struct ConcatStringPayload {
    encoding: StringEncoding,
    code_unit_len: u32,
    bytes: Vec<u8>,
}

fn concat_string_refs(agent: &mut Agent, left: StringRef, right: StringRef) -> VmResult<StringRef> {
    let mut short_latin1_concat = None;
    let can_use_latin1_concat = {
        let heap_view = agent.heap().view();
        let Some(left_view) = heap_view.string_view(left) else {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        };
        let Some(right_view) = heap_view.string_view(right) else {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        };
        if left_view.code_unit_len() == 0 {
            return Ok(right);
        }
        if right_view.code_unit_len() == 0 {
            return Ok(left);
        }
        if let (Some(left_bytes), Some(right_bytes)) = (
            left_view.flat_latin1_bytes(),
            right_view.flat_latin1_bytes(),
        ) {
            let len = left_bytes.len() + right_bytes.len();
            if (2..=3).contains(&len) {
                let mut bytes = [0_u8; 3];
                bytes[..left_bytes.len()].copy_from_slice(left_bytes);
                bytes[left_bytes.len()..len].copy_from_slice(right_bytes);
                short_latin1_concat = Some((bytes, len));
            }
            true
        } else {
            left_view.encoding() == StringEncoding::Latin1
                && right_view.encoding() == StringEncoding::Latin1
        }
    };

    if let Some((bytes, len)) = short_latin1_concat {
        return Ok(agent.cached_short_latin1_string(&bytes[..len], AllocationLifetime::Default));
    }
    if can_use_latin1_concat
        && let Some(string) = agent.heap_mut().mutator().alloc_latin1_concat_string(
            left,
            right,
            AllocationLifetime::Default,
        )
    {
        return Ok(string);
    }
    if let Some(string) = agent.heap_mut().mutator().alloc_utf16_concat_string(
        left,
        right,
        AllocationLifetime::Default,
    ) {
        return Ok(string);
    }

    let payload = {
        let heap_view = agent.heap().view();
        let Some(left_view) = heap_view.string_view(left) else {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        };
        let Some(right_view) = heap_view.string_view(right) else {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        };
        concat_string_views(&left_view, &right_view)
    };

    if payload.encoding == StringEncoding::Latin1 && payload.code_unit_len == 1 {
        return Ok(agent.latin1_single_code_unit_string(payload.bytes[0]));
    }

    Ok(agent.heap_mut().mutator().alloc_string(
        payload.encoding,
        payload.code_unit_len,
        &payload.bytes,
        None,
        AllocationLifetime::Default,
    ))
}

fn concat_string_views(
    left: &PrimitiveStringView<'_>,
    right: &PrimitiveStringView<'_>,
) -> ConcatStringPayload {
    if let (Some(left_bytes), Some(right_bytes)) = (left.latin1_bytes(), right.latin1_bytes()) {
        let len = left_bytes.len() + right_bytes.len();
        let mut bytes = Vec::with_capacity(len);
        bytes.extend_from_slice(left_bytes);
        bytes.extend_from_slice(right_bytes);
        return ConcatStringPayload {
            encoding: StringEncoding::Latin1,
            code_unit_len: u32::try_from(len).expect("latin1 concat length should fit into u32"),
            bytes,
        };
    }

    let code_unit_len = left.code_unit_len() + right.code_unit_len();
    let mut bytes = Vec::with_capacity(code_unit_len as usize * 2);
    append_string_view_utf16_bytes(left, &mut bytes);
    append_string_view_utf16_bytes(right, &mut bytes);
    ConcatStringPayload {
        encoding: StringEncoding::Utf16,
        code_unit_len,
        bytes,
    }
}

fn append_string_view_utf16_bytes(view: &PrimitiveStringView<'_>, output: &mut Vec<u8>) {
    if let Some(bytes) = view.utf16_bytes() {
        output.extend_from_slice(bytes);
        return;
    }
    if let Some(bytes) = view.latin1_bytes() {
        for byte in bytes {
            output.extend_from_slice(&u16::from(*byte).to_le_bytes());
        }
    }
}

#[inline]
pub(super) fn alloc_atom_string(
    agent: &mut Agent,
    atom: AtomId,
    text: &str,
) -> lyng_js_types::StringRef {
    alloc_string(agent, text, Some(atom))
}

#[inline]
pub(super) fn alloc_atom_utf16_string(
    agent: &mut Agent,
    atom: AtomId,
    units: &[u16],
) -> lyng_js_types::StringRef {
    alloc_utf16_string(agent, units, Some(atom))
}

pub(super) fn alloc_string(
    agent: &mut Agent,
    text: &str,
    atom: Option<AtomId>,
) -> lyng_js_types::StringRef {
    if text.chars().all(|ch| u32::from(ch) <= 0xff) {
        let bytes: Vec<u8> = text.chars().map(|ch| ch as u8).collect();
        return agent.heap_mut().mutator().alloc_string(
            StringEncoding::Latin1,
            u32::try_from(bytes.len()).expect("latin1 string length should fit into u32"),
            &bytes,
            atom,
            AllocationLifetime::Default,
        );
    }

    let code_units: Vec<u16> = text.encode_utf16().collect();
    let mut bytes = Vec::with_capacity(code_units.len() * 2);
    for unit in &code_units {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    agent.heap_mut().mutator().alloc_string(
        StringEncoding::Utf16,
        u32::try_from(code_units.len()).expect("utf16 string length should fit into u32"),
        &bytes,
        atom,
        AllocationLifetime::Default,
    )
}

#[inline]
pub(super) fn alloc_code_unit_string(
    agent: &mut Agent,
    units: &[u16],
    atom: Option<AtomId>,
) -> lyng_js_types::StringRef {
    if atom.is_none() && units.len() == 1 && units[0] <= 0x00ff {
        return agent.latin1_single_code_unit_string(
            u8::try_from(units[0]).expect("Latin-1 code unit should fit into u8"),
        );
    }
    if atom.is_none()
        && let [left, right] = units
        && (*left > 0x00ff || *right > 0x00ff)
    {
        return agent.cached_two_code_unit_string([*left, *right], AllocationLifetime::Default);
    }
    if atom.is_none() && (2..=3).contains(&units.len()) && units.iter().all(|unit| *unit <= 0x00ff)
    {
        let mut bytes = [0_u8; 3];
        for (index, unit) in units.iter().copied().enumerate() {
            bytes[index] = u8::try_from(unit).expect("Latin-1 code unit should fit into u8");
        }
        return agent
            .cached_short_latin1_string(&bytes[..units.len()], AllocationLifetime::Default);
    }
    if units.len() <= 4 {
        let mut latin1 = [0_u8; 4];
        let mut is_latin1 = true;
        for (index, unit) in units.iter().copied().enumerate() {
            if unit <= 0x00ff {
                latin1[index] = u8::try_from(unit).expect("Latin-1 code unit should fit into u8");
            } else {
                is_latin1 = false;
                break;
            }
        }
        if is_latin1 {
            return agent.heap_mut().mutator().alloc_string(
                StringEncoding::Latin1,
                u32::try_from(units.len()).expect("latin1 string length should fit into u32"),
                &latin1[..units.len()],
                atom,
                AllocationLifetime::Default,
            );
        }

        let mut utf16 = [0_u8; 8];
        for (index, unit) in units.iter().copied().enumerate() {
            let offset = index * 2;
            utf16[offset..offset + 2].copy_from_slice(&unit.to_le_bytes());
        }
        return agent.heap_mut().mutator().alloc_string(
            StringEncoding::Utf16,
            u32::try_from(units.len()).expect("utf16 string length should fit into u32"),
            &utf16[..units.len() * 2],
            atom,
            AllocationLifetime::Default,
        );
    }
    if units.iter().all(|unit| *unit <= 0x00ff) {
        let bytes: Vec<u8> = units
            .iter()
            .map(|unit| u8::try_from(*unit).expect("Latin-1 code unit should fit into u8"))
            .collect();
        return agent.heap_mut().mutator().alloc_string(
            StringEncoding::Latin1,
            u32::try_from(bytes.len()).expect("latin1 string length should fit into u32"),
            &bytes,
            atom,
            AllocationLifetime::Default,
        );
    }

    alloc_utf16_string(agent, units, atom)
}

#[inline]
fn alloc_utf16_string(
    agent: &mut Agent,
    units: &[u16],
    atom: Option<AtomId>,
) -> lyng_js_types::StringRef {
    let mut bytes = Vec::with_capacity(units.len() * 2);
    for unit in units {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    agent.heap_mut().mutator().alloc_string(
        StringEncoding::Utf16,
        u32::try_from(units.len()).expect("utf16 string length should fit into u32"),
        &bytes,
        atom,
        AllocationLifetime::Default,
    )
}

#[inline]
const fn immediate_constant_value(constant: ConstantValue) -> Option<Value> {
    match constant {
        ConstantValue::Smi(value) => Some(Value::from_smi(value)),
        ConstantValue::Float64Bits(bits) => Some(Value::from_f64(f64::from_bits(bits))),
        ConstantValue::Atom(_) | ConstantValue::Builtin(_) => None,
    }
}

fn decode_string_view(view: &PrimitiveStringView<'_>) -> String {
    if let Some(bytes) = view.latin1_bytes() {
        return bytes.iter().map(|byte| char::from(*byte)).collect();
    }

    let Some(bytes) = view.utf16_bytes() else {
        return String::new();
    };
    let mut units = Vec::with_capacity(view.code_unit_len() as usize);
    for chunk in bytes.chunks_exact(2) {
        units.push(u16::from_le_bytes([chunk[0], chunk[1]]));
    }
    String::from_utf16_lossy(&units)
}

#[inline]
pub(super) const fn decode_env_operand(operand: u32) -> (u8, u32) {
    ((operand >> 24) as u8, operand & 0x00ff_ffff)
}

#[inline]
pub(super) fn to_f64_number(agent: &Agent, value: Value) -> VmResult<f64> {
    let number = read::to_number(agent.heap().view(), value).map_err(VmError::Abrupt)?;
    Ok(number
        .as_f64()
        .expect("ToNumber must always produce a numeric Value"))
}

fn numeric_conversion_error(agent: &mut Agent, abrupt: AbruptCompletion) -> VmError {
    match abrupt {
        AbruptCompletion::Throw(value) if value.is_undefined() => {
            VmError::Abrupt(errors::throw_type_error(agent))
        }
        abrupt => VmError::Abrupt(abrupt),
    }
}

#[inline]
fn to_f64_number_or_type_error(agent: &mut Agent, value: Value) -> VmResult<f64> {
    if value.is_symbol() || value.is_bigint() {
        return Err(VmError::Abrupt(errors::throw_type_error(agent)));
    }
    to_f64_number(agent, value)
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

const fn number_to_i32_after_range_check(number: f64) -> i32 {
    #[allow(
        clippy::cast_possible_truncation,
        reason = "caller validates the ECMAScript Number range before narrowing to i32"
    )]
    let integer = number as i32;
    integer
}

const fn number_to_u32_after_range_check(number: f64) -> u32 {
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "caller validates the ECMAScript Number range before narrowing to u32"
    )]
    let integer = number as u32;
    integer
}

#[inline]
pub(super) fn encode_number(number: f64) -> Value {
    if number == 0.0 && number.to_bits() != 0.0f64.to_bits() {
        return Value::from_f64(number);
    }
    if number.is_finite()
        && number.fract() == 0.0
        && number >= f64::from(i32::MIN)
        && number <= f64::from(i32::MAX)
    {
        return Value::from_smi(number_to_i32_after_range_check(number));
    }
    Value::from_f64(number)
}

#[inline]
fn number_to_array_index(number: f64) -> Option<u32> {
    if !number.is_finite() || number < 0.0 || number.fract() != 0.0 {
        return None;
    }
    if number > f64::from(PropertyKey::MAX_ARRAY_INDEX) {
        return None;
    }
    Some(number_to_u32_after_range_check(number))
}

#[inline]
fn string_view_array_index(view: &PrimitiveStringView<'_>) -> Option<u32> {
    let len = view.code_unit_len() as usize;
    if len == 0 {
        return None;
    }
    let first = ascii_digit_value(view.code_unit_at(0)?)?;
    if first == 0 {
        return (len == 1).then_some(0);
    }
    let mut value = u64::from(first);
    for index in 1..len {
        let digit = u64::from(ascii_digit_value(view.code_unit_at(index)?)?);
        value = value.checked_mul(10)?.checked_add(digit)?;
        if value > u64::from(PropertyKey::MAX_ARRAY_INDEX) {
            return None;
        }
    }
    u32::try_from(value).ok()
}

#[inline]
pub(super) fn string_text_array_index(text: &str) -> Option<u32> {
    let bytes = text.as_bytes();
    if bytes.is_empty() {
        return None;
    }
    let first = bytes[0].checked_sub(b'0').filter(|digit| *digit <= 9)?;
    if first == 0 {
        return (bytes.len() == 1).then_some(0);
    }
    let mut value = u64::from(first);
    for &byte in &bytes[1..] {
        let digit = u64::from(byte.checked_sub(b'0').filter(|digit| *digit <= 9)?);
        value = value.checked_mul(10)?.checked_add(digit)?;
        if value > u64::from(PropertyKey::MAX_ARRAY_INDEX) {
            return None;
        }
    }
    u32::try_from(value).ok()
}

#[inline]
fn utf16_code_units(view: &PrimitiveStringView<'_>) -> Option<Vec<u16>> {
    if let Some(bytes) = view.utf16_bytes() {
        return Some(
            bytes
                .chunks_exact(2)
                .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                .collect(),
        );
    }
    view.latin1_bytes()
        .map(|bytes| bytes.iter().copied().map(u16::from).collect())
}

fn append_string_ref_code_units(
    agent: &mut Agent,
    string: StringRef,
    output: &mut Vec<u16>,
) -> VmResult<()> {
    let heap_view = agent.heap().view();
    let Some(view) = heap_view.string_view(string) else {
        return Err(VmError::Abrupt(errors::throw_type_error(agent)));
    };
    if let Some(bytes) = view.latin1_bytes() {
        output.extend(bytes.iter().copied().map(u16::from));
        return Ok(());
    }
    let Some(bytes) = view.utf16_bytes() else {
        return Ok(());
    };
    output.extend(
        bytes
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]])),
    );
    Ok(())
}

fn bigint_negate_value(agent: &mut Agent, value: Value) -> VmResult<Value> {
    let (sign, limbs) = bigint_value_parts(agent, value)?;
    let sign = match sign {
        BigIntSign::NonNegative => BigIntSign::Negative,
        BigIntSign::Negative => BigIntSign::NonNegative,
    };
    Ok(alloc_bigint_value(agent, sign, &limbs))
}

fn bigint_bitwise_not_value(agent: &mut Agent, value: Value) -> VmResult<Value> {
    let (sign, limbs) = bigint_value_parts(agent, value)?;
    let sign = match sign {
        BigIntSign::NonNegative => BigIntSign::Negative,
        BigIntSign::Negative => BigIntSign::NonNegative,
    };
    let (sign, limbs) = bigint_add_signed_parts(sign, &limbs, BigIntSign::Negative, &[1]);
    Ok(alloc_bigint_value(agent, sign, &limbs))
}

fn bigint_add_values(agent: &mut Agent, left: Value, right: Value) -> VmResult<Value> {
    let (left_sign, left_limbs) = bigint_value_parts(agent, left)?;
    let (right_sign, right_limbs) = bigint_value_parts(agent, right)?;
    let (sign, limbs) = bigint_add_signed_parts(left_sign, &left_limbs, right_sign, &right_limbs);
    Ok(alloc_bigint_value(agent, sign, &limbs))
}

fn bigint_subtract_values(agent: &mut Agent, left: Value, right: Value) -> VmResult<Value> {
    let (left_sign, left_limbs) = bigint_value_parts(agent, left)?;
    let (right_sign, right_limbs) = bigint_value_parts(agent, right)?;
    let right_sign = match right_sign {
        BigIntSign::NonNegative => BigIntSign::Negative,
        BigIntSign::Negative => BigIntSign::NonNegative,
    };
    let (sign, limbs) = bigint_add_signed_parts(left_sign, &left_limbs, right_sign, &right_limbs);
    Ok(alloc_bigint_value(agent, sign, &limbs))
}

fn bigint_multiply_values(agent: &mut Agent, left: Value, right: Value) -> VmResult<Value> {
    let (left_sign, left_limbs) = bigint_value_parts(agent, left)?;
    let (right_sign, right_limbs) = bigint_value_parts(agent, right)?;
    let result_limbs = bigint_multiply_limbs(&left_limbs, &right_limbs);
    let sign = if result_limbs.is_empty() || left_sign == right_sign {
        BigIntSign::NonNegative
    } else {
        BigIntSign::Negative
    };
    Ok(alloc_bigint_value(agent, sign, &result_limbs))
}

fn bigint_divide_values(agent: &mut Agent, left: Value, right: Value) -> VmResult<Value> {
    let (left_sign, left_limbs) = bigint_value_parts(agent, left)?;
    let (right_sign, right_limbs) = bigint_value_parts(agent, right)?;
    let Some((quotient_limbs, _)) = bigint_divide_unsigned(&left_limbs, &right_limbs) else {
        return Err(VmError::Abrupt(errors::throw_range_error(agent)));
    };
    let sign = if quotient_limbs.is_empty() || left_sign == right_sign {
        BigIntSign::NonNegative
    } else {
        BigIntSign::Negative
    };
    Ok(alloc_bigint_value(agent, sign, &quotient_limbs))
}

fn bigint_remainder_values(agent: &mut Agent, left: Value, right: Value) -> VmResult<Value> {
    let (left_sign, left_limbs) = bigint_value_parts(agent, left)?;
    let (_, right_limbs) = bigint_value_parts(agent, right)?;
    let Some((_, remainder_limbs)) = bigint_divide_unsigned(&left_limbs, &right_limbs) else {
        return Err(VmError::Abrupt(errors::throw_range_error(agent)));
    };
    let sign = if remainder_limbs.is_empty() {
        BigIntSign::NonNegative
    } else {
        left_sign
    };
    Ok(alloc_bigint_value(agent, sign, &remainder_limbs))
}

fn bigint_exponentiate_values(agent: &mut Agent, left: Value, right: Value) -> VmResult<Value> {
    let (base_sign, base_limbs) = bigint_value_parts(agent, left)?;
    let (exponent_sign, mut exponent_limbs) = bigint_value_parts(agent, right)?;
    if exponent_sign == BigIntSign::Negative {
        return Err(VmError::Abrupt(errors::throw_range_error(agent)));
    }

    let exponent_is_odd = exponent_limbs.first().is_some_and(|limb| limb & 1 == 1);
    let mut result_limbs = vec![1];
    let mut power_limbs = base_limbs;
    normalize_bigint_limbs(&mut exponent_limbs);
    while !exponent_limbs.is_empty() {
        if exponent_limbs[0] & 1 == 1 {
            result_limbs = bigint_multiply_limbs(&result_limbs, &power_limbs);
        }
        bigint_shift_right_one(&mut exponent_limbs);
        if !exponent_limbs.is_empty() {
            power_limbs = bigint_multiply_limbs(&power_limbs, &power_limbs);
        }
    }

    let sign = if result_limbs.is_empty() {
        BigIntSign::NonNegative
    } else if base_sign == BigIntSign::Negative && exponent_is_odd {
        BigIntSign::Negative
    } else {
        BigIntSign::NonNegative
    };
    Ok(alloc_bigint_value(agent, sign, &result_limbs))
}

pub(super) fn bigint_bitwise_and_values(
    agent: &mut Agent,
    left: Value,
    right: Value,
) -> VmResult<Value> {
    bigint_bitwise_values(agent, left, right, |left, right| left & right)
}

pub(super) fn bigint_bitwise_or_values(
    agent: &mut Agent,
    left: Value,
    right: Value,
) -> VmResult<Value> {
    bigint_bitwise_values(agent, left, right, |left, right| left | right)
}

pub(super) fn bigint_bitwise_xor_values(
    agent: &mut Agent,
    left: Value,
    right: Value,
) -> VmResult<Value> {
    bigint_bitwise_values(agent, left, right, |left, right| left ^ right)
}

pub(super) fn bigint_shift_left_values(
    agent: &mut Agent,
    left: Value,
    right: Value,
) -> VmResult<Value> {
    let (right_sign, right_limbs) = bigint_value_parts(agent, right)?;
    let Some(shift) = bigint_shift_count(&right_limbs) else {
        return if right_sign == BigIntSign::Negative {
            bigint_shift_right_by_large_count(agent, left)
        } else {
            Err(VmError::Abrupt(errors::throw_range_error(agent)))
        };
    };
    if right_sign == BigIntSign::Negative {
        return bigint_shift_right_by_count(agent, left, shift);
    }

    let (left_sign, left_limbs) = bigint_value_parts(agent, left)?;
    let limbs = bigint_shift_left_bits(&left_limbs, shift);
    Ok(alloc_bigint_value(agent, left_sign, &limbs))
}

pub(super) fn bigint_shift_right_values(
    agent: &mut Agent,
    left: Value,
    right: Value,
) -> VmResult<Value> {
    let (right_sign, right_limbs) = bigint_value_parts(agent, right)?;
    let Some(shift) = bigint_shift_count(&right_limbs) else {
        return if right_sign == BigIntSign::Negative {
            Err(VmError::Abrupt(errors::throw_range_error(agent)))
        } else {
            bigint_shift_right_by_large_count(agent, left)
        };
    };
    if right_sign == BigIntSign::Negative {
        let (left_sign, left_limbs) = bigint_value_parts(agent, left)?;
        let limbs = bigint_shift_left_bits(&left_limbs, shift);
        return Ok(alloc_bigint_value(agent, left_sign, &limbs));
    }

    bigint_shift_right_by_count(agent, left, shift)
}

pub(super) fn compare_numeric_values(
    agent: &mut Agent,
    left: Value,
    right: Value,
) -> VmResult<Option<std::cmp::Ordering>> {
    if left.is_number() && right.is_number() {
        return Ok(left.as_f64().unwrap().partial_cmp(&right.as_f64().unwrap()));
    }
    if left.is_bigint() && right.is_bigint() {
        let (left_sign, left_limbs) = bigint_value_parts(agent, left)?;
        let (right_sign, right_limbs) = bigint_value_parts(agent, right)?;
        return Ok(Some(compare_bigint_signed_parts(
            left_sign,
            &left_limbs,
            right_sign,
            &right_limbs,
        )));
    }
    if left.is_bigint() && right.is_number() {
        return compare_bigint_to_number(agent, left, right.as_f64().unwrap());
    }
    if left.is_number() && right.is_bigint() {
        return compare_bigint_to_number(agent, right, left.as_f64().unwrap())
            .map(|ordering| ordering.map(std::cmp::Ordering::reverse));
    }

    Err(VmError::Abrupt(errors::throw_type_error(agent)))
}

fn bigint_value_parts(agent: &mut Agent, value: Value) -> VmResult<(BigIntSign, Vec<u64>)> {
    let bigint = value
        .as_bigint_ref()
        .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
    let view = {
        let heap_view = agent.heap().view();
        match heap_view.bigint_view(bigint) {
            Some(view) => view,
            None => return Err(VmError::Abrupt(errors::throw_type_error(agent))),
        }
    };
    Ok((view.sign(), view.to_limbs()))
}

fn alloc_bigint_value(agent: &mut Agent, sign: BigIntSign, limbs: &[u64]) -> Value {
    let bigint = agent
        .heap_mut()
        .mutator()
        .alloc_bigint(sign, limbs, AllocationLifetime::Default);
    Value::from_bigint_ref(bigint)
}

fn bigint_bitwise_values(
    agent: &mut Agent,
    left: Value,
    right: Value,
    op: impl Fn(u64, u64) -> u64,
) -> VmResult<Value> {
    let (left_sign, left_limbs) = bigint_value_parts(agent, left)?;
    let (right_sign, right_limbs) = bigint_value_parts(agent, right)?;
    let width =
        normalized_bigint_limb_len(&left_limbs).max(normalized_bigint_limb_len(&right_limbs)) + 1;
    let left_bits = bigint_to_twos_complement(left_sign, &left_limbs, width);
    let right_bits = bigint_to_twos_complement(right_sign, &right_limbs, width);
    let mut result = Vec::with_capacity(width);
    for (left, right) in left_bits.iter().zip(right_bits.iter()) {
        result.push(op(*left, *right));
    }
    Ok(twos_complement_to_bigint(agent, &result))
}

fn bigint_to_twos_complement(sign: BigIntSign, limbs: &[u64], width: usize) -> Vec<u64> {
    let mut bits = vec![0; width.max(1)];
    let len = normalized_bigint_limb_len(limbs).min(bits.len());
    bits[..len].copy_from_slice(&limbs[..len]);
    if normalize_bigint_sign(sign, limbs) == BigIntSign::Negative {
        for limb in &mut bits {
            *limb = !*limb;
        }
        add_one_to_limbs(&mut bits);
    }
    bits
}

fn twos_complement_to_bigint(agent: &mut Agent, bits: &[u64]) -> Value {
    let negative = bits.last().is_some_and(|limb| (limb & (1_u64 << 63)) != 0);
    let mut limbs = bits.to_vec();
    let sign = if negative {
        for limb in &mut limbs {
            *limb = !*limb;
        }
        add_one_to_limbs(&mut limbs);
        BigIntSign::Negative
    } else {
        BigIntSign::NonNegative
    };
    normalize_bigint_limbs(&mut limbs);
    alloc_bigint_value(agent, normalize_bigint_sign(sign, &limbs), &limbs)
}

fn bigint_shift_count(limbs: &[u64]) -> Option<usize> {
    let len = normalized_bigint_limb_len(limbs);
    if len == 0 {
        return Some(0);
    }
    if len > usize::BITS as usize / 64 {
        return None;
    }
    let mut shift = 0_usize;
    for (index, limb) in limbs.iter().take(len).copied().enumerate() {
        let bit_offset = index.checked_mul(64)?;
        if bit_offset >= usize::BITS as usize {
            return (limb == 0).then_some(shift);
        }
        let limb_value = usize::try_from(limb).ok()?;
        shift |= limb_value.checked_shl(u32::try_from(bit_offset).ok()?)?;
    }
    Some(shift)
}

fn bigint_shift_right_by_count(agent: &mut Agent, value: Value, shift: usize) -> VmResult<Value> {
    let (sign, limbs) = bigint_value_parts(agent, value)?;
    if shift == 0 {
        return Ok(alloc_bigint_value(agent, sign, &limbs));
    }

    let (mut result, truncated_non_zero) = bigint_shift_right_abs_bits(&limbs, shift);
    let sign = if sign == BigIntSign::Negative && (!result.is_empty() || truncated_non_zero) {
        if truncated_non_zero {
            result = bigint_add_limbs(&result, &[1]);
        }
        BigIntSign::Negative
    } else {
        BigIntSign::NonNegative
    };
    Ok(alloc_bigint_value(agent, sign, &result))
}

fn bigint_shift_right_by_large_count(agent: &mut Agent, value: Value) -> VmResult<Value> {
    let (sign, limbs) = bigint_value_parts(agent, value)?;
    if normalized_bigint_limb_len(&limbs) == 0 {
        return Ok(alloc_bigint_value(agent, BigIntSign::NonNegative, &[]));
    }
    if sign == BigIntSign::Negative {
        return Ok(alloc_bigint_value(agent, BigIntSign::Negative, &[1]));
    }
    Ok(alloc_bigint_value(agent, BigIntSign::NonNegative, &[]))
}

fn bigint_shift_right_abs_bits(limbs: &[u64], shift: usize) -> (Vec<u64>, bool) {
    let len = normalized_bigint_limb_len(limbs);
    if len == 0 {
        return (Vec::new(), false);
    }

    let limb_shift = shift / 64;
    let bit_shift = shift % 64;
    let truncated_by_limb = limbs
        .iter()
        .take(limb_shift.min(len))
        .any(|limb| *limb != 0);
    if limb_shift >= len {
        return (Vec::new(), truncated_by_limb);
    }
    let truncated_by_bits = bit_shift != 0 && (limbs[limb_shift] & ((1_u64 << bit_shift) - 1)) != 0;
    let mut result = Vec::with_capacity(len - limb_shift);
    for index in limb_shift..len {
        let mut limb = limbs[index] >> bit_shift;
        if bit_shift != 0 && index + 1 < len {
            limb |= limbs[index + 1] << (64 - bit_shift);
        }
        result.push(limb);
    }
    normalize_bigint_limbs(&mut result);
    (result, truncated_by_limb || truncated_by_bits)
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

fn compare_bigint_signed_parts(
    left_sign: BigIntSign,
    left_limbs: &[u64],
    right_sign: BigIntSign,
    right_limbs: &[u64],
) -> std::cmp::Ordering {
    let left_sign = normalize_bigint_sign(left_sign, left_limbs);
    let right_sign = normalize_bigint_sign(right_sign, right_limbs);
    match (left_sign, right_sign) {
        (BigIntSign::Negative, BigIntSign::NonNegative) => std::cmp::Ordering::Less,
        (BigIntSign::NonNegative, BigIntSign::Negative) => std::cmp::Ordering::Greater,
        (BigIntSign::NonNegative, BigIntSign::NonNegative) => {
            bigint_compare_limbs(left_limbs, right_limbs)
        }
        (BigIntSign::Negative, BigIntSign::Negative) => {
            bigint_compare_limbs(right_limbs, left_limbs)
        }
    }
}

fn compare_bigint_to_number(
    agent: &mut Agent,
    bigint: Value,
    number: f64,
) -> VmResult<Option<std::cmp::Ordering>> {
    if number.is_nan() {
        return Ok(None);
    }
    if number == f64::INFINITY {
        return Ok(Some(std::cmp::Ordering::Less));
    }
    if number == f64::NEG_INFINITY {
        return Ok(Some(std::cmp::Ordering::Greater));
    }

    let (bigint_sign, bigint_limbs) = bigint_value_parts(agent, bigint)?;
    let truncated = number.trunc();
    let (number_sign, number_limbs) = number_to_bigint_parts(truncated)
        .expect("finite truncated number must convert to bigint parts");
    let ordering =
        compare_bigint_signed_parts(bigint_sign, &bigint_limbs, number_sign, &number_limbs);
    if number.fract() == 0.0 {
        return Ok(Some(ordering));
    }
    if number.is_sign_positive() {
        if ordering.is_gt() {
            Ok(Some(std::cmp::Ordering::Greater))
        } else {
            Ok(Some(std::cmp::Ordering::Less))
        }
    } else if ordering.is_lt() {
        Ok(Some(std::cmp::Ordering::Less))
    } else {
        Ok(Some(std::cmp::Ordering::Greater))
    }
}

#[allow(clippy::float_cmp)]
fn number_to_bigint_parts(number: f64) -> Option<(BigIntSign, Vec<u64>)> {
    if !number.is_finite() || number != number.trunc() {
        return None;
    }
    if number == 0.0 {
        return Some((BigIntSign::NonNegative, Vec::new()));
    }

    let bits = number.to_bits();
    let sign = if bits >> 63 == 0 {
        BigIntSign::NonNegative
    } else {
        BigIntSign::Negative
    };
    let exponent_bits = ((bits >> 52) & 0x7ff) as i32;
    if exponent_bits == 0 {
        return None;
    }

    let exponent = exponent_bits - 1023;
    let significand = (1_u64 << 52) | (bits & ((1_u64 << 52) - 1));
    let shift = exponent - 52;
    let mut limbs = if shift >= 0 {
        number_shift_left_word(significand, shift.cast_unsigned())
    } else {
        let right_shift = (-shift).cast_unsigned();
        if right_shift >= 64 {
            return None;
        }
        let mask = (1_u64 << right_shift) - 1;
        if significand & mask != 0 {
            return None;
        }
        let truncated = significand >> right_shift;
        if truncated == 0 {
            Vec::new()
        } else {
            vec![truncated]
        }
    };
    normalize_bigint_limbs(&mut limbs);
    Some((sign, limbs))
}

fn number_shift_left_word(word: u64, shift: u32) -> Vec<u64> {
    let whole_words = usize::try_from(shift / 64).expect("whole-word shift must fit into usize");
    let bit_shift = shift % 64;
    let mut limbs = vec![0; whole_words];

    if bit_shift == 0 {
        limbs.push(word);
        return limbs;
    }

    limbs.push(word << bit_shift);
    let carry = word >> (64 - bit_shift);
    if carry != 0 {
        limbs.push(carry);
    }
    limbs
}

fn normalize_bigint_sign(sign: BigIntSign, limbs: &[u64]) -> BigIntSign {
    if normalized_bigint_limb_len(limbs) == 0 {
        BigIntSign::NonNegative
    } else {
        sign
    }
}

fn bigint_add_signed_parts(
    left_sign: BigIntSign,
    left_limbs: &[u64],
    right_sign: BigIntSign,
    right_limbs: &[u64],
) -> (BigIntSign, Vec<u64>) {
    if left_sign == right_sign {
        return (left_sign, bigint_add_limbs(left_limbs, right_limbs));
    }
    match bigint_compare_limbs(left_limbs, right_limbs) {
        std::cmp::Ordering::Greater => (left_sign, bigint_subtract_limbs(left_limbs, right_limbs)),
        std::cmp::Ordering::Less => (right_sign, bigint_subtract_limbs(right_limbs, left_limbs)),
        std::cmp::Ordering::Equal => (BigIntSign::NonNegative, Vec::new()),
    }
}

fn bigint_compare_limbs(left: &[u64], right: &[u64]) -> std::cmp::Ordering {
    let left_len = normalized_bigint_limb_len(left);
    let right_len = normalized_bigint_limb_len(right);
    match left_len.cmp(&right_len) {
        std::cmp::Ordering::Equal => {
            for index in (0..left_len).rev() {
                match left[index].cmp(&right[index]) {
                    std::cmp::Ordering::Equal => {}
                    ordering => return ordering,
                }
            }
            std::cmp::Ordering::Equal
        }
        ordering => ordering,
    }
}

fn bigint_add_limbs(left: &[u64], right: &[u64]) -> Vec<u64> {
    let mut result = Vec::with_capacity(left.len().max(right.len()) + 1);
    let mut carry = 0_u128;
    let max_len = left.len().max(right.len());
    for index in 0..max_len {
        let left_limb = u128::from(*left.get(index).unwrap_or(&0));
        let right_limb = u128::from(*right.get(index).unwrap_or(&0));
        let total = left_limb + right_limb + carry;
        result.push(low_u64(total));
        carry = total >> 64;
    }
    if carry != 0 {
        result.push(u64::try_from(carry).expect("BigInt addition carry should fit into u64"));
    }
    normalize_bigint_limbs(&mut result);
    result
}

fn bigint_subtract_limbs(left: &[u64], right: &[u64]) -> Vec<u64> {
    let mut result = Vec::with_capacity(left.len());
    let mut borrow = 0_u128;
    for (index, &left_limb) in left.iter().enumerate() {
        let minuend = u128::from(left_limb);
        let subtrahend = u128::from(*right.get(index).unwrap_or(&0)) + borrow;
        if minuend >= subtrahend {
            result.push(
                u64::try_from(minuend - subtrahend)
                    .expect("BigInt subtraction limb should fit into u64"),
            );
            borrow = 0;
        } else {
            result.push(
                u64::try_from((1_u128 << 64) + minuend - subtrahend)
                    .expect("BigInt borrowed subtraction limb should fit into u64"),
            );
            borrow = 1;
        }
    }
    normalize_bigint_limbs(&mut result);
    result
}

fn bigint_multiply_limbs(left: &[u64], right: &[u64]) -> Vec<u64> {
    if left.is_empty() || right.is_empty() {
        return Vec::new();
    }

    let mut result = vec![0_u64; left.len() + right.len()];
    for (left_index, &left_limb) in left.iter().enumerate() {
        let mut carry = 0_u128;
        for (right_index, &right_limb) in right.iter().enumerate() {
            let slot_index = left_index + right_index;
            let total = u128::from(left_limb) * u128::from(right_limb)
                + u128::from(result[slot_index])
                + carry;
            result[slot_index] = low_u64(total);
            carry = total >> 64;
        }
        let mut slot_index = left_index + right.len();
        while carry != 0 {
            let total = u128::from(result[slot_index]) + carry;
            result[slot_index] = low_u64(total);
            carry = total >> 64;
            slot_index += 1;
        }
    }
    normalize_bigint_limbs(&mut result);
    result
}

fn low_u64(value: u128) -> u64 {
    u64::try_from(value & u128::from(u64::MAX)).expect("masked u128 limb should fit into u64")
}

fn bigint_divide_unsigned(dividend: &[u64], divisor: &[u64]) -> Option<(Vec<u64>, Vec<u64>)> {
    let mut dividend = dividend.to_vec();
    normalize_bigint_limbs(&mut dividend);

    let mut divisor = divisor.to_vec();
    normalize_bigint_limbs(&mut divisor);
    if divisor.is_empty() {
        return None;
    }
    if dividend.is_empty() || bigint_compare_limbs(&dividend, &divisor).is_lt() {
        return Some((Vec::new(), dividend));
    }

    let shift = bigint_bit_length(&dividend).checked_sub(bigint_bit_length(&divisor))?;
    let mut shifted_divisor = bigint_shift_left_bits(&divisor, shift);
    let mut quotient = vec![0_u64; shift / 64 + 1];

    for current_shift in (0..=shift).rev() {
        if !bigint_compare_limbs(&dividend, &shifted_divisor).is_lt() {
            dividend = bigint_subtract_limbs(&dividend, &shifted_divisor);
            quotient[current_shift / 64] |= 1_u64 << (current_shift % 64);
        }
        bigint_shift_right_one(&mut shifted_divisor);
    }

    normalize_bigint_limbs(&mut quotient);
    normalize_bigint_limbs(&mut dividend);
    Some((quotient, dividend))
}

fn bigint_bit_length(limbs: &[u64]) -> usize {
    let len = normalized_bigint_limb_len(limbs);
    if len == 0 {
        return 0;
    }
    let highest = limbs[len - 1];
    (len - 1) * 64 + (64 - highest.leading_zeros() as usize)
}

fn bigint_shift_left_bits(limbs: &[u64], shift: usize) -> Vec<u64> {
    let len = normalized_bigint_limb_len(limbs);
    if len == 0 {
        return Vec::new();
    }

    let limb_shift = shift / 64;
    let bit_shift = shift % 64;
    let mut result = vec![0_u64; limb_shift + len + usize::from(bit_shift != 0)];
    let mut carry = 0_u64;

    for (index, &limb) in limbs.iter().take(len).enumerate() {
        let target = index + limb_shift;
        if bit_shift == 0 {
            result[target] = limb;
            continue;
        }
        result[target] = (limb << bit_shift) | carry;
        carry = limb >> (64 - bit_shift);
    }

    if bit_shift != 0 && carry != 0 {
        result[limb_shift + len] = carry;
    }
    normalize_bigint_limbs(&mut result);
    result
}

fn bigint_shift_right_one(limbs: &mut Vec<u64>) {
    let mut carry = 0_u64;
    for limb in limbs.iter_mut().rev() {
        let next_carry = *limb << 63;
        *limb = (*limb >> 1) | carry;
        carry = next_carry;
    }
    normalize_bigint_limbs(limbs);
}

fn normalize_bigint_limbs(limbs: &mut Vec<u64>) {
    while limbs.last().is_some_and(|limb| *limb == 0) {
        limbs.pop();
    }
}

fn normalized_bigint_limb_len(limbs: &[u64]) -> usize {
    limbs
        .iter()
        .rposition(|limb| *limb != 0)
        .map_or(0, |index| index + 1)
}

#[inline]
fn ascii_digit_value(code_unit: u16) -> Option<u8> {
    if (u16::from(b'0')..=u16::from(b'9')).contains(&code_unit) {
        u8::try_from(code_unit - u16::from(b'0')).ok()
    } else {
        None
    }
}

#[inline]
pub(super) fn bytecode_index(id: BytecodeFunctionId) -> usize {
    usize::try_from(id.get()).expect("bytecode function id should fit in usize")
}

#[inline]
pub(super) fn code_index(id: CodeRef) -> usize {
    usize::try_from(id.get() - 1).expect("code ref should fit in usize")
}
