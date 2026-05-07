use crate::errors::throw_type_error;
use lyng_js_env::{
    Agent, ExecutableId, PromiseReactionHandler, PromiseReactionKind, PromiseReactionRecord,
    PromiseState, RuntimeJobPayload,
};
use lyng_js_host::HostJobKind;
use lyng_js_types::{Completion, ObjectRef, Value};

#[inline]
pub fn is_promise(agent: &Agent, value: Value) -> bool {
    value
        .as_object_ref()
        .and_then(|object| agent.promise_record(object))
        .is_some()
}

#[inline]
pub fn promise_state(agent: &Agent, promise: ObjectRef) -> Option<PromiseState> {
    agent
        .promise_record(promise)
        .map(lyng_js_env::PromiseRecord::state)
}

#[inline]
pub fn promise_result(agent: &Agent, promise: ObjectRef) -> Option<Value> {
    agent
        .promise_record(promise)
        .map(lyng_js_env::PromiseRecord::result)
}

#[inline]
pub fn create_promise_reaction(
    agent: &mut Agent,
    kind: PromiseReactionKind,
    handler: PromiseReactionHandler,
    capability: Option<lyng_js_env::PromiseCapabilityId>,
) -> lyng_js_env::PromiseReactionId {
    let script_or_module_referrer = agent
        .current_execution_context()
        .and_then(lyng_js_env::ExecutionContext::script_or_module_referrer);
    agent.alloc_promise_reaction(
        PromiseReactionRecord::new(kind, handler, capability)
            .with_script_or_module_referrer(script_or_module_referrer),
    )
}

/// Fulfills a pending promise and enqueues its fulfillment reactions.
///
/// # Errors
/// Returns a type-error completion when the promise record is missing or cannot be transitioned.
pub fn fulfill_promise(agent: &mut Agent, promise: ObjectRef, value: Value) -> Completion<()> {
    let Some(record) = agent.promise_record(promise).cloned() else {
        return Err(throw_type_error(agent));
    };
    if record.state() != PromiseState::Pending {
        return Ok(());
    }
    if !agent.set_promise_fulfilled(promise, value) {
        return Err(throw_type_error(agent));
    }
    let reactions = agent
        .take_promise_reactions(promise, PromiseReactionKind::Fulfill)
        .ok_or_else(|| throw_type_error(agent))?;
    let _ = agent.take_promise_reactions(promise, PromiseReactionKind::Reject);
    for reaction in reactions {
        let _ = agent.enqueue_job_with_payload(
            HostJobKind::Promise,
            ExecutableId::Builtin,
            RuntimeJobPayload::PromiseReaction {
                reaction,
                argument: value,
            },
            Some(record.realm()),
            Some("PromiseReaction".into()),
        );
    }
    Ok(())
}

/// Rejects a pending promise and enqueues its rejection reactions.
///
/// # Errors
/// Returns a type-error completion when the promise record is missing or cannot be transitioned.
pub fn reject_promise(agent: &mut Agent, promise: ObjectRef, reason: Value) -> Completion<()> {
    let Some(record) = agent.promise_record(promise).cloned() else {
        return Err(throw_type_error(agent));
    };
    if record.state() != PromiseState::Pending {
        return Ok(());
    }
    if !agent.set_promise_rejected(promise, reason) {
        return Err(throw_type_error(agent));
    }
    let _ = agent.set_promise_handled(promise, true);
    let reactions = agent
        .take_promise_reactions(promise, PromiseReactionKind::Reject)
        .ok_or_else(|| throw_type_error(agent))?;
    let _ = agent.take_promise_reactions(promise, PromiseReactionKind::Fulfill);
    for reaction in reactions {
        let _ = agent.enqueue_job_with_payload(
            HostJobKind::Promise,
            ExecutableId::Builtin,
            RuntimeJobPayload::PromiseReaction {
                reaction,
                argument: reason,
            },
            Some(record.realm()),
            Some("PromiseReaction".into()),
        );
    }
    Ok(())
}

pub fn enqueue_thenable_job(
    agent: &mut Agent,
    realm: lyng_js_types::RealmRef,
    promise: ObjectRef,
    thenable: ObjectRef,
    then: ObjectRef,
) {
    let _ = agent.enqueue_job_with_payload(
        HostJobKind::Promise,
        ExecutableId::Builtin,
        RuntimeJobPayload::PromiseThenableResolve {
            promise,
            thenable,
            then,
        },
        Some(realm),
        Some("PromiseResolveThenableJob".into()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use lyng_js_env::Runtime;
    use lyng_js_host::NoopHostHooks;

    #[test]
    fn settlement_helpers_flip_state_and_enqueue_reaction_jobs() {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let realm = agent.default_realm().unwrap();
        let promise = realm.global_object();
        let _ = agent.alloc_promise(promise, realm.id());
        let reaction = create_promise_reaction(
            agent,
            PromiseReactionKind::Fulfill,
            PromiseReactionHandler::Identity,
            None,
        );
        assert!(agent.push_promise_reaction(promise, PromiseReactionKind::Fulfill, reaction));

        fulfill_promise(agent, promise, Value::from_smi(17)).unwrap();

        assert_eq!(promise_state(agent, promise), Some(PromiseState::Fulfilled));
        let job = agent
            .dequeue_job(lyng_js_env::JobQueueKind::Promise)
            .expect("reaction job should be queued");
        assert_eq!(
            job.payload(),
            RuntimeJobPayload::PromiseReaction {
                reaction,
                argument: Value::from_smi(17),
            }
        );
    }
}
