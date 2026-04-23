#![allow(
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    clippy::cast_possible_truncation,
    clippy::collapsible_else_if,
    clippy::collapsible_if,
    clippy::collapsible_match,
    clippy::doc_markdown,
    clippy::match_same_arms,
    clippy::module_name_repetitions,
    clippy::needless_range_loop,
    clippy::only_used_in_recursion,
    clippy::single_match,
    clippy::struct_excessive_bools,
    clippy::too_many_lines,
    clippy::trivially_copy_pass_by_ref,
    clippy::unnecessary_map_or,
    clippy::unused_self
)]

//! Semantic analysis pass for the lyng-js JavaScript engine.
//!
//! This crate walks the parsed AST and produces side-table-driven metadata:
//! scope tables, binding tables, function-level records, use-site resolution,
//! private-name analysis, and early-error diagnostics.
//!
//! **The AST is never mutated.** All output lives in side tables indexed by
//! typed IDs.

mod analyzer;
pub mod binding;
pub mod class_private_layout;
pub mod function_sema;
pub mod ids;
pub mod private_name;
pub mod private_use;
mod results;
pub mod scope;
pub mod use_site;

pub use binding::{BindingRecord, BindingTable, DeclarationKind, StorageClass};
pub use class_private_layout::{
    ClassPrivateElementKind, ClassPrivateElementRecord, ClassPrivateLayoutRecord,
    ClassPrivateLayoutTable,
};
pub use function_sema::{FunctionSemaRecord, FunctionSemaTable};
pub use ids::{FunctionSemaId, PrivateNameId, ScopeId, SemanticBindingId, UseSiteId};
pub use private_name::{PrivateNameRecord, PrivateNameTable};
pub use private_use::{PrivateUseRecord, PrivateUseTable};
pub use results::{
    analyze_direct_eval_script, analyze_module, analyze_script, DirectEvalScriptAnalysisOptions,
    ModuleSema, ProgramSemaView, ScriptSema,
};
pub use scope::{ScopeKind, ScopeRecord, ScopeTable};
pub use use_site::{ResolutionKind, UseSiteRecord, UseSiteTable};

#[cfg(test)]
mod tests;
