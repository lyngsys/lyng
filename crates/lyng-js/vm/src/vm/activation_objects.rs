use super::runtime_objects::length_value;
use super::{
    Agent, AllocationLifetime, ArgumentsMode, EnvironmentRef, ObjectAllocation, ObjectRef,
    RealmRef, Value, Vm, VmError, VmResult, WellKnownAtom,
};
use lyng_js_objects::ObjectFlags;
use lyng_js_ops::{errors, object};
use lyng_js_types::{PropertyDescriptor, PropertyKey, WellKnownSymbolId};

impl Vm {
    pub(super) fn create_arguments_object(
        &mut self,
        agent: &mut Agent,
        realm: RealmRef,
        arguments: &[Value],
        callee: Value,
        mapped_slots: Option<Vec<Option<u32>>>,
        environment: EnvironmentRef,
        restricted_callee: bool,
    ) -> VmResult<ObjectRef> {
        let root_shape = agent
            .realm(realm)
            .and_then(lyng_js_env::RealmRecord::root_shape)
            .ok_or(VmError::MissingRootShape(realm))?;
        let object_prototype = agent
            .realm(realm)
            .and_then(|record| record.intrinsics().object_prototype());
        let object = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape)
                    .with_flags(ObjectFlags::extensible().union(ObjectFlags::ARGUMENTS_OBJECT))
                    .with_prototype(object_prototype)
                    .with_element_capacity(arguments.len()),
                AllocationLifetime::Default,
            )
        });

        for (index, value) in arguments.iter().copied().enumerate() {
            agent.with_heap_and_objects(|heap, objects| {
                let mut mutator = heap.mutator();
                objects.set_element(
                    &mut mutator,
                    object,
                    u32::try_from(index).expect("arguments index should fit u32"),
                    value,
                    AllocationLifetime::Default,
                )
            });
        }
        Self::define_length_property(
            agent,
            object,
            u32::try_from(arguments.len()).unwrap_or(u32::MAX),
            true,
        )?;

        if let Some(iterator_symbol) = agent.well_known_symbol(WellKnownSymbolId::Iterator)
            && let Some(array_prototype) = agent
                .realm(realm)
                .and_then(|record| record.intrinsics().array_prototype())
        {
            let iterator_method = object::ordinary_get(
                agent,
                array_prototype,
                PropertyKey::from_symbol(iterator_symbol),
            )
            .map_err(VmError::Abrupt)?;
            if !iterator_method.is_undefined() {
                Self::define_data_property_with_attrs(
                    agent,
                    object,
                    PropertyKey::from_symbol(iterator_symbol),
                    iterator_method,
                    true,
                    false,
                    true,
                )?;
            }
        }

        if let Some(mapped_slots) = mapped_slots {
            self.activation_tables.track_mapped_arguments(
                object,
                crate::activation::MappedArgumentsObject::new(environment, mapped_slots),
            );
        }

        if restricted_callee {
            Self::define_strict_arguments_callee(agent, realm, object)?;
        } else {
            let callee_key = agent.atoms_mut().intern_collectible("callee");
            Self::define_data_property_with_attrs(
                agent,
                object,
                PropertyKey::from_atom(callee_key),
                callee,
                true,
                false,
                true,
            )?;
        }

        Ok(object)
    }

    fn define_strict_arguments_callee(
        agent: &mut Agent,
        realm: RealmRef,
        object: ObjectRef,
    ) -> VmResult<()> {
        let caller = agent.atoms_mut().intern_collectible("caller");
        let function_prototype = agent
            .realm(realm)
            .and_then(|record| record.intrinsics().function_prototype())
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let thrower = object::ordinary_get_own_property(
            agent,
            function_prototype,
            PropertyKey::from_atom(caller),
        )
        .map_err(VmError::Abrupt)?
        .and_then(|descriptor| descriptor.getter().zip(descriptor.setter()))
        .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let callee_atom = agent.atoms_mut().intern_collectible("callee");
        let mut descriptor = PropertyDescriptor::new();
        descriptor.set_getter(thrower.0);
        descriptor.set_setter(thrower.1);
        descriptor.set_enumerable(false);
        descriptor.set_configurable(false);
        let defined = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.define_own_property(
                &mut mutator,
                object,
                PropertyKey::from_atom(callee_atom),
                descriptor,
                AllocationLifetime::Default,
            )
        });
        let _ = defined.map_err(|_error| VmError::Abrupt(errors::throw_type_error(agent)))?;
        Ok(())
    }

    pub(super) fn define_length_property(
        agent: &mut Agent,
        object: ObjectRef,
        length: u32,
        configurable: bool,
    ) -> VmResult<()> {
        Self::define_length_property_with_attrs(agent, object, length, true, configurable)
    }

    pub(super) fn define_length_property_with_attrs(
        agent: &mut Agent,
        object: ObjectRef,
        length: u32,
        writable: bool,
        configurable: bool,
    ) -> VmResult<()> {
        let mut descriptor = PropertyDescriptor::new();
        descriptor.set_value(length_value(length));
        descriptor.set_writable(writable);
        descriptor.set_enumerable(false);
        descriptor.set_configurable(configurable);
        let defined = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.define_own_property(
                &mut mutator,
                object,
                PropertyKey::from_atom(WellKnownAtom::length.id()),
                descriptor,
                AllocationLifetime::Default,
            )
        });
        let _ = defined.map_err(|_error| VmError::Abrupt(errors::throw_type_error(agent)))?;
        Ok(())
    }

    pub(super) fn sync_engine_array_length(
        agent: &mut Agent,
        object: ObjectRef,
    ) -> VmResult<()> {
        let is_engine_array = agent
            .objects()
            .object_header(agent.heap().view(), object)
            .is_some_and(|header| header.flags().is_engine_array());
        if !is_engine_array {
            return Ok(());
        }
        let logical_len = agent.objects().element_logical_len(object).unwrap_or(0);
        let (current_len, writable) = agent
            .objects()
            .get_own_property(
                agent.heap().view(),
                object,
                PropertyKey::from_atom(WellKnownAtom::length.id()),
            )
            .ok()
            .flatten()
            .map_or((logical_len, true), |descriptor| {
                let current_len = descriptor
                    .value()
                    .and_then(|value| value.as_smi().and_then(|value| u32::try_from(value).ok()))
                    .or_else(|| {
                        descriptor
                            .value()
                            .and_then(|value| value.as_f64().map(|value| value.max(0.0) as u32))
                    })
                    .unwrap_or(logical_len);
                (current_len, descriptor.writable().unwrap_or(true))
            });
        Self::define_length_property_with_attrs(
            agent,
            object,
            current_len.max(logical_len),
            writable,
            false,
        )
    }

    pub(super) fn initialize_activation_objects(
        &mut self,
        agent: &mut Agent,
        init: ActivationObjectInit<'_>,
    ) -> VmResult<()> {
        if init.arguments_mode == ArgumentsMode::None && !init.has_rest_parameter {
            return Ok(());
        }

        if let Some(rest_slot) = function_rest_slot(init.has_rest_parameter) {
            let rest_start = usize::from(init.parameter_count).min(init.arguments.len());
            let rest_array = Self::create_array(
                agent,
                init.realm,
                init.arguments.len().saturating_sub(rest_start),
            )?;
            for (index, value) in init.arguments[rest_start..].iter().copied().enumerate() {
                agent.with_heap_and_objects(|heap, objects| {
                    let mut mutator = heap.mutator();
                    objects.set_element(
                        &mut mutator,
                        rest_array,
                        u32::try_from(index).expect("rest index should fit u32"),
                        value,
                        AllocationLifetime::Default,
                    )
                });
            }
            Self::sync_engine_array_length(agent, rest_array)?;
            Self::initialize_environment_slot(
                agent,
                init.lexical_env,
                rest_slot,
                Value::from_object_ref(rest_array),
            )?;
        }

        let Some(arguments_slot) = function_arguments_slot(
            init.parameter_count,
            init.arguments_mode,
            init.has_rest_parameter,
        ) else {
            return Ok(());
        };
        let mapped_slots = if init.arguments_mode == ArgumentsMode::Mapped {
            let mapped_count = usize::from(init.parameter_count).min(init.arguments.len());
            Some(
                (0..init.arguments.len())
                    .map(|index| {
                        (index < mapped_count).then_some(u32::try_from(index).unwrap_or(u32::MAX))
                    })
                    .collect::<Vec<_>>(),
            )
        } else {
            None
        };
        let arguments_object = self.create_arguments_object(
            agent,
            init.realm,
            init.arguments,
            init.callee,
            mapped_slots,
            init.lexical_env,
            init.arguments_mode == ArgumentsMode::Unmapped,
        )?;
        Self::initialize_environment_slot(
            agent,
            init.lexical_env,
            arguments_slot,
            Value::from_object_ref(arguments_object),
        )?;
        Ok(())
    }

    pub(super) fn finalize_mapped_arguments(
        &mut self,
        agent: &mut Agent,
        environment: EnvironmentRef,
    ) -> VmResult<()> {
        if self
            .frames
            .iter()
            .any(|frame| frame.lexical_env() == environment)
        {
            return Ok(());
        }
        for (object, mapped) in self
            .activation_tables
            .drain_mapped_arguments_for_environment(environment)
        {
            for (index, slot) in mapped.mapped_indices() {
                let value = Self::read_environment_slot_raw(agent, environment, slot)?;
                agent.with_heap_and_objects(|heap, objects| {
                    let mut mutator = heap.mutator();
                    objects.set_element(
                        &mut mutator,
                        object,
                        index,
                        value,
                        AllocationLifetime::Default,
                    )
                });
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub(super) struct ActivationObjectInit<'a> {
    pub(super) realm: RealmRef,
    pub(super) parameter_count: u16,
    pub(super) arguments_mode: ArgumentsMode,
    pub(super) has_rest_parameter: bool,
    pub(super) lexical_env: EnvironmentRef,
    pub(super) arguments: &'a [Value],
    pub(super) callee: Value,
}

fn function_rest_slot(has_rest_parameter: bool) -> Option<u32> {
    has_rest_parameter.then_some(0)
}

fn function_arguments_slot(
    parameter_count: u16,
    arguments_mode: ArgumentsMode,
    has_rest_parameter: bool,
) -> Option<u32> {
    match arguments_mode {
        ArgumentsMode::None => None,
        ArgumentsMode::Mapped => Some(u32::from(parameter_count)),
        ArgumentsMode::Unmapped => Some(u32::from(has_rest_parameter)),
    }
}
