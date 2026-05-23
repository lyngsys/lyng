use super::{
    FunctionEntryIdentity, FunctionObjectData, InternalMethodError, InternalMethodResult,
    NativeCallRequest, NativeConstructRequest, NativeFunctionRegistry, ObjectColdData, ObjectKind,
    ObjectRef, ObjectRuntime, PrimitiveMutator, Value,
};

impl ObjectRuntime {
    pub fn function_data(&self, id: ObjectRef) -> Option<&FunctionObjectData> {
        match &self.object_metadata(id)?.cold {
            ObjectColdData::Function(data) => Some(data),
            ObjectColdData::Ordinary(_) | ObjectColdData::Proxy(_) => None,
        }
    }

    pub fn set_function_home_object(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        home_object: Option<ObjectRef>,
    ) -> bool {
        let Some(metadata) = self.object_metadata_mut(id) else {
            return false;
        };
        let ObjectColdData::Function(data) = &mut metadata.cold else {
            return false;
        };
        *data = data.clone().with_home_object(home_object);
        data.gc_payload()
            .is_none_or(|payload| heap.set_function_payload_home_object(payload, home_object))
    }

    pub fn set_function_environment(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        environment: Option<lyng_js_types::EnvironmentRef>,
    ) -> bool {
        let Some(metadata) = self.object_metadata_mut(id) else {
            return false;
        };
        let ObjectColdData::Function(data) = &mut metadata.cold else {
            return false;
        };
        *data = data.clone().with_environment(environment);
        data.gc_payload()
            .is_none_or(|payload| heap.set_function_payload_environment(payload, environment))
    }

    pub fn set_function_private_env(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        private_env: Option<lyng_js_types::EnvironmentRef>,
    ) -> bool {
        let Some(metadata) = self.object_metadata_mut(id) else {
            return false;
        };
        let ObjectColdData::Function(data) = &mut metadata.cold else {
            return false;
        };
        *data = data.clone().with_private_env(private_env);
        data.gc_payload()
            .is_none_or(|payload| heap.set_function_payload_private_env(payload, private_env))
    }

    /// Invokes one callable function object through the substrate-native registry.
    ///
    /// # Errors
    /// Returns an error when the callee is not callable, lacks required metadata, or dispatch is
    /// not yet implemented for the selected function entry.
    pub fn call(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        this_value: Value,
        arguments: &[Value],
        registry: &mut dyn NativeFunctionRegistry,
    ) -> InternalMethodResult<Value> {
        if self.is_proxy_object(id) {
            if self.is_proxy_revoked(id) == Some(true) {
                return Err(InternalMethodError::RevokedProxy);
            }
            if !self.is_callable(id) {
                return Err(InternalMethodError::NotCallable);
            }
            let target = self
                .proxy_target(id)
                .ok_or(InternalMethodError::MissingObject)?;
            return self.call(heap, target, this_value, arguments, registry);
        }
        let data = self.require_callable_function(id)?.clone();
        match data
            .entry()
            .ok_or(InternalMethodError::MissingFunctionPayload)?
        {
            FunctionEntryIdentity::Native(entry) => registry.call(
                self,
                heap,
                NativeCallRequest::new(
                    id,
                    this_value,
                    arguments,
                    data.realm()
                        .ok_or(InternalMethodError::MissingFunctionPayload)?,
                    data.environment()
                        .ok_or(InternalMethodError::MissingFunctionPayload)?,
                    data.private_env(),
                    data.this_mode(),
                    data.home_object(),
                    data.constructor_flags(),
                    data.kind_flags(),
                    entry,
                ),
            ),
            FunctionEntryIdentity::Bound => {
                let bound = self.bound_function_record(heap.view(), id)?;
                let mut combined_arguments = Vec::new();
                if let Some(bound_arguments) = bound.arguments() {
                    combined_arguments.extend_from_slice(
                        heap.view()
                            .object_slots(bound_arguments)
                            .ok_or(InternalMethodError::MissingFunctionPayload)?,
                    );
                }
                combined_arguments.extend_from_slice(arguments);
                self.call(
                    heap,
                    bound.target(),
                    bound.this_value(),
                    &combined_arguments,
                    registry,
                )
            }
            FunctionEntryIdentity::Bytecode(_) => Err(InternalMethodError::BytecodeDispatchPending),
        }
    }

    /// Invokes one constructible function object through the substrate-native registry.
    ///
    /// # Errors
    /// Returns an error when the callee is not constructible, lacks required metadata, or dispatch
    /// is not yet implemented for the selected function entry.
    pub fn construct(
        &mut self,
        heap: &mut PrimitiveMutator<'_>,
        id: ObjectRef,
        arguments: &[Value],
        new_target: Option<ObjectRef>,
        registry: &mut dyn NativeFunctionRegistry,
    ) -> InternalMethodResult<ObjectRef> {
        if self.is_proxy_object(id) {
            if self.is_proxy_revoked(id) == Some(true) {
                return Err(InternalMethodError::RevokedProxy);
            }
            if !self.is_constructor(id) {
                return Err(InternalMethodError::NotConstructible);
            }
            let target = self
                .proxy_target(id)
                .ok_or(InternalMethodError::MissingObject)?;
            let forwarded_new_target = match new_target {
                Some(candidate) if candidate == id => Some(target),
                other => other,
            };
            return self.construct(heap, target, arguments, forwarded_new_target, registry);
        }
        let data = self.require_constructible_function(id)?.clone();
        let new_target = new_target.unwrap_or(id);
        match data
            .entry()
            .ok_or(InternalMethodError::MissingFunctionPayload)?
        {
            FunctionEntryIdentity::Native(entry) => registry.construct(
                self,
                heap,
                NativeConstructRequest::new(
                    id,
                    new_target,
                    arguments,
                    data.realm()
                        .ok_or(InternalMethodError::MissingFunctionPayload)?,
                    data.environment()
                        .ok_or(InternalMethodError::MissingFunctionPayload)?,
                    data.private_env(),
                    data.this_mode(),
                    data.home_object(),
                    data.constructor_flags(),
                    data.kind_flags(),
                    entry,
                ),
            ),
            FunctionEntryIdentity::Bound => {
                let bound = self.bound_function_record(heap.view(), id)?;
                let mut combined_arguments = Vec::new();
                if let Some(bound_arguments) = bound.arguments() {
                    combined_arguments.extend_from_slice(
                        heap.view()
                            .object_slots(bound_arguments)
                            .ok_or(InternalMethodError::MissingFunctionPayload)?,
                    );
                }
                combined_arguments.extend_from_slice(arguments);
                let forwarded_new_target = if new_target == id {
                    bound.target()
                } else {
                    new_target
                };
                self.construct(
                    heap,
                    bound.target(),
                    &combined_arguments,
                    Some(forwarded_new_target),
                    registry,
                )
            }
            FunctionEntryIdentity::Bytecode(_) => Err(InternalMethodError::BytecodeDispatchPending),
        }
    }

    pub(crate) fn require_object_kind(&self, id: ObjectRef) -> InternalMethodResult<ObjectKind> {
        self.object_metadata(id)
            .map(|metadata| metadata.kind)
            .ok_or(InternalMethodError::MissingObject)
    }

    fn require_function_data(&self, id: ObjectRef) -> InternalMethodResult<&FunctionObjectData> {
        match self.require_object_kind(id)? {
            ObjectKind::Ordinary | ObjectKind::Proxy => Err(InternalMethodError::NotCallable),
            ObjectKind::Function => self
                .function_data(id)
                .ok_or(InternalMethodError::MissingFunctionPayload),
        }
    }

    fn require_callable_function(
        &self,
        id: ObjectRef,
    ) -> InternalMethodResult<&FunctionObjectData> {
        let data = self.require_function_data(id)?;
        if data.kind_flags().is_class_constructor() {
            return Err(InternalMethodError::NotCallable);
        }
        Ok(data)
    }

    fn require_constructible_function(
        &self,
        id: ObjectRef,
    ) -> InternalMethodResult<&FunctionObjectData> {
        let data = self.require_function_data(id)?;
        if !data.is_constructible() {
            return Err(InternalMethodError::NotConstructible);
        }
        Ok(data)
    }

    fn bound_function_record(
        &self,
        heap: lyng_js_gc::PrimitiveHeapView<'_>,
        id: ObjectRef,
    ) -> InternalMethodResult<lyng_js_gc::RuntimeBoundFunctionRecord> {
        let payload = self
            .require_function_data(id)?
            .gc_payload()
            .ok_or(InternalMethodError::MissingFunctionPayload)?;
        heap.function_payload(payload)
            .and_then(lyng_js_gc::RuntimeFunctionRecord::bound)
            .ok_or(InternalMethodError::MissingFunctionPayload)
    }
}
