use crate::error::VmResult;
use crate::{FrameRecord, Vm, VmError};
use lyng_js_builtins::BootstrapArtifacts;
use lyng_js_env::Agent;
use lyng_js_gc::AllocationLifetime;
use lyng_js_host::HostHooks;
use lyng_js_objects::{NativeFunctionRegistry, ObjectAllocation};
use lyng_js_ops::errors;
use lyng_js_types::{
    BuiltinFunctionId, EmbeddingFunctionId, ObjectRef, PropertyDescriptor, PropertyKey, RealmRef,
    Value,
};
use std::sync::Arc;

pub type SharedRealmExtensionProvider = Arc<dyn RealmExtensionProvider>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EmbeddingFunctionMetadata {
    display_name: &'static str,
    length: u16,
    constructible: bool,
    has_prototype_property: bool,
}

impl EmbeddingFunctionMetadata {
    #[inline]
    pub const fn new(
        display_name: &'static str,
        length: u16,
        constructible: bool,
        has_prototype_property: bool,
    ) -> Self {
        Self {
            display_name,
            length,
            constructible,
            has_prototype_property,
        }
    }

    #[inline]
    pub const fn display_name(self) -> &'static str {
        self.display_name
    }

    #[inline]
    pub const fn length(self) -> u16 {
        self.length
    }

    #[inline]
    pub const fn constructible(self) -> bool {
        self.constructible
    }

    #[inline]
    pub const fn has_prototype_property(self) -> bool {
        self.has_prototype_property
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EmbeddingInvocation<'a> {
    this_value: Value,
    arguments: &'a [Value],
    new_target: Option<ObjectRef>,
}

impl<'a> EmbeddingInvocation<'a> {
    #[inline]
    pub const fn new(
        this_value: Value,
        arguments: &'a [Value],
        new_target: Option<ObjectRef>,
    ) -> Self {
        Self {
            this_value,
            arguments,
            new_target,
        }
    }

    #[inline]
    pub const fn this_value(self) -> Value {
        self.this_value
    }

    #[inline]
    pub const fn arguments(self) -> &'a [Value] {
        self.arguments
    }

    #[inline]
    pub const fn new_target(self) -> Option<ObjectRef> {
        self.new_target
    }
}

pub struct RealmExtensionInstallation<'a> {
    vm: &'a mut Vm,
    agent: &'a mut Agent,
    provider: &'a SharedRealmExtensionProvider,
    artifacts: BootstrapArtifacts,
}

impl<'a> RealmExtensionInstallation<'a> {
    #[inline]
    pub(crate) const fn new(
        vm: &'a mut Vm,
        agent: &'a mut Agent,
        provider: &'a SharedRealmExtensionProvider,
        artifacts: BootstrapArtifacts,
    ) -> Self {
        Self {
            vm,
            agent,
            provider,
            artifacts,
        }
    }

    #[inline]
    pub const fn artifacts(&self) -> BootstrapArtifacts {
        self.artifacts
    }

    #[inline]
    pub const fn realm(&self) -> RealmRef {
        self.artifacts.realm()
    }

    #[inline]
    pub const fn global_object(&self) -> ObjectRef {
        self.artifacts.global_object()
    }

    #[inline]
    pub fn agent(&mut self) -> &mut Agent {
        self.agent
    }

    pub fn allocate_ordinary_object(
        &mut self,
        prototype: Option<ObjectRef>,
    ) -> Result<ObjectRef, VmError> {
        let root_shape = self
            .agent
            .realm(self.realm())
            .and_then(|record| record.root_shape())
            .ok_or(VmError::MissingRootShape(self.realm()))?;
        Ok(self.agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.alloc_object(
                &mut mutator,
                ObjectAllocation::ordinary(root_shape).with_prototype(prototype),
                AllocationLifetime::Default,
            )
        }))
    }

    pub fn allocate_function(&mut self, entry: EmbeddingFunctionId) -> Result<ObjectRef, VmError> {
        self.vm
            .allocate_embedding_function_object(self.agent, self.realm(), entry, self.provider)
    }

    pub fn builtin_constant(&mut self, entry: BuiltinFunctionId) -> Result<Value, VmError> {
        self.vm
            .builtin_constant(self.agent, self.realm(), entry)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(self.agent)))
    }

    pub fn define_property(
        &mut self,
        target: ObjectRef,
        key: PropertyKey,
        descriptor: PropertyDescriptor,
    ) -> Result<(), VmError> {
        let defined = self.agent.with_heap_and_objects(|heap, objects| {
            let mut mutator = heap.mutator();
            objects.define_own_property(
                &mut mutator,
                target,
                key,
                descriptor,
                AllocationLifetime::Default,
            )
        });
        if matches!(defined, Ok(true)) {
            return Ok(());
        }
        Err(VmError::Abrupt(errors::throw_type_error(self.agent)))
    }

    pub fn define_data_property(
        &mut self,
        target: ObjectRef,
        key: PropertyKey,
        value: Value,
        writable: bool,
        enumerable: bool,
        configurable: bool,
    ) -> Result<(), VmError> {
        let mut descriptor = PropertyDescriptor::new();
        descriptor.set_value(value);
        descriptor.set_writable(writable);
        descriptor.set_enumerable(enumerable);
        descriptor.set_configurable(configurable);
        self.define_property(target, key, descriptor)
    }

    pub fn define_function_property(
        &mut self,
        target: ObjectRef,
        key: PropertyKey,
        entry: EmbeddingFunctionId,
        writable: bool,
        enumerable: bool,
        configurable: bool,
    ) -> Result<ObjectRef, VmError> {
        let function = self.allocate_function(entry)?;
        self.define_data_property(
            target,
            key,
            Value::from_object_ref(function),
            writable,
            enumerable,
            configurable,
        )?;
        Ok(function)
    }
}

pub struct EmbeddingFunctionContext<'a> {
    vm: &'a mut Vm,
    agent: &'a mut Agent,
    host: &'a dyn HostHooks,
    registry: &'a mut dyn NativeFunctionRegistry,
    provider: &'a SharedRealmExtensionProvider,
    caller_frame: FrameRecord,
    callee_object: ObjectRef,
}

impl<'a> EmbeddingFunctionContext<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) const fn new(
        vm: &'a mut Vm,
        agent: &'a mut Agent,
        host: &'a dyn HostHooks,
        registry: &'a mut dyn NativeFunctionRegistry,
        provider: &'a SharedRealmExtensionProvider,
        caller_frame: FrameRecord,
        callee_object: ObjectRef,
    ) -> Self {
        Self {
            vm,
            agent,
            host,
            registry,
            provider,
            caller_frame,
            callee_object,
        }
    }

    #[inline]
    pub fn agent(&mut self) -> &mut Agent {
        self.agent
    }

    #[inline]
    pub const fn callee_object(&self) -> ObjectRef {
        self.callee_object
    }

    #[inline]
    pub const fn caller_realm(&self) -> RealmRef {
        self.caller_frame.realm()
    }

    pub fn function_realm(&self) -> RealmRef {
        self.agent
            .objects()
            .function_data(self.callee_object)
            .and_then(|data| data.realm())
            .unwrap_or(self.caller_frame.realm())
    }

    pub fn evaluate_script_in_realm(
        &mut self,
        realm: RealmRef,
        source_text: &str,
    ) -> Result<Value, VmError> {
        self.vm
            .evaluate_script_source(self.agent, self.host, self.registry, realm, source_text)
    }

    pub fn value_to_string_text(&mut self, value: Value) -> Result<String, VmError> {
        self.vm.value_to_string_text(self.agent, value)
    }

    pub fn create_embedding_realm(&mut self) -> Result<BootstrapArtifacts, VmError> {
        self.vm.create_embedding_realm(self.agent, self.provider)
    }
}

pub trait RealmExtensionProvider: Send + Sync {
    fn embedding_function_metadata(
        &self,
        entry: EmbeddingFunctionId,
    ) -> Option<EmbeddingFunctionMetadata>;

    fn install_realm_extensions(
        &self,
        installation: &mut RealmExtensionInstallation<'_>,
    ) -> VmResult<()>;

    fn call_embedding_function(
        &self,
        context: &mut EmbeddingFunctionContext<'_>,
        entry: EmbeddingFunctionId,
        invocation: EmbeddingInvocation<'_>,
    ) -> VmResult<Value>;

    fn construct_embedding_function(
        &self,
        context: &mut EmbeddingFunctionContext<'_>,
        _entry: EmbeddingFunctionId,
        _invocation: EmbeddingInvocation<'_>,
        _new_target: ObjectRef,
    ) -> VmResult<ObjectRef> {
        Err(VmError::Abrupt(errors::throw_type_error(context.agent())))
    }
}
