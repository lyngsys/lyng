//! Lowering scaffolding for the lyng-js Phase 4 compiler.
//!
//! Ownership: `lyng_js_compiler` owns lowering state, activation metadata, and
//! the installable compiled-unit bridge from AST/sema to `lyng_js_bytecode`.
//! It does not own runtime installation, feedback storage, or execution.

#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::return_self_not_must_use
)]

mod environment;
mod error;
mod module;
mod script;

use lyng_js_ast::FunctionId;
use lyng_js_bytecode::{
    ArgumentsMode, BytecodeFunctionId, BytecodeMarker, CompiledFunctionUnit, CompiledScriptUnit,
};
use lyng_js_common::SourceId;
use lyng_js_env::EnvironmentLayoutId;
use lyng_js_sema::{FunctionSemaId, ScopeId};
use lyng_js_types::CodeRef;

pub use environment::{
    derive_environment_layout_plan, install_environment_layout_plan, seed_global_var_names,
    EnvironmentLayoutPlan, EnvironmentLayoutPlanError, EnvironmentLayoutPlanResult,
    FunctionEnvironmentLayoutPlan, InstalledEnvironmentLayouts, ScopeEnvironmentLayoutPlan,
};
pub use error::{LoweringError, LoweringResult};
pub use module::{
    compile_module, CompiledModuleUnit, DynamicImportSite, IndirectExportEntry, LocalExportEntry,
    ModuleImportEntry, ModuleImportKind, RequestedModule, StarExportEntry,
};
pub use script::compile_script;

/// Minimal lowering context scaffold for Phase 4 compiler work.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LoweringContext {
    function: FunctionSemaId,
    next_register: u16,
    next_hidden_register: u16,
}

impl LoweringContext {
    #[inline]
    pub const fn new(function: FunctionSemaId) -> Self {
        Self {
            function,
            next_register: 0,
            next_hidden_register: 0,
        }
    }

    #[inline]
    pub const fn function(self) -> FunctionSemaId {
        self.function
    }

    #[inline]
    pub const fn next_register(self) -> u16 {
        self.next_register
    }

    #[inline]
    pub const fn next_hidden_register(self) -> u16 {
        self.next_hidden_register
    }

    #[inline]
    pub fn allocate_register(&mut self) -> u16 {
        let register = self.next_register;
        self.next_register = self
            .next_register
            .checked_add(1)
            .expect("register count should remain within u16 during scaffolding");
        register
    }

    #[inline]
    pub fn allocate_hidden_register(&mut self) -> u16 {
        let register = self.next_hidden_register;
        self.next_hidden_register = self
            .next_hidden_register
            .checked_add(1)
            .expect("hidden register count should remain within u16 during scaffolding");
        register
    }
}

/// Activation policy shell shared between sema-derived metadata and bytecode lowering.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ActivationMetadata {
    strict: bool,
    arguments_mode: ArgumentsMode,
    needs_environment: bool,
    environment_layout: Option<EnvironmentLayoutId>,
    has_rest_parameter: bool,
}

impl ActivationMetadata {
    #[inline]
    pub const fn new(
        strict: bool,
        arguments_mode: ArgumentsMode,
        needs_environment: bool,
        environment_layout: Option<EnvironmentLayoutId>,
        has_rest_parameter: bool,
    ) -> Self {
        Self {
            strict,
            arguments_mode,
            needs_environment,
            environment_layout,
            has_rest_parameter,
        }
    }

    #[inline]
    pub const fn strict(self) -> bool {
        self.strict
    }

    #[inline]
    pub const fn arguments_mode(self) -> ArgumentsMode {
        self.arguments_mode
    }

    #[inline]
    pub const fn needs_environment(self) -> bool {
        self.needs_environment
    }

    #[inline]
    pub const fn environment_layout(self) -> Option<EnvironmentLayoutId> {
        self.environment_layout
    }

    #[inline]
    pub const fn has_rest_parameter(self) -> bool {
        self.has_rest_parameter
    }
}

/// Minimal lowered-function scaffold linking AST, sema, bytecode, and environment layout data.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LoweredFunctionPlan {
    ast_function: FunctionId,
    sema_function: FunctionSemaId,
    scope_root: ScopeId,
    activation: ActivationMetadata,
    bytecode: BytecodeFunctionId,
}

impl LoweredFunctionPlan {
    #[inline]
    pub const fn new(
        ast_function: FunctionId,
        sema_function: FunctionSemaId,
        scope_root: ScopeId,
        activation: ActivationMetadata,
        bytecode: BytecodeFunctionId,
    ) -> Self {
        Self {
            ast_function,
            sema_function,
            scope_root,
            activation,
            bytecode,
        }
    }

    #[inline]
    pub const fn ast_function(self) -> FunctionId {
        self.ast_function
    }

    #[inline]
    pub const fn sema_function(self) -> FunctionSemaId {
        self.sema_function
    }

    #[inline]
    pub const fn scope_root(self) -> ScopeId {
        self.scope_root
    }

    #[inline]
    pub const fn activation(self) -> ActivationMetadata {
        self.activation
    }

    #[inline]
    pub const fn bytecode(self) -> BytecodeFunctionId {
        self.bytecode
    }
}

/// Minimal compiler-owned marker proving lowering sits between sema and bytecode, not inside the VM.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CompilerMarker {
    bytecode: BytecodeMarker,
    scope_root: ScopeId,
    function: FunctionSemaId,
}

impl CompilerMarker {
    #[inline]
    pub const fn new(
        bytecode: BytecodeMarker,
        scope_root: ScopeId,
        function: FunctionSemaId,
    ) -> Self {
        Self {
            bytecode,
            scope_root,
            function,
        }
    }

    #[inline]
    pub const fn bytecode(self) -> BytecodeMarker {
        self.bytecode
    }

    #[inline]
    pub const fn scope_root(self) -> ScopeId {
        self.scope_root
    }

    #[inline]
    pub const fn function(self) -> FunctionSemaId {
        self.function
    }
}

/// Placeholder script-compilation entrypoint reserved for later Phase 4 lowering work.
pub fn installable_script_unit(
    source: SourceId,
    unit: CompiledScriptUnit,
) -> (SourceId, CompiledScriptUnit) {
    (source, unit)
}

/// Minimal module-compilation entrypoint reserved for runtime installation wiring.
pub fn installable_module_unit(
    source: SourceId,
    unit: CompiledModuleUnit,
) -> (SourceId, CompiledModuleUnit) {
    (source, unit)
}

/// Placeholder function-compilation entrypoint reserved for later Phase 4 lowering work.
pub fn installable_function_unit(
    code: CodeRef,
    unit: CompiledFunctionUnit,
) -> (CodeRef, CompiledFunctionUnit) {
    (code, unit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lyng_js_common::SourceId;
    use lyng_js_types::FeedbackSlotId;
    use std::num::NonZeroU32;

    #[test]
    fn lowering_context_allocates_register_classes_independently() {
        let mut context = LoweringContext::new(FunctionSemaId::new(3));

        assert_eq!(context.allocate_register(), 0);
        assert_eq!(context.allocate_register(), 1);
        assert_eq!(context.allocate_hidden_register(), 0);
        assert_eq!(context.allocate_hidden_register(), 1);
        assert_eq!(context.function(), FunctionSemaId::new(3));
    }

    #[test]
    fn compiler_marker_round_trips_layering_inputs() {
        let marker = CompilerMarker::new(
            BytecodeMarker::new(
                SourceId::new(8),
                BytecodeFunctionId::new(NonZeroU32::new(2).unwrap()),
                FeedbackSlotId::new(NonZeroU32::new(5).unwrap()),
            ),
            ScopeId::new(4),
            FunctionSemaId::new(7),
        );

        assert_eq!(marker.scope_root(), ScopeId::new(4));
        assert_eq!(marker.function(), FunctionSemaId::new(7));
        assert_eq!(marker.bytecode().source(), SourceId::new(8));
    }
}
