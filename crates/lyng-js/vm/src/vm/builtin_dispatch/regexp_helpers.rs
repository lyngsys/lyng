use super::{errors, Agent, FrameRecord, Value, Vm, VmError, VmResult};
use lyng_js_objects::RegExpPayload;
use lyng_js_parser::validate_regexp_literal;

impl Vm {
    pub(super) fn regexp_literal_builtin(
        agent: &mut Agent,
        caller: &FrameRecord,
        arguments: &[Value],
    ) -> VmResult<Value> {
        let site = arguments
            .first()
            .copied()
            .and_then(Value::as_smi)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(agent)))?;
        let pattern_value = arguments.get(1).copied().unwrap_or(Value::undefined());
        let flags_value = arguments.get(2).copied().unwrap_or(Value::undefined());
        let pattern_text = Self::value_to_string_text(agent, pattern_value)?;
        let flags_text = Self::value_to_string_text(agent, flags_value)?;
        let realm = caller.realm();

        if validate_regexp_literal(&pattern_text, &flags_text).is_err() {
            return Err(Self::abrupt_intrinsic_error(
                agent,
                realm,
                errors::ErrorKind::Syntax,
            ));
        }

        let payload = if let Some(payload) = agent
            .regexp_literal_cached_payload(realm, caller.code(), site)
            .cloned()
        {
            payload
        } else {
            let payload = RegExpPayload::compile(&pattern_text, &flags_text).map_err(|_| {
                Self::abrupt_intrinsic_error(agent, realm, errors::ErrorKind::Syntax)
            })?;
            let _ = agent.cache_regexp_literal_payload(realm, caller.code(), site, payload.clone());
            payload
        };

        let object = Self::allocate_regexp_object_with_payload(agent, realm, payload)?;
        Ok(Value::from_object_ref(object))
    }
}
