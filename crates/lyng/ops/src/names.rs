use crate::{
    errors::{internal_method_error, throw_reference_error, throw_type_error},
    object, read,
};
use lyng_js_common::AtomId;
use lyng_js_env::{Agent, EnvironmentLayoutId, EnvironmentRecord, ObjectEnvironmentRecord};
use lyng_js_gc::AllocationLifetime;
use lyng_js_types::{Completion, EnvironmentRef, PropertyKey, Value, WellKnownSymbolId};

fn lookup_environment_binding(
    agent: &Agent,
    environment: EnvironmentRef,
    layout: EnvironmentLayoutId,
    name: AtomId,
) -> Option<Value> {
    let layout = agent.environment_layout(layout)?;
    let (index, binding) = layout
        .bindings()
        .iter()
        .enumerate()
        .find(|(_, binding)| binding.name() == Some(name))?;
    let index = u32::try_from(index).expect("environment slot index must fit into u32");
    let value = agent.environment_slot(environment, index)?;
    if binding.flags().is_dynamic() && value == Value::deleted_environment_binding() {
        return None;
    }
    Some(value)
}

fn lookup_global_lexical_binding(
    agent: &Agent,
    record: &lyng_js_env::GlobalEnvironmentRecord,
    name: AtomId,
) -> Option<Value> {
    let binding = record.lexical_binding(name)?;
    agent.environment_slot(binding.environment(), binding.slot())
}

/// ECMAScript `HasBinding` for one object environment record, including
/// `@@unscopables` filtering for `with` environments.
///
/// # Errors
/// Returns abrupt completions from proxy/property access or boolean coercion.
pub fn object_environment_has_binding(
    agent: &mut Agent,
    record: ObjectEnvironmentRecord,
    name: AtomId,
) -> Completion<bool> {
    let key = PropertyKey::from_atom(name);
    let binding_object = record.binding_object();
    let receiver = Value::from_object_ref(binding_object);

    let found = object::ordinary_has_property(agent, binding_object, key)?;
    if !found {
        return Ok(false);
    }
    if !record.with_environment() {
        return Ok(true);
    }

    let Some(unscopables_symbol) = agent.well_known_symbol(WellKnownSymbolId::Unscopables) else {
        return Ok(true);
    };
    let unscopables = object::ordinary_get_with_receiver(
        agent,
        binding_object,
        PropertyKey::from_symbol(unscopables_symbol),
        receiver,
    )?;
    let Some(unscopables_object) = unscopables.as_object_ref() else {
        return Ok(true);
    };

    let blocked = object::ordinary_get_with_receiver(
        agent,
        unscopables_object,
        key,
        Value::from_object_ref(unscopables_object),
    )?;
    Ok(!read::to_boolean_agent(agent, blocked)?)
}

/// ECMAScript `GetBindingValue` for one object environment record.
///
/// # Errors
/// Returns abrupt completions from proxy/property access or a strict-mode
/// `ReferenceError` when the binding no longer exists.
pub fn object_environment_get_binding_value(
    agent: &mut Agent,
    record: ObjectEnvironmentRecord,
    name: AtomId,
    strict: bool,
) -> Completion<Value> {
    let key = PropertyKey::from_atom(name);
    let binding_object = record.binding_object();
    let still_exists = object::ordinary_has_property(agent, binding_object, key)?;
    if !still_exists {
        if strict {
            return Err(throw_reference_error(agent));
        }
        return Ok(Value::undefined());
    }
    object::ordinary_get(agent, binding_object, key)
}

/// ECMAScript `SetMutableBinding` for one object environment record.
///
/// # Errors
/// Returns abrupt completions from proxy/property access and strict-mode
/// `ReferenceError`/`TypeError` results from the underlying set.
pub fn object_environment_set_mutable_binding(
    agent: &mut Agent,
    record: ObjectEnvironmentRecord,
    name: AtomId,
    value: Value,
    strict: bool,
    lifetime: AllocationLifetime,
) -> Completion<()> {
    let key = PropertyKey::from_atom(name);
    let binding_object = record.binding_object();
    let still_exists = object::ordinary_has_property(agent, binding_object, key)?;
    if !still_exists && strict {
        return Err(throw_reference_error(agent));
    }

    let stored = object::ordinary_set(agent, binding_object, key, value, lifetime)?;
    if !stored && strict {
        return Err(throw_type_error(agent));
    }
    Ok(())
}

/// Probes one identifier binding through the environment chain without throwing
/// on an unresolvable name.
///
/// # Errors
/// Returns an abrupt completion if the environment chain is corrupt or if an
/// object/global-environment property access fails.
pub fn probe_identifier_value(
    agent: &mut Agent,
    start: EnvironmentRef,
    name: AtomId,
    strict: bool,
) -> Completion<Option<Value>> {
    let key = PropertyKey::from_atom(name);
    let mut current = Some(start);

    while let Some(environment) = current {
        let record = agent
            .environment(environment)
            .ok_or_else(|| throw_type_error(agent))?;
        match record {
            EnvironmentRecord::Declarative(record) => {
                if let Some(value) =
                    lookup_environment_binding(agent, record.id(), record.layout(), name)
                {
                    return Ok(Some(value));
                }
                current = record.outer();
            }
            EnvironmentRecord::Private(record) => {
                current = record.outer();
            }
            EnvironmentRecord::Function(record) => {
                let declarative = record.declarative();
                if let Some(value) =
                    lookup_environment_binding(agent, declarative.id(), declarative.layout(), name)
                {
                    return Ok(Some(value));
                }
                current = declarative.outer();
            }
            EnvironmentRecord::Module(record) => {
                if let Some(value) =
                    lookup_environment_binding(agent, record.id(), record.layout(), name)
                {
                    return Ok(Some(value));
                }
                current = record.outer();
            }
            EnvironmentRecord::Global(record) => {
                if let Some(value) = lookup_global_lexical_binding(agent, &record, name) {
                    return Ok(Some(value));
                }
                if let Some(value) =
                    lookup_environment_binding(agent, record.id(), record.layout(), name)
                {
                    return Ok(Some(value));
                }
                if record.has_var_name(name) {
                    let value = agent
                        .objects()
                        .get(
                            agent.heap().view(),
                            record.global_object(),
                            key,
                            Value::from_object_ref(record.global_object()),
                        )
                        .map_err(|error| internal_method_error(agent, error))?;
                    return Ok(Some(value));
                }
                current = record.outer();
            }
            EnvironmentRecord::Object(record) => {
                if object_environment_has_binding(agent, record, name)? {
                    let value = object_environment_get_binding_value(agent, record, name, strict)?;
                    return Ok(Some(value));
                }
                current = record.outer();
            }
        }
    }

    Ok(None)
}

/// Reports whether one identifier name resolves anywhere in the current environment chain.
#[inline]
///
/// # Errors
/// Propagates the same abrupt completions as [`probe_identifier_value`].
pub fn has_identifier_binding(
    agent: &mut Agent,
    start: EnvironmentRef,
    name: AtomId,
) -> Completion<bool> {
    probe_identifier_value(agent, start, name, false).map(|value| value.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lyng_js_env::{
        Agent, EnvironmentBindingLayout, EnvironmentLayout, EnvironmentLayoutKind,
        EnvironmentSlotFlags, Runtime,
    };
    use lyng_js_gc::AllocationLifetime;
    use lyng_js_host::NoopHostHooks;
    use lyng_js_objects::ObjectAllocation;

    const GLOBAL_LEXICAL: AtomId = AtomId::from_raw(601);
    const GLOBAL_VAR: AtomId = AtomId::from_raw(602);
    const OBJECT_NAME: AtomId = AtomId::from_raw(603);
    const MISSING_NAME: AtomId = AtomId::from_raw(604);

    fn configure_environment_chain(agent: &mut Agent) -> (EnvironmentRef, EnvironmentRef) {
        let default_realm = agent.default_realm().expect("default realm should exist");
        let root_shape = default_realm
            .root_shape()
            .expect("default realm should expose a root shape");
        let (global_object, binding_object) = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            let global_object = objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape),
                AllocationLifetime::Default,
            );
            let binding_object = objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape),
                AllocationLifetime::Default,
            );
            (global_object, binding_object)
        });

        let global_layout = agent.alloc_environment_layout(EnvironmentLayout::new(
            EnvironmentLayoutKind::Global,
            [EnvironmentBindingLayout::new(
                Some(GLOBAL_LEXICAL),
                EnvironmentSlotFlags::mutable_lexical(),
            )],
            true,
        ));
        let global_env = agent
            .alloc_global_environment(
                None,
                global_layout,
                global_object,
                AllocationLifetime::Default,
            )
            .expect("global environment should allocate");
        assert!(agent.init_environment_slot(global_env, 0, Value::from_smi(10)));
        assert!(agent.global_add_var_name(global_env, GLOBAL_VAR));

        agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            let mut descriptor = lyng_js_types::PropertyDescriptor::new();
            descriptor.set_value(Value::from_smi(20));
            descriptor.set_writable(true);
            descriptor.set_enumerable(true);
            descriptor.set_configurable(true);
            assert!(objects
                .define_own_property(
                    &mut mutator,
                    global_object,
                    PropertyKey::from_atom(GLOBAL_VAR),
                    descriptor,
                    AllocationLifetime::Default,
                )
                .unwrap());

            let mut binding = lyng_js_types::PropertyDescriptor::new();
            binding.set_value(Value::from_smi(30));
            binding.set_writable(true);
            binding.set_enumerable(true);
            binding.set_configurable(true);
            assert!(objects
                .define_own_property(
                    &mut mutator,
                    binding_object,
                    PropertyKey::from_atom(OBJECT_NAME),
                    binding,
                    AllocationLifetime::Default,
                )
                .unwrap());
        });

        let object_env = agent.alloc_object_environment(
            Some(global_env),
            binding_object,
            true,
            AllocationLifetime::Default,
        );

        (global_env, object_env)
    }

    #[test]
    fn name_probe_handles_object_global_and_missing_bindings() {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let (global_env, object_env) = configure_environment_chain(agent);

        assert_eq!(
            probe_identifier_value(agent, object_env, OBJECT_NAME, false).unwrap(),
            Some(Value::from_smi(30))
        );
        assert_eq!(
            probe_identifier_value(agent, object_env, GLOBAL_LEXICAL, false).unwrap(),
            Some(Value::from_smi(10))
        );
        assert_eq!(
            probe_identifier_value(agent, object_env, GLOBAL_VAR, false).unwrap(),
            Some(Value::from_smi(20))
        );
        assert_eq!(
            probe_identifier_value(agent, object_env, MISSING_NAME, false).unwrap(),
            None
        );
        assert!(has_identifier_binding(agent, object_env, GLOBAL_VAR).unwrap());
        assert!(!has_identifier_binding(agent, global_env, MISSING_NAME).unwrap());
    }
}
