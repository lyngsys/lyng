//! Compiler lowering for the lyng-js bytecode pipeline.
//!
//! Ownership: `lyng_js_compiler` owns lowering state, activation metadata, and
//! the installable compiled-unit bridge from AST/sema to `lyng_js_bytecode`.
//! It does not own runtime installation, feedback storage, or execution.

#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    reason = "crate root exports the lowering API; engine-domain names and cheap accessors stay explicit for callers"
)]

pub mod dynamic;
mod environment;
mod error;
mod module;
mod script;

pub use environment::{
    derive_environment_layout_plan, install_environment_layout_plan, seed_global_var_names,
    EnvironmentLayoutPlan, EnvironmentLayoutPlanError, EnvironmentLayoutPlanResult,
    FunctionEnvironmentLayoutPlan, InstalledEnvironmentLayouts, ScopeEnvironmentLayoutPlan,
};
pub use error::{LoweringError, LoweringResult};
pub use module::{
    compile_module, CompiledModuleUnit, DynamicImportSite, IndirectExportEntry, LocalExportEntry,
    ModuleImportEntry, ModuleImportKind, ModuleRequestPhase, RequestedModule, StarExportEntry,
};
pub use script::compile_script;

#[inline]
pub(crate) fn checked_u32_index(index: usize) -> u32 {
    u32::try_from(index).expect("compiler table index should fit in u32")
}
