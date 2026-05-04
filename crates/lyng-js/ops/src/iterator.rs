use crate::{errors::throw_type_error, object, read};
use lyng_js_common::WellKnownAtom;
use lyng_js_env::Agent;
use lyng_js_gc::AllocationLifetime;
use lyng_js_types::{
    AbruptCompletion, Completion, ObjectRef, PropertyKey, RealmRef, Value, WellKnownSymbolId,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IteratorKind {
    Sync,
    Async,
    AsyncFromSync,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AsyncFromSyncState {
    None,
    Next { done: bool },
    Return,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DelegateYieldAwaitState {
    None,
    IteratorResult { return_completion: bool },
    Value { done: bool, return_completion: bool },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IteratorRecord {
    iterator: ObjectRef,
    next_method: Value,
    kind: IteratorKind,
    async_from_sync_state: AsyncFromSyncState,
    delegate_yield_await_state: DelegateYieldAwaitState,
    preserve_completion_on_close: bool,
    done: bool,
    delegate_started: bool,
}

impl IteratorRecord {
    #[inline]
    pub const fn new(iterator: ObjectRef, next_method: ObjectRef) -> Self {
        Self {
            iterator,
            next_method: Value::from_object_ref(next_method),
            kind: IteratorKind::Sync,
            async_from_sync_state: AsyncFromSyncState::None,
            delegate_yield_await_state: DelegateYieldAwaitState::None,
            preserve_completion_on_close: false,
            done: false,
            delegate_started: false,
        }
    }

    #[inline]
    const fn new_with_next_value(
        iterator: ObjectRef,
        next_method: Value,
        kind: IteratorKind,
    ) -> Self {
        Self {
            iterator,
            next_method,
            kind,
            async_from_sync_state: AsyncFromSyncState::None,
            delegate_yield_await_state: DelegateYieldAwaitState::None,
            preserve_completion_on_close: false,
            done: false,
            delegate_started: false,
        }
    }

    #[inline]
    pub const fn iterator(self) -> ObjectRef {
        self.iterator
    }

    #[inline]
    pub const fn new_async(iterator: ObjectRef, next_method: ObjectRef) -> Self {
        Self::new_with_next_value(
            iterator,
            Value::from_object_ref(next_method),
            IteratorKind::Async,
        )
    }

    #[inline]
    pub const fn new_async_from_sync(iterator: ObjectRef, next_method: ObjectRef) -> Self {
        Self::new_with_next_value(
            iterator,
            Value::from_object_ref(next_method),
            IteratorKind::AsyncFromSync,
        )
    }

    #[inline]
    pub const fn next_method(self) -> Value {
        self.next_method
    }

    #[inline]
    pub const fn kind(self) -> IteratorKind {
        self.kind
    }

    #[inline]
    pub const fn is_async(self) -> bool {
        !matches!(self.kind, IteratorKind::Sync)
    }

    #[inline]
    pub const fn is_async_from_sync(self) -> bool {
        matches!(self.kind, IteratorKind::AsyncFromSync)
    }

    #[inline]
    pub const fn async_from_sync_state(self) -> AsyncFromSyncState {
        self.async_from_sync_state
    }

    #[inline]
    pub fn set_async_from_sync_state(&mut self, state: AsyncFromSyncState) {
        self.async_from_sync_state = state;
    }

    #[inline]
    pub const fn delegate_yield_await_state(self) -> DelegateYieldAwaitState {
        self.delegate_yield_await_state
    }

    #[inline]
    pub fn set_delegate_yield_await_state(&mut self, state: DelegateYieldAwaitState) {
        self.delegate_yield_await_state = state;
    }

    #[inline]
    pub const fn preserve_completion_on_close(self) -> bool {
        self.preserve_completion_on_close
    }

    #[inline]
    pub fn set_preserve_completion_on_close(&mut self, preserve_completion_on_close: bool) {
        self.preserve_completion_on_close = preserve_completion_on_close;
    }

    #[inline]
    pub const fn done(self) -> bool {
        self.done
    }

    #[inline]
    pub fn set_done(&mut self, done: bool) {
        self.done = done;
    }

    #[inline]
    pub const fn delegate_started(self) -> bool {
        self.delegate_started
    }

    #[inline]
    pub fn set_delegate_started(&mut self, delegate_started: bool) {
        self.delegate_started = delegate_started;
    }
}

pub trait IteratorOpsContext {
    type Error;

    fn agent(&mut self) -> &mut Agent;

    fn realm(&self) -> RealmRef;

    fn abrupt(&mut self, completion: AbruptCompletion) -> Self::Error;

    fn type_error(&mut self) -> Self::Error;

    fn get_property_value(
        &mut self,
        receiver: Value,
        key: PropertyKey,
    ) -> Result<Value, Self::Error>;

    fn require_callable_object(&mut self, value: Value) -> Result<ObjectRef, Self::Error>;

    fn call_to_completion(
        &mut self,
        callee_object: ObjectRef,
        this_value: Value,
        arguments: &[Value],
    ) -> Result<Value, Self::Error>;
}

#[inline]
fn map_completion<Cx: IteratorOpsContext, T>(
    cx: &mut Cx,
    completion: Completion<T>,
) -> Result<T, Cx::Error> {
    completion.map_err(|completion| cx.abrupt(completion))
}

#[inline]
fn key_from_text(agent: &mut Agent, text: &str) -> PropertyKey {
    PropertyKey::from_atom(agent.atoms_mut().intern_collectible(text))
}

fn get_method<Cx: IteratorOpsContext>(
    cx: &mut Cx,
    receiver: Value,
    key: PropertyKey,
) -> Result<Option<ObjectRef>, Cx::Error> {
    let method = cx.get_property_value(receiver, key)?;
    if method.is_undefined() || method.is_null() {
        return Ok(None);
    }
    cx.require_callable_object(method).map(Some)
}

fn get_iterator_from_method<Cx: IteratorOpsContext>(
    cx: &mut Cx,
    receiver: Value,
    method: ObjectRef,
    kind: IteratorKind,
) -> Result<IteratorRecord, Cx::Error> {
    let iterator = cx.call_to_completion(method, receiver, &[])?;
    let iterator_object = iterator.as_object_ref().ok_or_else(|| cx.type_error())?;
    let next_key = {
        let agent = cx.agent();
        key_from_text(agent, "next")
    };
    let next_value = cx.get_property_value(Value::from_object_ref(iterator_object), next_key)?;
    Ok(match kind {
        IteratorKind::Sync => {
            IteratorRecord::new_with_next_value(iterator_object, next_value, IteratorKind::Sync)
        }
        IteratorKind::Async => {
            IteratorRecord::new_with_next_value(iterator_object, next_value, IteratorKind::Async)
        }
        IteratorKind::AsyncFromSync => IteratorRecord::new_with_next_value(
            iterator_object,
            next_value,
            IteratorKind::AsyncFromSync,
        ),
    })
}

pub fn create_iterator_result_object(
    agent: &mut Agent,
    realm: RealmRef,
    value: Value,
    done: bool,
) -> Completion<ObjectRef> {
    let realm = agent.realm(realm).ok_or_else(|| throw_type_error(agent))?;
    let root_shape = realm.root_shape().ok_or_else(|| throw_type_error(agent))?;
    let prototype = realm
        .intrinsics()
        .object_prototype()
        .ok_or_else(|| throw_type_error(agent))?;
    let object_ref = agent.with_heap_and_objects(|heap, objects| {
        let mut mutator = heap.mutator();
        objects.alloc_object(
            &mut mutator,
            lyng_js_objects::ObjectAllocation::ordinary(root_shape).with_prototype(Some(prototype)),
            AllocationLifetime::Default,
        )
    });
    let value_defined = object::ordinary_create_data_property(
        agent,
        object_ref,
        PropertyKey::from_atom(WellKnownAtom::value.id()),
        value,
        AllocationLifetime::Default,
    )?;
    if !value_defined {
        return Err(throw_type_error(agent));
    }
    let done_key = key_from_text(agent, "done");
    let done_defined = object::ordinary_create_data_property(
        agent,
        object_ref,
        done_key,
        Value::from_bool(done),
        AllocationLifetime::Default,
    )?;
    if !done_defined {
        return Err(throw_type_error(agent));
    }
    Ok(object_ref)
}

pub fn get_iterator<Cx: IteratorOpsContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<IteratorRecord, Cx::Error> {
    let iterator_symbol = {
        let agent = cx.agent();
        agent.well_known_symbol(WellKnownSymbolId::Iterator)
    }
    .ok_or_else(|| cx.type_error())?;
    let method = get_method(cx, value, PropertyKey::from_symbol(iterator_symbol))?
        .ok_or_else(|| cx.type_error())?;
    get_iterator_from_method(cx, value, method, IteratorKind::Sync)
}

pub fn get_async_iterator<Cx: IteratorOpsContext>(
    cx: &mut Cx,
    value: Value,
) -> Result<IteratorRecord, Cx::Error> {
    let async_iterator_symbol = {
        let agent = cx.agent();
        agent.well_known_symbol(WellKnownSymbolId::AsyncIterator)
    }
    .ok_or_else(|| cx.type_error())?;
    if let Some(method) = get_method(cx, value, PropertyKey::from_symbol(async_iterator_symbol))? {
        return get_iterator_from_method(cx, value, method, IteratorKind::Async);
    }

    let iterator_symbol = {
        let agent = cx.agent();
        agent.well_known_symbol(WellKnownSymbolId::Iterator)
    }
    .ok_or_else(|| cx.type_error())?;
    let method = get_method(cx, value, PropertyKey::from_symbol(iterator_symbol))?
        .ok_or_else(|| cx.type_error())?;
    get_iterator_from_method(cx, value, method, IteratorKind::AsyncFromSync)
}

pub fn iterator_next<Cx: IteratorOpsContext>(
    cx: &mut Cx,
    iterator_record: &IteratorRecord,
    value: Option<Value>,
) -> Result<ObjectRef, Cx::Error> {
    let mut arguments = [Value::undefined(); 1];
    let slice = if let Some(value) = value {
        arguments[0] = value;
        &arguments[..1]
    } else {
        &arguments[..0]
    };
    let next_method = cx.require_callable_object(iterator_record.next_method())?;
    let result = cx.call_to_completion(
        next_method,
        Value::from_object_ref(iterator_record.iterator()),
        slice,
    )?;
    result.as_object_ref().ok_or_else(|| cx.type_error())
}

pub fn iterator_complete<Cx: IteratorOpsContext>(
    cx: &mut Cx,
    iter_result: ObjectRef,
) -> Result<bool, Cx::Error> {
    let done_key = {
        let agent = cx.agent();
        key_from_text(agent, "done")
    };
    let done = cx.get_property_value(Value::from_object_ref(iter_result), done_key)?;
    let completion = {
        let agent = cx.agent();
        read::to_boolean_agent(agent, done)
    };
    map_completion(cx, completion)
}

pub fn iterator_value<Cx: IteratorOpsContext>(
    cx: &mut Cx,
    iter_result: ObjectRef,
) -> Result<Value, Cx::Error> {
    cx.get_property_value(
        Value::from_object_ref(iter_result),
        PropertyKey::from_atom(WellKnownAtom::value.id()),
    )
}

pub fn iterator_step<Cx: IteratorOpsContext>(
    cx: &mut Cx,
    iterator_record: &mut IteratorRecord,
) -> Result<Option<ObjectRef>, Cx::Error> {
    if iterator_record.done() {
        return Ok(None);
    }
    let result = iterator_next(cx, iterator_record, None)?;
    if iterator_complete(cx, result)? {
        iterator_record.set_done(true);
        return Ok(None);
    }
    Ok(Some(result))
}

pub fn iterator_close<Cx: IteratorOpsContext, T>(
    cx: &mut Cx,
    iterator_record: &mut IteratorRecord,
    completion: Completion<T>,
) -> Result<T, Cx::Error> {
    if iterator_record.done() {
        return completion.map_err(|abrupt| cx.abrupt(abrupt));
    }
    iterator_record.set_done(true);
    let preserve_completion = completion.is_err();
    let return_method = match get_method(
        cx,
        Value::from_object_ref(iterator_record.iterator()),
        PropertyKey::from_atom(WellKnownAtom::r#return.id()),
    ) {
        Ok(return_method) => return_method,
        Err(error) if preserve_completion => {
            return completion.map_err(|abrupt| cx.abrupt(abrupt));
        }
        Err(error) => return Err(error),
    };
    let Some(return_method) = return_method else {
        return completion.map_err(|abrupt| cx.abrupt(abrupt));
    };
    let inner_result = cx.call_to_completion(
        return_method,
        Value::from_object_ref(iterator_record.iterator()),
        &[],
    );
    if preserve_completion {
        return completion.map_err(|abrupt| cx.abrupt(abrupt));
    }
    let inner_result = inner_result?;
    if inner_result.as_object_ref().is_none() {
        return Err(cx.type_error());
    }
    completion.map_err(|abrupt| cx.abrupt(abrupt))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{errors, object};
    use lyng_js_env::Runtime;
    use lyng_js_host::NoopHostHooks;
    use lyng_js_objects::{
        FunctionConstructorFlags, FunctionObjectData, FunctionThisMode, InternalMethodResult,
        NativeCallRequest, NativeConstructRequest, NativeFunctionRegistry, ObjectAllocation,
        ObjectColdData, ObjectRuntime,
    };
    use lyng_js_types::BuiltinFunctionId;

    const ITERATOR_ENTRY: u32 = 7001;
    const NEXT_ENTRY: u32 = 7002;
    const RETURN_ENTRY: u32 = 7003;
    const THROWING_RETURN_ENTRY: u32 = 7004;
    const THROWING_NEXT_ENTRY: u32 = 7005;

    #[derive(Default)]
    struct IteratorRegistry {
        iterator_object: Option<ObjectRef>,
        next_results: Vec<ObjectRef>,
        next_calls: usize,
        return_calls: usize,
    }

    impl NativeFunctionRegistry for IteratorRegistry {
        fn call(
            &mut self,
            _runtime: &mut ObjectRuntime,
            _heap: &mut lyng_js_gc::PrimitiveMutator<'_>,
            request: NativeCallRequest<'_>,
        ) -> InternalMethodResult<Value> {
            let raw = request
                .entry()
                .builtin_entry()
                .expect("iterator tests install builtin entries")
                .get();
            match raw {
                ITERATOR_ENTRY => Ok(Value::from_object_ref(
                    self.iterator_object
                        .expect("iterator object should be installed"),
                )),
                NEXT_ENTRY => {
                    let result = self.next_results[self.next_calls];
                    self.next_calls += 1;
                    Ok(Value::from_object_ref(result))
                }
                RETURN_ENTRY => {
                    self.return_calls += 1;
                    Ok(Value::from_object_ref(self.next_results[0]))
                }
                THROWING_RETURN_ENTRY | THROWING_NEXT_ENTRY => {
                    Err(lyng_js_objects::InternalMethodError::NotCallable)
                }
                other => panic!("unexpected native entry {other}"),
            }
        }

        fn construct(
            &mut self,
            _runtime: &mut ObjectRuntime,
            _heap: &mut lyng_js_gc::PrimitiveMutator<'_>,
            _request: NativeConstructRequest<'_>,
        ) -> InternalMethodResult<ObjectRef> {
            Err(lyng_js_objects::InternalMethodError::NotConstructible)
        }
    }

    struct IteratorProbe<'a> {
        agent: &'a mut Agent,
        realm: RealmRef,
        registry: &'a mut IteratorRegistry,
    }

    impl IteratorOpsContext for IteratorProbe<'_> {
        type Error = AbruptCompletion;

        fn agent(&mut self) -> &mut Agent {
            self.agent
        }

        fn realm(&self) -> RealmRef {
            self.realm
        }

        fn abrupt(&mut self, completion: AbruptCompletion) -> Self::Error {
            completion
        }

        fn type_error(&mut self) -> Self::Error {
            errors::throw_type_error(self.agent)
        }

        fn get_property_value(
            &mut self,
            receiver: Value,
            key: PropertyKey,
        ) -> Result<Value, Self::Error> {
            let object = receiver
                .as_object_ref()
                .ok_or_else(|| errors::throw_type_error(self.agent))?;
            object::ordinary_get(self.agent, object, key)
        }

        fn require_callable_object(&mut self, value: Value) -> Result<ObjectRef, Self::Error> {
            let object = value
                .as_object_ref()
                .ok_or_else(|| errors::throw_type_error(self.agent))?;
            if self.agent.objects().function_data(object).is_some() {
                Ok(object)
            } else {
                Err(errors::throw_type_error(self.agent))
            }
        }

        fn call_to_completion(
            &mut self,
            callee_object: ObjectRef,
            this_value: Value,
            arguments: &[Value],
        ) -> Result<Value, Self::Error> {
            object::call(
                self.agent,
                callee_object,
                this_value,
                arguments,
                self.registry,
            )
        }
    }

    fn native_function(agent: &mut Agent, realm: RealmRef, entry: u32) -> ObjectRef {
        let realm_record = agent.realm(realm).expect("realm should exist");
        let root_shape = realm_record
            .root_shape()
            .expect("default realm should expose a root shape");
        let entry = BuiltinFunctionId::from_raw(entry).expect("builtin id");
        agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.alloc_object(
                &mut mutator,
                ObjectAllocation::function(root_shape).with_cold_data(ObjectColdData::Function(
                    FunctionObjectData::native(realm, realm_record.global_env(), entry)
                        .with_this_mode(FunctionThisMode::Global)
                        .with_constructor_flags(FunctionConstructorFlags::empty()),
                )),
                AllocationLifetime::Default,
            )
        })
    }

    fn ordinary_object(agent: &mut Agent, realm: RealmRef) -> ObjectRef {
        let realm_record = agent.realm(realm).expect("realm should exist");
        let root_shape = realm_record
            .root_shape()
            .expect("default realm should expose a root shape");
        agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape),
                AllocationLifetime::Default,
            )
        })
    }

    fn install_test_object_prototype(agent: &mut Agent, realm: RealmRef) -> ObjectRef {
        if let Some(object_prototype) = agent
            .realm(realm)
            .expect("realm should exist")
            .intrinsics()
            .object_prototype()
        {
            return object_prototype;
        }
        let object_prototype = ordinary_object(agent, realm);
        let updated = agent
            .realm(realm)
            .expect("realm should exist")
            .intrinsics()
            .with_object_prototype(Some(object_prototype));
        assert!(agent.set_realm_intrinsics(realm, updated));
        object_prototype
    }

    fn install_property(agent: &mut Agent, object_ref: ObjectRef, key: PropertyKey, value: Value) {
        assert!(object::ordinary_create_data_property(
            agent,
            object_ref,
            key,
            value,
            AllocationLifetime::Default,
        )
        .unwrap());
    }

    fn install_iterator_pair(
        agent: &mut Agent,
        realm: RealmRef,
        iterable: ObjectRef,
        iterator: ObjectRef,
        next: ObjectRef,
    ) {
        let iterator_symbol = agent
            .well_known_symbol(WellKnownSymbolId::Iterator)
            .expect("iterator symbol should exist");
        let iterator_method = native_function(agent, realm, ITERATOR_ENTRY);
        install_property(
            agent,
            iterable,
            PropertyKey::from_symbol(iterator_symbol),
            Value::from_object_ref(iterator_method),
        );
        let next_key = key_from_text(agent, "next");
        install_property(agent, iterator, next_key, Value::from_object_ref(next));
    }

    #[test]
    fn create_iterator_result_object_materializes_value_and_done() {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let realm = agent
            .default_realm()
            .expect("default realm should exist")
            .id();
        let _ = install_test_object_prototype(agent, realm);
        let object_ref =
            create_iterator_result_object(agent, realm, Value::from_smi(7), true).unwrap();

        assert_eq!(
            object::ordinary_get(
                agent,
                object_ref,
                PropertyKey::from_atom(WellKnownAtom::value.id())
            )
            .unwrap(),
            Value::from_smi(7)
        );
        let done_key = key_from_text(agent, "done");
        assert_eq!(
            object::ordinary_get(agent, object_ref, done_key).unwrap(),
            Value::from_bool(true)
        );
    }

    #[test]
    fn get_iterator_and_step_use_shared_record_state() {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let realm = agent
            .default_realm()
            .expect("default realm should exist")
            .id();
        let _ = install_test_object_prototype(agent, realm);
        let iterable = ordinary_object(agent, realm);
        let iterator = ordinary_object(agent, realm);
        let next = native_function(agent, realm, NEXT_ENTRY);
        let first =
            create_iterator_result_object(agent, realm, Value::from_smi(11), false).unwrap();
        let second =
            create_iterator_result_object(agent, realm, Value::from_smi(99), true).unwrap();
        let mut registry = IteratorRegistry {
            iterator_object: Some(iterator),
            next_results: vec![first, second],
            next_calls: 0,
            return_calls: 0,
        };
        install_iterator_pair(agent, realm, iterable, iterator, next);
        let mut probe = IteratorProbe {
            agent,
            realm,
            registry: &mut registry,
        };

        let mut record = get_iterator(&mut probe, Value::from_object_ref(iterable)).unwrap();
        let first = iterator_step(&mut probe, &mut record).unwrap().unwrap();

        assert_eq!(
            iterator_value(&mut probe, first).unwrap(),
            Value::from_smi(11)
        );
        assert_eq!(iterator_step(&mut probe, &mut record).unwrap(), None);
        assert_eq!(iterator_step(&mut probe, &mut record).unwrap(), None);
        assert_eq!(probe.registry.next_calls, 2);
        assert!(record.done());
    }

    #[test]
    fn iterator_close_calls_return_on_normal_completion() {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let realm = agent
            .default_realm()
            .expect("default realm should exist")
            .id();
        let _ = install_test_object_prototype(agent, realm);
        let iterable = ordinary_object(agent, realm);
        let iterator = ordinary_object(agent, realm);
        let next = native_function(agent, realm, NEXT_ENTRY);
        let return_method = native_function(agent, realm, RETURN_ENTRY);
        let result = create_iterator_result_object(agent, realm, Value::from_smi(0), true).unwrap();
        let mut registry = IteratorRegistry {
            iterator_object: Some(iterator),
            next_results: vec![result],
            next_calls: 0,
            return_calls: 0,
        };
        install_iterator_pair(agent, realm, iterable, iterator, next);
        install_property(
            agent,
            iterator,
            PropertyKey::from_atom(WellKnownAtom::r#return.id()),
            Value::from_object_ref(return_method),
        );
        let mut probe = IteratorProbe {
            agent,
            realm,
            registry: &mut registry,
        };
        let mut record = get_iterator(&mut probe, Value::from_object_ref(iterable)).unwrap();

        let closed = iterator_close(&mut probe, &mut record, Ok(Value::from_smi(5))).unwrap();

        assert_eq!(closed, Value::from_smi(5));
        assert_eq!(probe.registry.return_calls, 1);
        assert!(record.done());
    }

    #[test]
    fn iterator_close_preserves_prior_abrupt_completion_when_return_throws() {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let realm = agent
            .default_realm()
            .expect("default realm should exist")
            .id();
        let _ = install_test_object_prototype(agent, realm);
        let iterable = ordinary_object(agent, realm);
        let iterator = ordinary_object(agent, realm);
        let next = native_function(agent, realm, NEXT_ENTRY);
        let return_method = native_function(agent, realm, THROWING_RETURN_ENTRY);
        let result = create_iterator_result_object(agent, realm, Value::from_smi(0), true).unwrap();
        let mut registry = IteratorRegistry {
            iterator_object: Some(iterator),
            next_results: vec![result],
            next_calls: 0,
            return_calls: 0,
        };
        install_iterator_pair(agent, realm, iterable, iterator, next);
        install_property(
            agent,
            iterator,
            PropertyKey::from_atom(WellKnownAtom::r#return.id()),
            Value::from_object_ref(return_method),
        );
        let mut probe = IteratorProbe {
            agent,
            realm,
            registry: &mut registry,
        };
        let mut record = get_iterator(&mut probe, Value::from_object_ref(iterable)).unwrap();
        let abrupt = AbruptCompletion::throw(Value::from_smi(44));

        let result = iterator_close::<_, Value>(&mut probe, &mut record, Err(abrupt));

        assert_eq!(result, Err(abrupt));
        assert!(record.done());
    }

    #[test]
    fn iterator_close_preserves_prior_abrupt_completion_when_return_is_non_callable() {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let realm = agent
            .default_realm()
            .expect("default realm should exist")
            .id();
        let _ = install_test_object_prototype(agent, realm);
        let iterable = ordinary_object(agent, realm);
        let iterator = ordinary_object(agent, realm);
        let next = native_function(agent, realm, NEXT_ENTRY);
        let result = create_iterator_result_object(agent, realm, Value::from_smi(0), true).unwrap();
        let mut registry = IteratorRegistry {
            iterator_object: Some(iterator),
            next_results: vec![result],
            next_calls: 0,
            return_calls: 0,
        };
        install_iterator_pair(agent, realm, iterable, iterator, next);
        install_property(
            agent,
            iterator,
            PropertyKey::from_atom(WellKnownAtom::r#return.id()),
            Value::from_smi(0),
        );
        let mut probe = IteratorProbe {
            agent,
            realm,
            registry: &mut registry,
        };
        let mut record = get_iterator(&mut probe, Value::from_object_ref(iterable)).unwrap();
        let abrupt = AbruptCompletion::throw(Value::from_smi(44));

        let closed = iterator_close::<_, Value>(&mut probe, &mut record, Err(abrupt));

        assert_eq!(closed, Err(abrupt));
        assert_eq!(probe.registry.return_calls, 0);
        assert!(record.done());
    }

    #[test]
    fn iterator_close_skips_return_when_iterator_is_already_done() {
        let mut runtime = Runtime::new(NoopHostHooks);
        let agent = runtime.root_agent_mut();
        let realm = agent
            .default_realm()
            .expect("default realm should exist")
            .id();
        let _ = install_test_object_prototype(agent, realm);
        let iterator = ordinary_object(agent, realm);
        let next = native_function(agent, realm, NEXT_ENTRY);
        let return_method = native_function(agent, realm, RETURN_ENTRY);
        let result = create_iterator_result_object(agent, realm, Value::from_smi(0), true).unwrap();
        let mut registry = IteratorRegistry {
            iterator_object: Some(iterator),
            next_results: vec![result],
            next_calls: 0,
            return_calls: 0,
        };
        let next_key = key_from_text(agent, "next");
        install_property(agent, iterator, next_key, Value::from_object_ref(next));
        install_property(
            agent,
            iterator,
            PropertyKey::from_atom(WellKnownAtom::r#return.id()),
            Value::from_object_ref(return_method),
        );
        let mut probe = IteratorProbe {
            agent,
            realm,
            registry: &mut registry,
        };
        let mut record = IteratorRecord::new(iterator, next);
        record.set_done(true);

        let closed = iterator_close(&mut probe, &mut record, Ok(Value::from_smi(9))).unwrap();

        assert_eq!(closed, Value::from_smi(9));
        assert_eq!(probe.registry.return_calls, 0);
    }
}
