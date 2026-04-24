use std::collections::{HashMap, HashSet};
use std::num::NonZeroU32;

use lyng_js_ast::{
    AssignOp, Ast, BinaryOp, CatchClause, Decl, DeclId, Expr, ExprId, ForInOfLeft, ForInit,
    FunctionId, FunctionKind, NodeList, Pattern, Property, PropertyKind, Stmt, StmtId, SwitchCase,
    VariableKind,
};
use lyng_js_bytecode::{
    ArgumentsMode, BytecodeBuilder, BytecodeEnvironmentBinding, BytecodeEnvironmentSlotFlags,
    BytecodeFunction, BytecodeFunctionFlags, BytecodeFunctionId, BytecodeFunctionKind, CallRange,
    CaptureDescriptor, CaptureSource, CompiledAtom, CompiledScriptUnit, ConstantValue,
    DeoptFrameValue, DeoptSnapshot, DeoptValueSource, DirectEvalLexicalScope, ExceptionHandler,
    ExceptionHandlerKind, FeedbackSiteKind, FeedbackSiteMetadata, Opcode, SafepointKind, ThisMode,
};
use lyng_js_common::{AtomId, AtomTable, Span, WellKnownAtom};
use lyng_js_sema::{
    DeclarationKind, FunctionSemaId, ProgramSemaView, ResolutionKind, ScopeId, ScopeKind,
    SemanticBindingId, StorageClass, UseSiteRecord,
};
use lyng_js_types::{
    js3_add_async_disposable_resource_builtin, js3_add_sync_disposable_resource_builtin,
    js3_bigint_builtin, js3_create_async_disposal_scope_builtin,
    js3_create_sync_disposal_scope_builtin, js3_dispose_scope_async_builtin,
    js3_dispose_scope_builtin, js3_eval_builtin, js3_internal_bind_function_private_env_builtin,
    js3_internal_construct_super_builtin, js3_internal_define_class_getter_property_builtin,
    js3_internal_define_class_setter_property_builtin, js3_internal_define_getter_property_builtin,
    js3_internal_define_method_property_builtin, js3_internal_define_private_field_builtin,
    js3_internal_define_setter_property_builtin, js3_internal_direct_eval_builtin,
    js3_internal_get_instance_field_key_builtin, js3_internal_get_template_object_builtin,
    js3_internal_install_instance_field_key_builtin, js3_internal_instance_of_builtin,
    js3_internal_object_literal_set_prototype_builtin, js3_internal_private_field_get_builtin,
    js3_internal_private_field_init_builtin, js3_internal_private_field_set_builtin,
    js3_internal_private_has_builtin, js3_internal_set_function_home_object_builtin,
    js3_internal_super_property_get_builtin, js3_internal_super_property_set_builtin,
    js3_internal_template_to_string_builtin, js3_object_set_prototype_of_builtin,
    js3_reference_error_builtin, js3_regexp_builtin, BuiltinFunctionId,
};

use crate::error::{LoweringError, LoweringResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProgramRootKind {
    Script,
    Module,
}

#[derive(Clone, Copy)]
pub(crate) struct ProgramSource<'a> {
    pub(crate) ast: &'a Ast,
    pub(crate) body: NodeList<StmtId>,
    pub(crate) span: Span,
    pub(crate) strict: bool,
    pub(crate) kind: ProgramRootKind,
}

mod activation;
mod bindings;
mod calls;
mod classes;
mod control;
mod emit;
mod expr;
mod function;
mod loops;
mod operators;
mod optional_chains;
mod property_exprs;
mod reference_targets;
mod state;
mod stmt;
mod templates;
mod variables;

#[cfg(test)]
mod tests;

pub use function::compile_script;

use activation::{
    build_function_activation_plan, collect_arguments_owners, parent_function_for,
    FunctionActivationPlan,
};
use reference_targets::{PreparedReferenceTarget, ReferenceUsage};
pub(crate) use state::CompilationState;
use state::{
    ActiveClassContext, CallBridgeRegisters, CompletionKind, CompletionRegisters, ControlTarget,
    ControlTargetKind, FinallyContext, FunctionCompiler, LoweredCallArguments, ParameterSource,
};
