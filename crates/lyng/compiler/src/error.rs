use lyng_js_ast::{DeclId, ExprId, FunctionId, PatternId, StmtId};
use lyng_js_bytecode::BytecodeBuildError;
use lyng_js_common::AtomId;
use lyng_js_sema::{FunctionSemaId, SemanticBindingId};

pub type LoweringResult<T> = Result<T, LoweringError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoweringError {
    MissingUseSite {
        expr: ExprId,
    },
    MissingPrivateUse {
        expr: ExprId,
    },
    MissingResolvedBinding {
        expr: ExprId,
        name: AtomId,
    },
    MissingBinding {
        binding: SemanticBindingId,
    },
    MissingFunctionRecord {
        function: FunctionId,
    },
    MissingEnvironmentSlot {
        binding: SemanticBindingId,
    },
    MissingDeclarationBinding {
        name: AtomId,
    },
    AmbiguousDeclarationBinding {
        name: AtomId,
    },
    InvalidCapturedBindingDepth {
        binding: SemanticBindingId,
        function: Option<FunctionSemaId>,
    },
    RegisterOverflow {
        register: u16,
    },
    BytecodeBuild {
        error: BytecodeBuildError,
    },
    ConstantIndexOverflow {
        index: u32,
    },
    UnsupportedStatement {
        stmt: StmtId,
    },
    UnsupportedDeclaration {
        decl: DeclId,
    },
    UnsupportedExpression {
        expr: ExprId,
    },
    UnsupportedFunction {
        function: FunctionId,
    },
    UnsupportedPattern {
        pattern: PatternId,
    },
    UnsupportedDynamicName {
        expr: ExprId,
        name: AtomId,
    },
    UnsupportedNamedPropertyKey {
        expr: ExprId,
    },
}

impl From<BytecodeBuildError> for LoweringError {
    fn from(error: BytecodeBuildError) -> Self {
        Self::BytecodeBuild { error }
    }
}
