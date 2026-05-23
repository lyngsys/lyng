use crate::{errors::internal_method_error, typed_array};
use lyng_js_env::Agent;
use lyng_js_types::{Completion, ObjectRef, PropertyDescriptor, PropertyKey};

pub(super) use crate::typed_array::NumericKey as TypedArrayNumericKey;

pub(super) fn typed_array_numeric_key(
    agent: &Agent,
    object: ObjectRef,
    key: PropertyKey,
) -> Option<typed_array::NumericKey> {
    typed_array::numeric_key(agent, object, key)
}

pub fn is_typed_array_numeric_key(agent: &Agent, object: ObjectRef, key: PropertyKey) -> bool {
    typed_array::is_numeric_key(agent, object, key)
}

pub(super) fn typed_array_index_descriptor(
    agent: &mut Agent,
    object: ObjectRef,
    index: usize,
) -> Option<PropertyDescriptor> {
    let record = agent.objects().typed_array(object)?;
    let bits = typed_array::read_storage_bits(agent, record, index)?;
    let mut descriptor = PropertyDescriptor::new();
    descriptor.set_value(typed_array::value_from_storage_bits(
        agent,
        record.kind(),
        bits,
    ));
    descriptor.set_writable(true);
    descriptor.set_enumerable(true);
    descriptor.set_configurable(true);
    Some(descriptor)
}

pub(super) fn typed_array_own_property_keys(
    agent: &mut Agent,
    object: ObjectRef,
) -> Completion<Option<Vec<PropertyKey>>> {
    let Some(record) = agent.objects().typed_array(object) else {
        return Ok(None);
    };
    let length = typed_array::current_length(agent, record).unwrap_or(0);
    let mut keys = (0..u32::try_from(length).unwrap_or(u32::MAX))
        .map(PropertyKey::Index)
        .collect::<Vec<_>>();
    let own_keys = match agent
        .objects()
        .own_property_keys(agent.heap().view(), object)
    {
        Ok(keys) => keys,
        Err(error) => return Err(internal_method_error(agent, error)),
    };
    keys.extend(own_keys.into_iter().filter(|key| key.as_index().is_none()));
    Ok(Some(keys))
}
