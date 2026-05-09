use super::{
    Agent, AllocationLifetime, AtomId, ObjectRef, RealmRecord, Value, Vm, VmError, VmResult,
};
use lyng_js_bytecode::GlobalScriptInstantiationPlan;
use lyng_js_ops::{errors, object};
use lyng_js_types::{PropertyDescriptor, PropertyKey};

const BULK_GLOBAL_BINDING_DICTIONARY_THRESHOLD: usize = 64;

impl Vm {
    /// # Errors
    ///
    /// Returns a VM error if global declaration instantiation fails.
    pub fn instantiate_global_script(
        agent: &mut Agent,
        realm: &RealmRecord,
        plan: &GlobalScriptInstantiationPlan,
    ) -> VmResult<()> {
        if plan.lexical_names().is_empty()
            && plan.function_names().is_empty()
            && plan.var_names().is_empty()
        {
            return Ok(());
        }

        let global_env = realm.global_env();
        let global_object = realm.global_object();

        for name in plan.lexical_names() {
            let name = agent.atoms_mut().intern_collectible(name);
            if Self::global_chain_has_lexical_binding(agent, global_env, name) {
                return Err(VmError::Abrupt(errors::throw_syntax_error(agent)));
            }
            if has_restricted_global_property(agent, global_object, name)? {
                return Err(VmError::Abrupt(errors::throw_syntax_error(agent)));
            }
        }

        for name in plan.function_names() {
            let name = agent.atoms_mut().intern_collectible(name);
            if Self::global_chain_has_lexical_binding(agent, global_env, name) {
                return Err(VmError::Abrupt(errors::throw_syntax_error(agent)));
            }
            if !can_declare_global_function(agent, global_object, name)? {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
        }

        for name in plan.var_names() {
            let name = agent.atoms_mut().intern_collectible(name);
            if Self::global_chain_has_lexical_binding(agent, global_env, name) {
                return Err(VmError::Abrupt(errors::throw_syntax_error(agent)));
            }
            if !can_declare_global_var(agent, global_object, name)? {
                return Err(VmError::Abrupt(errors::throw_type_error(agent)));
            }
        }

        if plan.function_names().len() + plan.var_names().len()
            >= BULK_GLOBAL_BINDING_DICTIONARY_THRESHOLD
        {
            ensure_global_object_dictionary(agent, global_object)?;
        }

        for name in plan.lexical_names() {
            let name = agent.atoms_mut().intern_collectible(name);
            let _ = agent.global_add_lexical_name(global_env, name);
        }

        for name in plan.function_names() {
            let name = agent.atoms_mut().intern_collectible(name);
            create_global_function_binding(agent, global_object, name)?;
            if !agent.global_has_var_name(global_env, name) {
                let _ = agent.global_add_var_name(global_env, name);
            }
        }

        for name in plan.var_names() {
            let name = agent.atoms_mut().intern_collectible(name);
            create_global_var_binding(agent, global_object, name)?;
            if !agent.global_has_var_name(global_env, name) {
                let _ = agent.global_add_var_name(global_env, name);
            }
        }

        Ok(())
    }
}

fn ensure_global_object_dictionary(agent: &mut Agent, global_object: ObjectRef) -> VmResult<()> {
    let converted = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.ensure_named_property_dictionary(&mut mutator, global_object)
    });
    if converted {
        Ok(())
    } else {
        Err(VmError::Abrupt(errors::throw_type_error(agent)))
    }
}

fn has_restricted_global_property(
    agent: &mut Agent,
    global_object: ObjectRef,
    name: AtomId,
) -> VmResult<bool> {
    let descriptor =
        object::ordinary_get_own_property(agent, global_object, PropertyKey::from_atom(name))
            .map_err(VmError::Abrupt)?;
    Ok(descriptor.is_some_and(|descriptor| descriptor.configurable() == Some(false)))
}

fn can_declare_global_var(
    agent: &mut Agent,
    global_object: ObjectRef,
    name: AtomId,
) -> VmResult<bool> {
    let descriptor =
        object::ordinary_get_own_property(agent, global_object, PropertyKey::from_atom(name))
            .map_err(VmError::Abrupt)?;
    if descriptor.is_some() {
        return Ok(true);
    }
    object::ordinary_is_extensible(agent, global_object).map_err(VmError::Abrupt)
}

fn can_declare_global_function(
    agent: &mut Agent,
    global_object: ObjectRef,
    name: AtomId,
) -> VmResult<bool> {
    let descriptor =
        object::ordinary_get_own_property(agent, global_object, PropertyKey::from_atom(name))
            .map_err(VmError::Abrupt)?;
    let Some(descriptor) = descriptor else {
        return object::ordinary_is_extensible(agent, global_object).map_err(VmError::Abrupt);
    };
    if descriptor.configurable() == Some(true) {
        return Ok(true);
    }
    Ok(is_data_descriptor(descriptor)
        && descriptor.writable() == Some(true)
        && descriptor.enumerable() == Some(true))
}

fn create_global_var_binding(
    agent: &mut Agent,
    global_object: ObjectRef,
    name: AtomId,
) -> VmResult<()> {
    let key = PropertyKey::from_atom(name);
    if object::ordinary_get_own_property(agent, global_object, key)
        .map_err(VmError::Abrupt)?
        .is_some()
    {
        return Ok(());
    }
    define_global_binding_property(agent, global_object, key)
}

fn create_global_function_binding(
    agent: &mut Agent,
    global_object: ObjectRef,
    name: AtomId,
) -> VmResult<()> {
    define_global_binding_property(agent, global_object, PropertyKey::from_atom(name))
}

fn define_global_binding_property(
    agent: &mut Agent,
    global_object: ObjectRef,
    key: PropertyKey,
) -> VmResult<()> {
    let mut descriptor = PropertyDescriptor::new();
    descriptor.set_value(Value::undefined());
    descriptor.set_writable(true);
    descriptor.set_enumerable(true);
    descriptor.set_configurable(false);
    let defined = object::ordinary_define_property(
        agent,
        global_object,
        key,
        descriptor,
        AllocationLifetime::Default,
    )
    .map_err(VmError::Abrupt)?;
    if defined {
        Ok(())
    } else {
        Err(VmError::Abrupt(errors::throw_type_error(agent)))
    }
}

const fn is_data_descriptor(descriptor: PropertyDescriptor) -> bool {
    descriptor.has_value() || descriptor.has_writable()
}
