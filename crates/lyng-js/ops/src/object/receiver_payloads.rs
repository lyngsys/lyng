use crate::errors::throw_type_error;
use lyng_js_env::Agent;
use lyng_js_objects::{TemporalObjectData, TemporalObjectKind};
use lyng_js_types::{Completion, Value};

/// Returns the stored time-value payload for a Date receiver.
///
/// # Errors
/// Returns a `TypeError` abrupt completion when the receiver is not a Date
/// object with an installed ordinary payload value.
pub fn require_date_value(agent: &mut Agent, value: Value) -> Completion<Value> {
    let Some(object) = value.as_object_ref() else {
        return Err(throw_type_error(agent));
    };
    if !agent.objects().is_date_object(object) {
        return Err(throw_type_error(agent));
    }

    agent
        .objects()
        .date_value(agent.heap().view(), object)
        .ok_or_else(|| throw_type_error(agent))
}

/// Returns the typed Temporal payload for one matching Temporal receiver.
///
/// # Errors
/// Returns a `TypeError` abrupt completion when the receiver is not a Temporal
/// object of the requested kind with an installed typed payload.
pub fn require_temporal_object(
    agent: &mut Agent,
    value: Value,
    expected: TemporalObjectKind,
) -> Completion<TemporalObjectData> {
    let Some(object) = value.as_object_ref() else {
        return Err(throw_type_error(agent));
    };
    if !agent.objects().is_temporal_object_kind(object, expected) {
        return Err(throw_type_error(agent));
    }

    agent
        .objects()
        .temporal_object(object)
        .copied()
        .ok_or_else(|| throw_type_error(agent))
}
