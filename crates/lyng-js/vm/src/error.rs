use lyng_js_builtins::BuiltinBootstrapError;
use lyng_js_bytecode::{BytecodeFunctionId, ConstantValue, Opcode};
use lyng_js_host::HostError;
use lyng_js_types::{
    AbruptCompletion, CodeRef, EmbeddingFunctionId, EnvironmentRef, RealmRef,
    SuspendedExecutionRef, Value,
};

pub type VmResult<T> = Result<T, VmError>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModuleLoadError {
    Host(HostError),
    Vm(VmError),
    Parse,
    Sema,
    Lowering,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VmError {
    Abrupt(AbruptCompletion),
    Host(HostError),
    MissingEntry(BytecodeFunctionId),
    InvalidEnvironmentLayout(BytecodeFunctionId),
    MissingInstalledCode(CodeRef),
    MissingCodeRecord(CodeRef),
    MissingActiveFrame,
    MissingRealm(CodeRef),
    MissingRootShape(RealmRef),
    MissingEnvironment(EnvironmentRef),
    MissingEnvironmentLayout(CodeRef),
    MissingDefaultRealm,
    MissingModuleRecord,
    MissingModuleCode,
    MissingModuleEnvironment,
    MissingModuleResolution,
    AmbiguousModuleExport,
    UnsupportedModuleGraph,
    MissingRealmExtensionProvider,
    MissingEmbeddingFunction(EmbeddingFunctionId),
    BuiltinBootstrap(BuiltinBootstrapError),
    InstructionOutOfBounds {
        code: CodeRef,
        instruction_offset: u32,
    },
    /// Two `Wide` / `ExtraWide` prefix bytes in a row. The bytecode
    /// emitter never produces this — accepting it would silently mask
    /// a corrupted instruction stream. Surfaced by the prefix semantic
    /// bodies (`op_wide_semantic` / `op_extra_wide_semantic`) when
    /// `state.prefix` is already `Some(...)` on entry.
    DoublePrefix {
        code: CodeRef,
        instruction_offset: u32,
    },
    RegisterOutOfBounds {
        code: CodeRef,
        register: u16,
    },
    InvalidJumpTarget {
        code: CodeRef,
        instruction_offset: u32,
        target_offset: i64,
    },
    UnsupportedOpcode {
        code: CodeRef,
        instruction_offset: u32,
        opcode: Opcode,
    },
    MissingInlineCallRange {
        code: CodeRef,
        instruction_offset: u32,
        opcode: Opcode,
    },
    GeneratorYield {
        value: Value,
        suspended: SuspendedExecutionRef,
        raw_iterator_result: bool,
    },
    GeneratorStart {
        suspended: SuspendedExecutionRef,
    },
    AsyncSuspend,
    UnsupportedConstant {
        code: CodeRef,
        index: u32,
        constant: ConstantValue,
    },
    InvalidAtomConstant {
        code: CodeRef,
        index: u32,
        constant: ConstantValue,
    },
    ExpectedObject {
        code: CodeRef,
        instruction_offset: u32,
        value: Value,
    },
    UnsupportedPropertyKey {
        code: CodeRef,
        instruction_offset: u32,
        value: Value,
    },
}
