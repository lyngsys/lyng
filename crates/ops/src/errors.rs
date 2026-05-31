use lyng_env::{Agent, Intrinsics, RealmRecord};
use lyng_gc::AllocationLifetime;
use lyng_objects::{
    AdaptiveProtoLoadDispatch, InternalMethodError, NoopAdaptiveProtoLoadDispatch,
    ObjectAllocation, ObjectFlags,
};
use lyng_types::{
    AbruptCompletion, Completion, ObjectRef, PropertyDescriptor, PropertyKey, RealmRef, Value,
};

/// Error families that can be materialized through the shared runtime error path.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    Error,
    Eval,
    Range,
    Reference,
    Syntax,
    Type,
    Uri,
}

fn current_realm(agent: &Agent) -> Option<RealmRecord> {
    let realm = agent
        .running_context()
        .map(lyng_env::RunningContext::realm)
        .or_else(|| agent.default_realm_id())?;
    agent.realm(realm)
}

const fn intrinsic_error_prototype(intrinsics: &Intrinsics, kind: ErrorKind) -> Option<ObjectRef> {
    match kind {
        ErrorKind::Error => intrinsics.error_prototype(),
        ErrorKind::Eval => intrinsics.eval_error_prototype(),
        ErrorKind::Range => intrinsics.range_error_prototype(),
        ErrorKind::Reference => intrinsics.reference_error_prototype(),
        ErrorKind::Syntax => intrinsics.syntax_error_prototype(),
        ErrorKind::Type => intrinsics.type_error_prototype(),
        ErrorKind::Uri => intrinsics.uri_error_prototype(),
    }
}

#[inline]
const fn fallback_throw_completion() -> AbruptCompletion {
    AbruptCompletion::throw(Value::undefined())
}

/// Resolves one intrinsic error prototype from the selected realm.
#[inline]
pub fn intrinsic_error_prototype_for_realm(
    agent: &Agent,
    realm: RealmRef,
    kind: ErrorKind,
) -> Option<ObjectRef> {
    intrinsic_error_prototype(&agent.realm(realm)?.intrinsics(), kind)
}

/// Allocates one bootstrapped error object with an explicit realm/prototype target.
///
/// # Errors
/// Returns an abrupt throw completion when realm lookup, shape lookup, or property definition fails.
#[inline]
pub fn create_error_object(
    agent: &mut Agent,
    realm: RealmRef,
    prototype: Option<ObjectRef>,
    message: Option<Value>,
    vm_dispatch: &mut dyn AdaptiveProtoLoadDispatch,
) -> Completion<ObjectRef> {
    let realm = agent.realm(realm).ok_or_else(fallback_throw_completion)?;
    let root_shape = realm.root_shape().ok_or_else(fallback_throw_completion)?;
    let object = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            ObjectAllocation::ordinary(root_shape)
                .with_prototype(prototype)
                .with_flags(ObjectFlags::extensible().union(ObjectFlags::ERROR_OBJECT)),
            AllocationLifetime::Default,
        )
    });

    if let Some(message) = message {
        let message_atom = agent.bootstrap_atoms().message();
        let mut descriptor = PropertyDescriptor::new();
        descriptor.set_value(message);
        descriptor.set_writable(true);
        descriptor.set_enumerable(false);
        descriptor.set_configurable(true);
        let defined = agent.define_own_property(
            object,
            PropertyKey::from_atom(message_atom),
            descriptor,
            AllocationLifetime::Default,
            vm_dispatch,
        );
        if !matches!(defined, Ok(true)) {
            return Err(fallback_throw_completion());
        }
    }

    Ok(object)
}

/// Allocates one bootstrapped intrinsic error object for the selected realm.
///
/// # Errors
/// Returns an abrupt throw completion when the intrinsic prototype or error object cannot be
/// materialized.
#[inline]
pub fn create_intrinsic_error_object(
    agent: &mut Agent,
    realm: RealmRef,
    kind: ErrorKind,
    message: Option<Value>,
) -> Completion<ObjectRef> {
    let prototype = intrinsic_error_prototype_for_realm(agent, realm, kind)
        .ok_or_else(fallback_throw_completion)?;
    // Newly-allocated error object cannot have AdaptiveProtoLoad watchpoints
    // registered on its initial shape, so the noop dispatcher is sound here.
    create_error_object(
        agent,
        realm,
        Some(prototype),
        message,
        &mut NoopAdaptiveProtoLoadDispatch,
    )
}

/// Returns the current realm-aware throw value for one common error kind.
pub fn error_value(agent: &mut Agent, kind: ErrorKind) -> Value {
    current_realm(agent)
        .map(|realm| realm.id())
        .map(|realm| create_intrinsic_error_object(agent, realm, kind, None))
        .transpose()
        .ok()
        .flatten()
        .map_or(Value::undefined(), Value::from_object_ref)
}

/// Returns the current realm-aware throw value for `TypeError` paths.
#[inline]
pub fn type_error_value(agent: &mut Agent) -> Value {
    error_value(agent, ErrorKind::Type)
}

/// Returns the current realm-aware throw value for `ReferenceError` paths.
#[inline]
pub fn reference_error_value(agent: &mut Agent) -> Value {
    error_value(agent, ErrorKind::Reference)
}

/// Returns the current realm-aware throw value for `SyntaxError` paths.
#[inline]
pub fn syntax_error_value(agent: &mut Agent) -> Value {
    error_value(agent, ErrorKind::Syntax)
}

/// Returns the current realm-aware throw value for `RangeError` paths.
#[inline]
pub fn range_error_value(agent: &mut Agent) -> Value {
    error_value(agent, ErrorKind::Range)
}

/// Constructs an abrupt `TypeError` completion using the current realm-aware throw value.
#[inline]
pub fn throw_type_error(agent: &mut Agent) -> AbruptCompletion {
    AbruptCompletion::throw(type_error_value(agent))
}

/// Constructs an abrupt `ReferenceError` completion using the current realm-aware throw value.
#[inline]
pub fn throw_reference_error(agent: &mut Agent) -> AbruptCompletion {
    AbruptCompletion::throw(reference_error_value(agent))
}

/// Constructs an abrupt `SyntaxError` completion using the current realm-aware throw value.
#[inline]
pub fn throw_syntax_error(agent: &mut Agent) -> AbruptCompletion {
    AbruptCompletion::throw(syntax_error_value(agent))
}

/// Constructs an abrupt `RangeError` completion using the current realm-aware throw value.
#[inline]
pub fn throw_range_error(agent: &mut Agent) -> AbruptCompletion {
    AbruptCompletion::throw(range_error_value(agent))
}

/// Maps the current internal-method error surface into the shared abrupt-completion channel.
#[inline]
pub(crate) fn internal_method_error(
    agent: &mut Agent,
    error: InternalMethodError,
) -> AbruptCompletion {
    match error {
        InternalMethodError::MissingObject
        | InternalMethodError::MissingClassRecord
        | InternalMethodError::CorruptObjectState
        | InternalMethodError::InvalidDescriptor
        | InternalMethodError::InvalidPrivateElement
        | InternalMethodError::InvalidPrivateBrand
        | InternalMethodError::DuplicatePrivateElement
        | InternalMethodError::ObjectNotExtensible
        | InternalMethodError::AccessorCallPending
        | InternalMethodError::MissingFunctionPayload
        | InternalMethodError::MissingNativeHandler
        | InternalMethodError::NotCallable
        | InternalMethodError::NotConstructible
        | InternalMethodError::RevokedProxy
        | InternalMethodError::BytecodeDispatchPending => throw_type_error(agent),
        InternalMethodError::ReferenceError => throw_reference_error(agent),
        InternalMethodError::RangeError => throw_range_error(agent),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lyng_env::Runtime;
    use lyng_gc::AllocationLifetime;
    use lyng_host::NoopHostHooks;

    fn install_test_error_prototypes(agent: &mut Agent) {
        let default_realm = agent.default_realm().expect("default realm should exist");
        let root_shape = default_realm
            .root_shape()
            .expect("default realm should expose a root shape");
        let (object_prototype, error_prototype, range_error_prototype, type_error_prototype) =
            agent.with_heap_and_objects(|heap, objects| {
                let mut mutator = heap.mutator();
                let object_prototype = objects.alloc_object(
                    &mut mutator,
                    ObjectAllocation::ordinary(root_shape),
                    AllocationLifetime::Default,
                );
                let error_prototype = objects.alloc_object(
                    &mut mutator,
                    ObjectAllocation::ordinary(root_shape).with_prototype(Some(object_prototype)),
                    AllocationLifetime::Default,
                );
                let range_error_prototype = objects.alloc_object(
                    &mut mutator,
                    ObjectAllocation::ordinary(root_shape).with_prototype(Some(error_prototype)),
                    AllocationLifetime::Default,
                );
                let type_error_prototype = objects.alloc_object(
                    &mut mutator,
                    ObjectAllocation::ordinary(root_shape).with_prototype(Some(error_prototype)),
                    AllocationLifetime::Default,
                );
                (
                    object_prototype,
                    error_prototype,
                    range_error_prototype,
                    type_error_prototype,
                )
            });
        let intrinsics = default_realm
            .intrinsics()
            .with_object_prototype(Some(object_prototype))
            .with_error_prototype(Some(error_prototype))
            .with_eval_error_prototype(Some(error_prototype))
            .with_range_error_prototype(Some(range_error_prototype))
            .with_reference_error_prototype(Some(error_prototype))
            .with_syntax_error_prototype(Some(error_prototype))
            .with_type_error_prototype(Some(type_error_prototype))
            .with_uri_error_prototype(Some(error_prototype));
        assert!(agent.set_realm_intrinsics(default_realm.id(), intrinsics));
    }

    #[test]
    fn type_error_allocates_bootstrapped_error_object() {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        install_test_error_prototypes(agent);
        let intrinsics = agent
            .default_realm()
            .expect("default realm should exist")
            .intrinsics();
        let thrown = throw_type_error(agent)
            .thrown_value()
            .expect("type error completion should carry a value");
        let object = thrown
            .as_object_ref()
            .expect("type error completion should throw an object");

        assert_eq!(
            agent
                .objects()
                .object_header(agent.heap().view(), object)
                .unwrap()
                .prototype(),
            intrinsics.type_error_prototype()
        );
    }

    #[test]
    fn range_error_allocates_distinct_error_objects() {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        install_test_error_prototypes(agent);

        let first = range_error_value(agent);
        let second = range_error_value(agent);

        assert_ne!(first, second);
        assert!(first.as_object_ref().is_some());
        assert!(second.as_object_ref().is_some());
    }

    /// Proves that `current_realm` (and therefore `throw_type_error`) follows
    /// the `running_context` scalar, not the default realm.
    ///
    /// We create a second realm with its own distinct `type_error_prototype`,
    /// set `agent.running_context` to point at that realm, and assert that the
    /// thrown error's prototype is the SECOND realm's type-error prototype — not
    /// the default realm's.
    #[test]
    fn current_realm_follows_running_context() {
        use lyng_env::RunningContext;

        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();

        // Set up the DEFAULT realm's error prototypes so the test has a valid
        // baseline to compare against.
        install_test_error_prototypes(agent);
        let default_realm = agent.default_realm().expect("default realm");
        let default_type_error_prototype = default_realm
            .intrinsics()
            .type_error_prototype()
            .expect("default realm type_error_prototype must be set");

        // --- Create a SECOND realm shell with its own root shape + prototypes ---
        let second_realm_id = agent.create_default_realm_shell(AllocationLifetime::Default);
        assert_ne!(second_realm_id, default_realm.id(), "should be a distinct realm");

        let second_realm = agent.realm(second_realm_id).expect("second realm record");
        let root_shape = second_realm.root_shape().expect("second realm root shape");

        let (second_type_error_prototype,) = agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            let second_type_error_prototype = objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape),
                AllocationLifetime::Default,
            );
            (second_type_error_prototype,)
        });

        // Wire the second realm's intrinsics with its unique type_error_prototype.
        let second_intrinsics = second_realm
            .intrinsics()
            .with_type_error_prototype(Some(second_type_error_prototype));
        assert!(agent.set_realm_intrinsics(second_realm_id, second_intrinsics));

        // --- Point running_context at the second realm ---
        agent.set_running_context(Some(RunningContext::new(second_realm_id, None)));

        // --- throw_type_error must use the SECOND realm ---
        let thrown = throw_type_error(agent)
            .thrown_value()
            .expect("type error should carry a value");
        let object = thrown
            .as_object_ref()
            .expect("type error should throw an object");

        let actual_proto = agent
            .objects()
            .object_header(agent.heap().view(), object)
            .unwrap()
            .prototype();

        assert_eq!(
            actual_proto,
            Some(second_type_error_prototype),
            "error prototype should come from the second realm (running_context), not the default"
        );
        assert_ne!(
            actual_proto,
            Some(default_type_error_prototype),
            "error prototype must NOT be the default realm's type_error_prototype"
        );
    }
}
