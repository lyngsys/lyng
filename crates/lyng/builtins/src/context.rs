use crate::BuiltinPropertyDescriptor;
use lyng_js_types::{
    BuiltinFunctionId, CodeRef, Completion, EnvironmentRef, ObjectRef, RealmRef, Value,
};

/// Return type for one builtin call body.
pub type BuiltinResult = Completion<Value>;

/// Dynamic function body flavor used by the `Function` constructor family.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DynamicFunctionKind {
    Ordinary,
    Generator,
    Async,
    AsyncGenerator,
}

/// Native handler signature for one builtin entrypoint.
pub type BuiltinHandler<Cx> = fn(
    &mut Cx,
    this_value: Value,
    arguments: &[Value],
    new_target: Option<ObjectRef>,
) -> BuiltinResult;

/// One builtin invocation shape as seen by a native entrypoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuiltinInvocation<'a> {
    this_value: Value,
    arguments: &'a [Value],
    new_target: Option<ObjectRef>,
}

impl<'a> BuiltinInvocation<'a> {
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

/// Request payload for builtin function-object allocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuiltinFunctionAllocation<'a> {
    entry: BuiltinFunctionId,
    display_name: &'a str,
    length: u16,
    constructible: bool,
    prototype: Option<ObjectRef>,
    environment: EnvironmentRef,
}

impl<'a> BuiltinFunctionAllocation<'a> {
    #[inline]
    pub const fn new(
        entry: BuiltinFunctionId,
        display_name: &'a str,
        length: u16,
        constructible: bool,
        prototype: Option<ObjectRef>,
        environment: EnvironmentRef,
    ) -> Self {
        Self {
            entry,
            display_name,
            length,
            constructible,
            prototype,
            environment,
        }
    }

    #[inline]
    pub const fn entry(self) -> BuiltinFunctionId {
        self.entry
    }

    #[inline]
    pub const fn display_name(self) -> &'a str {
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
    pub const fn prototype(self) -> Option<ObjectRef> {
        self.prototype
    }

    #[inline]
    pub const fn environment(self) -> EnvironmentRef {
        self.environment
    }
}

/// Request payload for dynamic-function compilation used by `Function` constructor paths.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DynamicFunctionPlan<'a> {
    kind: DynamicFunctionKind,
    parameters_source: &'a str,
    body_source: &'a str,
    strict_caller: bool,
}

impl<'a> DynamicFunctionPlan<'a> {
    #[inline]
    pub const fn new(
        kind: DynamicFunctionKind,
        parameters_source: &'a str,
        body_source: &'a str,
        strict_caller: bool,
    ) -> Self {
        Self {
            kind,
            parameters_source,
            body_source,
            strict_caller,
        }
    }

    #[inline]
    pub const fn kind(self) -> DynamicFunctionKind {
        self.kind
    }

    #[inline]
    pub const fn parameters_source(self) -> &'a str {
        self.parameters_source
    }

    #[inline]
    pub const fn body_source(self) -> &'a str {
        self.body_source
    }

    #[inline]
    pub const fn strict_caller(self) -> bool {
        self.strict_caller
    }
}

/// Narrow execution-layer API exposed to builtin implementations.
pub trait BuiltinCallContext {
    type Error;

    fn current_realm(&self) -> RealmRef;

    fn lexical_environment(&self) -> EnvironmentRef;

    fn variable_environment(&self) -> EnvironmentRef;

    /// Allocates one ordinary object for bootstrap or builtin helper paths.
    ///
    /// # Errors
    /// Returns an error when allocation cannot complete.
    fn allocate_ordinary_object(
        &mut self,
        prototype: Option<ObjectRef>,
    ) -> Result<ObjectRef, Self::Error>;

    /// Allocates one builtin function object with the planned metadata.
    ///
    /// # Errors
    /// Returns an error when builtin function allocation cannot complete.
    fn allocate_builtin_function(
        &mut self,
        allocation: BuiltinFunctionAllocation<'_>,
    ) -> Result<ObjectRef, Self::Error>;

    /// Installs one builtin property descriptor.
    ///
    /// # Errors
    /// Returns an error when descriptor installation cannot complete.
    fn define_builtin_property(
        &mut self,
        target: ObjectRef,
        descriptor: BuiltinPropertyDescriptor,
    ) -> Result<bool, Self::Error>;

    /// Compiles one dynamic function source payload.
    ///
    /// # Errors
    /// Returns an error when dynamic compilation is rejected or fails.
    fn compile_dynamic_function(
        &mut self,
        plan: DynamicFunctionPlan<'_>,
    ) -> Result<CodeRef, Self::Error>;
}
