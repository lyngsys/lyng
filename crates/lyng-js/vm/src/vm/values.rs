use super::*;
use crate::vm::property_access::ToPrimitiveHint;
use lyng_js_bytecode::{WideAbcOperands, WideAbxOperands};
use lyng_js_gc::{AllocationLifetime, BigIntSign, PrimitiveStringView, StringEncoding};
use lyng_js_ops::{errors, number_to_string, object, read};
use lyng_js_types::PropertyKey;

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
        frame: FrameRecord,
        left_register: u16,
        right_register: u16,
    ) -> VmResult<Value> {
        let left = self.to_primitive(
            agent,
            host,
            registry,
            frame,
            self.read_register(frame, left_register)?,
            ToPrimitiveHint::Number,
        )?;
        let right = self.to_primitive(
            agent,
            host,
            registry,
            frame,
            self.read_register(frame, right_register)?,
            ToPrimitiveHint::Number,
        )?;
        if left.is_string() || right.is_string() {
            let left_units = self.value_to_string_code_units(agent, left)?;
            let right_units = self.value_to_string_code_units(agent, right)?;
            let mut units = Vec::with_capacity(left_units.len() + right_units.len());
            units.extend(left_units);
            units.extend(right_units);
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

        let left = to_f64_number(agent, left)?;
        let right = to_f64_number(agent, right)?;
        Ok(encode_number(left + right))
    }

    pub(super) fn sub_values(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        left_register: u16,
        right_register: u16,
    ) -> VmResult<Value> {
        let left = self.to_primitive(
            agent,
            host,
            registry,
            frame,
            self.read_register(frame, left_register)?,
            ToPrimitiveHint::Number,
        )?;
        let right = self.to_primitive(
            agent,
            host,
            registry,
            frame,
            self.read_register(frame, right_register)?,
            ToPrimitiveHint::Number,
        )?;
        if left.is_bigint() || right.is_bigint() {
            if !left.is_bigint() || !right.is_bigint() {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
            return bigint_subtract_values(agent, left, right);
        }
        let left = to_f64_number(agent, left)?;
        let right = to_f64_number(agent, right)?;
        Ok(encode_number(left - right))
    }

    pub(super) fn negate_value(
        &self,
        agent: &mut Agent,
        frame: FrameRecord,
        register: u16,
    ) -> VmResult<Value> {
        let value = self.read_register(frame, register)?;
        if value.is_bigint() {
            return bigint_negate_value(agent, value);
        }
        let number = to_f64_number(agent, value)?;
        Ok(encode_number(-number))
    }

    pub(super) fn mul_values(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        left_register: u16,
        right_register: u16,
    ) -> VmResult<Value> {
        let left = self.to_primitive(
            agent,
            host,
            registry,
            frame,
            self.read_register(frame, left_register)?,
            ToPrimitiveHint::Number,
        )?;
        let right = self.to_primitive(
            agent,
            host,
            registry,
            frame,
            self.read_register(frame, right_register)?,
            ToPrimitiveHint::Number,
        )?;
        if left.is_bigint() || right.is_bigint() {
            if !left.is_bigint() || !right.is_bigint() {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
            return bigint_multiply_values(agent, left, right);
        }
        let left = to_f64_number(agent, left)?;
        let right = to_f64_number(agent, right)?;
        Ok(encode_number(left * right))
    }

    pub(super) fn div_values(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        left_register: u16,
        right_register: u16,
    ) -> VmResult<Value> {
        let left = self.to_primitive(
            agent,
            host,
            registry,
            frame,
            self.read_register(frame, left_register)?,
            ToPrimitiveHint::Number,
        )?;
        let right = self.to_primitive(
            agent,
            host,
            registry,
            frame,
            self.read_register(frame, right_register)?,
            ToPrimitiveHint::Number,
        )?;
        if left.is_bigint() || right.is_bigint() {
            if !left.is_bigint() || !right.is_bigint() {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
            return bigint_divide_values(agent, left, right);
        }
        let left = to_f64_number(agent, left)?;
        let right = to_f64_number(agent, right)?;
        Ok(encode_number(left / right))
    }

    pub(super) fn rem_values(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        left_register: u16,
        right_register: u16,
    ) -> VmResult<Value> {
        let left = self.to_primitive(
            agent,
            host,
            registry,
            frame,
            self.read_register(frame, left_register)?,
            ToPrimitiveHint::Number,
        )?;
        let right = self.to_primitive(
            agent,
            host,
            registry,
            frame,
            self.read_register(frame, right_register)?,
            ToPrimitiveHint::Number,
        )?;
        if left.is_bigint() || right.is_bigint() {
            if !left.is_bigint() || !right.is_bigint() {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
            return bigint_remainder_values(agent, left, right);
        }
        let left = to_f64_number(agent, left)?;
        let right = to_f64_number(agent, right)?;
        Ok(encode_number(left % right))
    }

    pub(super) fn exp_values(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        frame: FrameRecord,
        left_register: u16,
        right_register: u16,
    ) -> VmResult<Value> {
        let left = self.to_primitive(
            agent,
            host,
            registry,
            frame,
            self.read_register(frame, left_register)?,
            ToPrimitiveHint::Number,
        )?;
        let right = self.to_primitive(
            agent,
            host,
            registry,
            frame,
            self.read_register(frame, right_register)?,
            ToPrimitiveHint::Number,
        )?;
        if left.is_bigint() || right.is_bigint() {
            if !left.is_bigint() || !right.is_bigint() {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
            return bigint_exponentiate_values(agent, left, right);
        }
        let left = to_f64_number(agent, left)?;
        let right = to_f64_number(agent, right)?;
        Ok(encode_number(left.powf(right)))
    }

    pub(crate) fn value_to_string_text(&self, agent: &mut Agent, value: Value) -> VmResult<String> {
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
            return Ok(decode_string_view(view));
        }

        Err(VmError::Abrupt(errors::throw_type_error(agent)))
    }

    pub(super) fn value_to_string_code_units(
        &self,
        agent: &mut Agent,
        value: Value,
    ) -> VmResult<Vec<u16>> {
        if let Some(string) = value.as_string_ref() {
            let heap_view = agent.heap().view();
            let Some(view) = heap_view.string_view(string) else {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            };
            return utf16_code_units(view)
                .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)));
        }

        Ok(self
            .value_to_string_text(agent, value)?
            .encode_utf16()
            .collect())
    }

    pub(super) fn decode_abc_operands(
        &self,
        installed: &InstalledFunction,
        frame: FrameRecord,
        opcode: Opcode,
        a: u8,
        b: u8,
        c: u8,
    ) -> (u16, u16, u16) {
        if matches!(opcode, Opcode::Call | Opcode::TailCall | Opcode::Construct) {
            return (u16::from(a), u16::from(b), u16::from(c));
        }
        let operands = installed
            .wide_payload(frame.instruction_offset())
            .map_or_else(
                || WideAbcOperands::narrow(a, b, c),
                |payload| WideAbcOperands::decode(a, b, c, payload),
            );
        (operands.a(), operands.b(), operands.c())
    }

    pub(super) fn decode_abx_operands(
        &self,
        installed: &InstalledFunction,
        frame: FrameRecord,
        a: u8,
        bx: u16,
    ) -> (u16, u32) {
        let operands = installed
            .wide_payload(frame.instruction_offset())
            .map_or_else(
                || WideAbxOperands::narrow(a, bx),
                |payload| WideAbxOperands::decode(a, bx, payload),
            );
        (operands.a(), operands.bx())
    }

    pub(super) fn object_register(&self, frame: FrameRecord, register: u16) -> VmResult<ObjectRef> {
        let value = self.read_register(frame, register)?;
        self.require_object(frame, value)
    }

    pub(super) fn require_object(&self, frame: FrameRecord, value: Value) -> VmResult<ObjectRef> {
        value.as_object_ref().ok_or(VmError::ExpectedObject {
            code: frame.code(),
            instruction_offset: frame.instruction_offset(),
            value,
        })
    }

    pub(super) fn value_to_property_key(
        &mut self,
        agent: &mut Agent,
        _frame: FrameRecord,
        code: CodeRef,
        instruction_offset: u32,
        value: Value,
    ) -> VmResult<PropertyKey> {
        if let Some(index) = value.as_smi().and_then(|index| u32::try_from(index).ok()) {
            return Ok(PropertyKey::Index(index));
        }
        if let Some(number) = value.as_f64() {
            if let Some(index) = number_to_array_index(number) {
                return Ok(PropertyKey::Index(index));
            }
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
        Some(self.read_environment_slot(agent, environment, slot))
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
        &self,
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
        &mut self,
        agent: &mut Agent,
        environment: EnvironmentRef,
        slot: u32,
        value: Value,
    ) -> VmResult<()> {
        Self::set_environment_slot_raw(agent, environment, slot, value)
    }

    pub(super) fn copy_environment_slot(
        &mut self,
        agent: &mut Agent,
        source_environment: EnvironmentRef,
        target_environment: EnvironmentRef,
        slot: u32,
    ) -> VmResult<()> {
        let value = Self::read_environment_slot_raw(agent, source_environment, slot)?;
        Self::set_environment_slot_raw(agent, target_environment, slot, value)
    }

    pub(super) fn mirror_environment_slot(
        &mut self,
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
        if self
            .environment_slot_flags(agent, environment, slot)
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
        if let Some(flags) = self.environment_slot_flags(agent, environment, slot) {
            if !flags.is_mutable() {
                if flags.sloppy_immutable_assign_silent() && !strict {
                    return Ok(());
                }
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
        }
        Self::set_environment_slot_raw(agent, environment, slot, value)?;
        self.sync_loop_iteration_slot(agent, environment, slot, value)
    }

    fn environment_slot_flags(
        &self,
        agent: &Agent,
        environment: EnvironmentRef,
        slot: u32,
    ) -> Option<lyng_js_env::EnvironmentSlotFlags> {
        let layout = agent.environment(environment)?.layout()?;
        agent
            .environment_layout(layout)?
            .binding(slot)
            .map(|binding| binding.flags())
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
                if let Some(text) = self.atom_texts.get(&atom).map(|text| text.to_string()) {
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
        atom_texts: &[(AtomId, CompiledAtom)],
        canonical_atoms: &[Option<AtomId>],
    ) -> Option<Value> {
        match constant {
            ConstantValue::Smi(value) => Some(Value::from_smi(value)),
            ConstantValue::Float64Bits(bits) => Some(Value::from_f64(f64::from_bits(bits))),
            ConstantValue::Atom(atom) => {
                let compiled_atom = lookup_compiled_atom(atom_texts, atom)?;
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
            .and_then(|record| record.constants())
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
                let array_index = string_view_array_index(view);
                let cached_atom = view.cached_atom();
                let (latin1_bytes, utf16_units) = if array_index.is_none() && cached_atom.is_none()
                {
                    if let Some(bytes) = view.latin1_bytes() {
                        (Some(bytes.to_vec()), None)
                    } else {
                        (
                            None,
                            Some(utf16_code_units(view).ok_or(
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
fn alloc_code_unit_string(
    agent: &mut Agent,
    units: &[u16],
    atom: Option<AtomId>,
) -> lyng_js_types::StringRef {
    if units.iter().all(|unit| *unit <= 0x00ff) {
        let bytes: Vec<u8> = units.iter().map(|unit| *unit as u8).collect();
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
fn lookup_compiled_atom<'a>(
    atom_texts: &'a [(AtomId, CompiledAtom)],
    atom: AtomId,
) -> Option<&'a CompiledAtom> {
    atom_texts
        .iter()
        .find_map(|(candidate, text)| (*candidate == atom).then_some(text))
}

#[inline]
fn immediate_constant_value(constant: ConstantValue) -> Option<Value> {
    match constant {
        ConstantValue::Smi(value) => Some(Value::from_smi(value)),
        ConstantValue::Float64Bits(bits) => Some(Value::from_f64(f64::from_bits(bits))),
        ConstantValue::Atom(_) | ConstantValue::Builtin(_) => None,
    }
}

fn decode_string_view(view: PrimitiveStringView<'_>) -> String {
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
        return Value::from_smi(number as i32);
    }
    Value::from_f64(number)
}

#[inline]
fn number_to_array_index(number: f64) -> Option<u32> {
    if !number.is_finite() || number < 0.0 || number.fract() != 0.0 {
        return None;
    }
    let index = number as u64;
    PropertyKey::from_array_index(index).and_then(PropertyKey::as_index)
}

#[inline]
fn string_view_array_index(view: PrimitiveStringView<'_>) -> Option<u32> {
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
fn string_text_array_index(text: &str) -> Option<u32> {
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
fn utf16_code_units(view: PrimitiveStringView<'_>) -> Option<Vec<u16>> {
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

fn bigint_negate_value(agent: &mut Agent, value: Value) -> VmResult<Value> {
    let (sign, limbs) = bigint_value_parts(agent, value)?;
    let sign = match sign {
        BigIntSign::NonNegative => BigIntSign::Negative,
        BigIntSign::Negative => BigIntSign::NonNegative,
    };
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
                    std::cmp::Ordering::Equal => continue,
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
        result.push(total as u64);
        carry = total >> 64;
    }
    if carry != 0 {
        result.push(carry as u64);
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
            result.push((minuend - subtrahend) as u64);
            borrow = 0;
        } else {
            result.push(((1_u128 << 64) + minuend - subtrahend) as u64);
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
            result[slot_index] = total as u64;
            carry = total >> 64;
        }
        let mut slot_index = left_index + right.len();
        while carry != 0 {
            let total = u128::from(result[slot_index]) + carry;
            result[slot_index] = total as u64;
            carry = total >> 64;
            slot_index += 1;
        }
    }
    normalize_bigint_limbs(&mut result);
    result
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
