use super::{
    alloc_code_unit_string, alloc_string, errors, object, parse_script, read,
    string_from_code_point_builtin, Agent, AllocationLifetime, AtomTable, FrameRecord,
    FunctionConstructorFlags, FunctionObjectData, FunctionThisMode, HostHooks,
    NativeFunctionRegistry, ObjectAllocation, ObjectColdData, ObjectRef, PropertyKey, RealmRef,
    ToPrimitiveHint, Value, Vm, VmError, VmProxyBridge, VmResult, WellKnownAtom,
};

const MAX_FAST_APPLY_STRING_CODE_UNITS: usize = 1 << 20;

impl Vm {
    #[expect(
        clippy::too_many_arguments,
        reason = "VM helper threads interpreter, host, registry, and spec state explicitly at call sites"
    )]
    #[expect(
        clippy::too_many_lines,
        reason = "spec-shaped VM algorithm stays contiguous until the VM module split issue extracts it"
    )]
    pub(super) fn create_bound_function(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller: FrameRecord,
        target: ObjectRef,
        bound_this: Value,
        bound_arguments: &[Value],
    ) -> VmResult<ObjectRef> {
        if !agent.objects().is_callable(target) {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        let target_prototype = object::get_prototype_of_in_context(
            &mut VmProxyBridge {
                vm: self,
                agent,
                host,
                registry,
                frame: caller,
            },
            target,
        )?;
        let target_data = bound_target_function_data(agent, target)?;
        let realm = target_data
            .as_ref()
            .and_then(FunctionObjectData::realm)
            .unwrap_or_else(|| caller.realm());
        let realm_record = agent
            .realm(realm)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let root_shape = realm_record
            .root_shape()
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let environment = target_data
            .as_ref()
            .and_then(FunctionObjectData::environment)
            .unwrap_or_else(|| realm_record.global_env());
        let mut function_data = FunctionObjectData::bound(
            realm,
            environment,
            target,
            bound_this,
            bound_arguments.to_vec().into_boxed_slice(),
        )
        .with_has_prototype_property(false)
        .with_constructor_flags(if agent.objects().is_constructor(target) {
            FunctionConstructorFlags::constructible()
        } else {
            FunctionConstructorFlags::empty()
        })
        .with_this_mode(FunctionThisMode::Strict);
        if let Some(target_data) = target_data.as_ref() {
            function_data = function_data.with_kind_flags(target_data.kind_flags());
        }
        let function = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.alloc_object(
                &mut mutator,
                ObjectAllocation::function(root_shape)
                    .with_prototype(target_prototype)
                    .with_cold_data(ObjectColdData::Function(function_data)),
                AllocationLifetime::Default,
            )
        });

        let length_key = PropertyKey::from_atom(WellKnownAtom::length.id());
        let target_has_own_length = object::get_own_property_in_context(
            &mut VmProxyBridge {
                vm: self,
                agent,
                host,
                registry,
                frame: caller,
            },
            target,
            length_key,
        )?
        .is_some();
        let bound_length = if target_has_own_length {
            let target_length = self.get_property_from_object(
                agent,
                host,
                registry,
                caller,
                target,
                Value::from_object_ref(target),
                length_key,
            )?;
            bound_function_length_value(target_length, bound_arguments.len())
        } else {
            Value::from_smi(0)
        };
        let target_name = self.get_property_from_object(
            agent,
            host,
            registry,
            caller,
            target,
            Value::from_object_ref(target),
            PropertyKey::from_atom(WellKnownAtom::name.id()),
        )?;
        let target_name = if target_name.as_string_ref().is_some() {
            Self::value_to_string_text(agent, target_name)?
        } else {
            String::new()
        };
        Self::define_data_property_with_attrs(
            agent,
            function,
            PropertyKey::from_atom(WellKnownAtom::length.id()),
            bound_length,
            false,
            false,
            true,
        )?;
        let bound_name =
            Value::from_string_ref(alloc_string(agent, &format!("bound {target_name}"), None));
        Self::define_data_property_with_attrs(
            agent,
            function,
            PropertyKey::from_atom(WellKnownAtom::name.id()),
            bound_name,
            false,
            false,
            true,
        )?;
        Ok(function)
    }

    pub(super) fn collect_array_like_arguments(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller_frame: FrameRecord,
        realm: RealmRef,
        value: Value,
    ) -> VmResult<Vec<Value>> {
        if value.is_null() || value.is_undefined() {
            return Ok(Vec::new());
        }
        let object = value
            .as_object_ref()
            .ok_or_else(|| Self::abrupt_intrinsic_error(agent, realm, errors::ErrorKind::Type))?;
        if let Some(arguments) = Self::try_collect_fast_engine_array_arguments(agent, object)? {
            return Ok(arguments);
        }
        let length = self.get_property_from_object(
            agent,
            host,
            registry,
            caller_frame,
            object,
            Value::from_object_ref(object),
            PropertyKey::from_atom(WellKnownAtom::length.id()),
        )?;
        let length =
            self.array_like_arguments_length(agent, host, registry, caller_frame, length)?;
        let mut arguments = Vec::with_capacity(usize::try_from(length).unwrap_or(usize::MAX));
        for index in 0..length {
            arguments.push(self.get_property_from_object(
                agent,
                host,
                registry,
                caller_frame,
                object,
                Value::from_object_ref(object),
                PropertyKey::Index(index),
            )?);
        }
        Ok(arguments)
    }

    fn array_like_arguments_length(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        caller_frame: FrameRecord,
        value: Value,
    ) -> VmResult<u32> {
        const MAX_SAFE_LENGTH: f64 = 9_007_199_254_740_991.0;
        let primitive = self.to_primitive(
            agent,
            host,
            registry,
            caller_frame,
            value,
            ToPrimitiveHint::Number,
        )?;
        let number = read::to_number(agent.heap().view(), primitive)
            .map_err(|_| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let number = number_value_to_f64(number);
        if number.is_nan() || number <= 0.0 {
            return Ok(0);
        }
        if !number.is_finite() {
            return Ok(u32::MAX);
        }
        Ok(number_to_u32_after_range_check(
            number.trunc().min(MAX_SAFE_LENGTH).min(f64::from(u32::MAX)),
        ))
    }

    fn try_collect_fast_engine_array_arguments(
        agent: &mut Agent,
        object: ObjectRef,
    ) -> VmResult<Option<Vec<Value>>> {
        if !agent
            .objects()
            .object_header(agent.heap().view(), object)
            .is_some_and(|header| header.flags().is_engine_array())
            || !Self::engine_array_index_prototype_chain_is_clear(agent, object)
            || agent.objects().element_mode(object) == Some(lyng_js_objects::ElementMode::Sparse)
        {
            return Ok(None);
        }

        let length_descriptor = agent
            .objects()
            .get_own_property(
                agent.heap().view(),
                object,
                PropertyKey::from_atom(WellKnownAtom::length.id()),
            )
            .map_err(|_error| VmError::Abrupt(errors::throw_type_error(agent)))?
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let Some(length_value) = length_descriptor.value() else {
            return Ok(None);
        };
        let Some(length) = length_value
            .as_smi()
            .and_then(|value| u32::try_from(value).ok())
            .or_else(|| length_value.as_f64().and_then(number_to_u32_length))
        else {
            return Ok(None);
        };
        let capacity = usize::try_from(length).unwrap_or(usize::MAX);
        let mut arguments = Vec::with_capacity(capacity);
        for index in 0..length {
            let value = agent
                .objects()
                .element(agent.heap().view(), object, index)
                .unwrap_or(Value::array_hole());
            arguments.push(if value == Value::array_hole() {
                Value::undefined()
            } else {
                value
            });
        }
        Ok(Some(arguments))
    }

    pub(super) fn try_fast_apply_builtin(
        &mut self,
        agent: &mut Agent,
        target: ObjectRef,
        _this_value: Value,
        arguments: Value,
    ) -> VmResult<Option<Value>> {
        if Self::builtin_entry(agent, target) != Some(string_from_code_point_builtin()) {
            return Ok(None);
        }
        let Some(object) = arguments.as_object_ref() else {
            return Ok(None);
        };
        if !agent
            .objects()
            .object_header(agent.heap().view(), object)
            .is_some_and(|header| header.flags().is_engine_array())
            || !Self::engine_array_index_prototype_chain_is_clear(agent, object)
            || agent.objects().element_mode(object) == Some(lyng_js_objects::ElementMode::Sparse)
        {
            return Ok(None);
        }

        let length_descriptor = agent
            .objects()
            .get_own_property(
                agent.heap().view(),
                object,
                PropertyKey::from_atom(WellKnownAtom::length.id()),
            )
            .map_err(|_error| VmError::Abrupt(errors::throw_type_error(agent)))?
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let Some(length_value) = length_descriptor.value() else {
            return Ok(None);
        };
        let Some(length) = length_value
            .as_smi()
            .and_then(|value| u32::try_from(value).ok())
            .or_else(|| length_value.as_f64().and_then(number_to_u32_length))
        else {
            return Ok(None);
        };

        let mut units = std::mem::take(&mut self.string_code_units_scratch);
        units.clear();
        units.reserve(usize::try_from(length).unwrap_or(usize::MAX));
        for index in 0..length {
            let value = agent
                .objects()
                .element(agent.heap().view(), object, index)
                .unwrap_or(Value::array_hole());
            let Some(value) = value.as_smi() else {
                self.recycle_fast_apply_string_code_units(units);
                return Ok(None);
            };
            let Ok(code_point) = u32::try_from(value) else {
                self.recycle_fast_apply_string_code_units(units);
                return Err(VmError::Abrupt(errors::throw_range_error(agent)));
            };
            if code_point > 0x0010_FFFF {
                self.recycle_fast_apply_string_code_units(units);
                return Err(VmError::Abrupt(errors::throw_range_error(agent)));
            }
            append_code_point_units(&mut units, code_point);
        }

        let string = alloc_code_unit_string(agent, &units, None);
        self.recycle_fast_apply_string_code_units(units);
        Ok(Some(Value::from_string_ref(string)))
    }

    fn recycle_fast_apply_string_code_units(&mut self, mut units: Vec<u16>) {
        if units.capacity() > MAX_FAST_APPLY_STRING_CODE_UNITS {
            return;
        }
        units.clear();
        if units.capacity() > self.string_code_units_scratch.capacity() {
            self.string_code_units_scratch = units;
        }
    }

    pub(super) fn instantiate_dynamic_function(
        &mut self,
        agent: &mut Agent,
        host: &dyn HostHooks,
        registry: &mut dyn NativeFunctionRegistry,
        realm: RealmRef,
        installed: crate::InstalledCode,
        prototype: ObjectRef,
    ) -> VmResult<ObjectRef> {
        let (lexical_env, variable_env) = if let Some(realm_record) = agent.realm(realm) {
            (realm_record.global_env(), realm_record.global_env())
        } else {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        };
        let script_referrer = Self::active_script_or_module_referrer(agent)
            .map(|key| agent.atoms_mut().intern_collectible(key.as_str()));
        let function = self
            .evaluate_entry_with_registry(
                agent,
                installed,
                lexical_env,
                variable_env,
                script_referrer,
                host,
                registry,
                None,
                None,
            )?
            .as_object_ref()
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let _ = agent.with_heap_and_objects(|heap, objects| {
            objects.set_prototype(&mut heap.mutator(), function, Some(prototype))
        });
        Ok(function)
    }

    pub(super) fn native_function_source_text(
        agent: &mut Agent,
        function: ObjectRef,
    ) -> VmResult<String> {
        let name = Self::function_name_text(agent, function)?;
        Ok(if name.is_empty() {
            "function () { [native code] }".to_owned()
        } else {
            format!("function {name}() {{ [native code] }}")
        })
    }

    pub(super) fn source_function_source_text(
        &self,
        agent: &mut Agent,
        code: lyng_js_types::CodeRef,
        function: ObjectRef,
    ) -> VmResult<String> {
        let Some(installed) = self.installed_function(code) else {
            return Self::native_function_source_text(agent, function);
        };
        if let Some(span) = installed.source_span()
            && let Some(source_text) = self.source_text(span.source)
        {
            let start = usize::try_from(span.range.start.raw()).unwrap_or(usize::MAX);
            let end = usize::try_from(span.range.end.raw()).unwrap_or(usize::MAX);
            if start <= end && end <= source_text.len() {
                let candidate = &source_text[start..end];
                if let Some(trimmed) = Self::trim_function_source_prefix(span.source, candidate) {
                    return Ok(trimmed);
                }
                return Ok(candidate.to_owned());
            }
        }
        let name = Self::function_name_text(agent, function)?;
        Ok(if name.is_empty() {
            "function () { [source unavailable] }".to_owned()
        } else {
            format!("function {name}() {{ [source unavailable] }}")
        })
    }

    fn function_name_text(agent: &mut Agent, function: ObjectRef) -> VmResult<String> {
        let name = object::ordinary_get(
            agent,
            function,
            PropertyKey::from_atom(WellKnownAtom::name.id()),
        )
        .map_err(VmError::Abrupt)?;
        Self::value_to_string_text(agent, name)
    }

    fn trim_function_source_prefix(
        source: lyng_js_common::SourceId,
        candidate: &str,
    ) -> Option<String> {
        let mut atoms = AtomTable::new();
        for (index, ch) in candidate.char_indices() {
            if ch != '}' {
                continue;
            }
            let end = index + ch.len_utf8();
            let source_text = &candidate[..end];
            if Self::function_source_candidate_parses(&mut atoms, source, source_text) {
                return Some(candidate[..end].to_owned());
            }
        }
        None
    }

    fn function_source_candidate_parses(
        atoms: &mut AtomTable,
        source: lyng_js_common::SourceId,
        source_text: &str,
    ) -> bool {
        if !parse_script(atoms, source, source_text)
            .diagnostics
            .has_errors()
        {
            return true;
        }

        let expression_text = format!("({source_text});");
        if !parse_script(atoms, source, &expression_text)
            .diagnostics
            .has_errors()
        {
            return true;
        }

        let method_text = format!("({{{source_text}}});");
        if !parse_script(atoms, source, &method_text)
            .diagnostics
            .has_errors()
        {
            return true;
        }

        let class_method_text = format!("(class {{{source_text}}});");
        !parse_script(atoms, source, &class_method_text)
            .diagnostics
            .has_errors()
    }
}

fn append_code_point_units(units: &mut Vec<u16>, code_point: u32) {
    if code_point <= 0xFFFF {
        units.push(u16::try_from(code_point).expect("BMP code point should fit into u16"));
        return;
    }

    let adjusted = code_point - 0x1_0000;
    let high = u16::try_from(adjusted >> 10).expect("high surrogate payload should fit into u16");
    let low = u16::try_from(adjusted & 0x03FF).expect("low surrogate payload should fit into u16");
    units.push(0xD800_u16 | high);
    units.push(0xDC00_u16 | low);
}

fn bound_target_function_data(
    agent: &mut Agent,
    mut target: ObjectRef,
) -> VmResult<Option<FunctionObjectData>> {
    loop {
        if let Some(data) = agent.objects().function_data(target) {
            return Ok(Some(data.clone()));
        }
        if !agent.objects().is_proxy_object(target) {
            return Ok(None);
        }
        let Some(data) = agent.objects().proxy_data(target) else {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        };
        if data.revoked() {
            return Err(VmError::Abrupt(errors::throw_type_error(agent)));
        }
        target = data.target();
    }
}

fn number_to_u32_length(value: f64) -> Option<u32> {
    #[allow(
        clippy::float_cmp,
        reason = "array-like fast path accepts only exactly integral Number lengths"
    )]
    if !value.is_finite() || value < 0.0 || value.trunc() != value || value > f64::from(u32::MAX) {
        return None;
    }
    Some(number_to_u32_after_range_check(value))
}

fn number_value_to_f64(value: Value) -> f64 {
    value
        .as_smi()
        .map(f64::from)
        .or_else(|| value.as_f64())
        .expect("ToNumber must produce a numeric value")
}

fn bound_function_length_value(target_length: Value, bound_argument_count: usize) -> Value {
    let Some(number) = target_length.as_f64() else {
        return Value::from_smi(0);
    };

    let length = if number.is_nan() || number == 0.0 {
        0.0
    } else if number.is_infinite() {
        if number.is_sign_positive() {
            f64::INFINITY
        } else {
            0.0
        }
    } else {
        (number.trunc() - usize_to_f64_count(bound_argument_count)).max(0.0)
    };

    if length.is_infinite() {
        return Value::from_f64(length);
    }
    if let Ok(integer) = i32::try_from(number_to_i64_after_range_check(length)) {
        #[allow(
            clippy::float_cmp,
            reason = "Smi encoding is used only when the computed length is exactly representable"
        )]
        if f64::from(integer) == length {
            return Value::from_smi(integer);
        }
    }
    Value::from_f64(length)
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

const fn number_to_i64_after_range_check(number: f64) -> i64 {
    #[allow(
        clippy::cast_possible_truncation,
        reason = "finite length is narrowed only to probe Smi fit; out-of-range values fail the following i32 check"
    )]
    let integer = number as i64;
    integer
}

const fn usize_to_f64_count(value: usize) -> f64 {
    #[allow(
        clippy::cast_precision_loss,
        reason = "bound argument counts are represented as ECMAScript Number lengths"
    )]
    let number = value as f64;
    number
}
