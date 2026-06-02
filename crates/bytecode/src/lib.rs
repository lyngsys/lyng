//! Bytecode templates and metadata for the lyng execution layer.
//!
//! Ownership: `lyng_bytecode` owns immutable bytecode templates, instruction records,
//! compiled-unit containers, metadata shells, builders, and disassembly helpers shared by
//! the compiler and VM. It does not own lowering policy, runtime installation, or execution.

#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    reason = "crate root exports stable bytecode API types; domain prefixes and cheap accessors stay explicit for callers"
)]

mod builder;
mod decoder;
mod disassembler;
mod function;
mod ids;
mod instruction;
mod metadata;
mod opcode;

pub use builder::{
    BytecodeBuildError, BytecodeBuildResult, BytecodeBuilder, BytecodeLimitKind,
    BytecodeOperandKind,
};
pub use decoder::{
    DecodeError, DecodedInstructionStream, InvalidInstructionWord, decode_instruction_bytes,
    decode_instruction_stream, decode_instruction_word,
};
pub use disassembler::{disassemble, disassemble_instruction};
pub use function::{
    BytecodeEnvironmentBinding, BytecodeEnvironmentSlotFlags, BytecodeFunction,
    BytecodeFunctionHeader, CompiledAtom, CompiledFunctionUnit, CompiledScriptUnit,
    GlobalLexicalBindingPlan, GlobalScriptInstantiationPlan, InstructionStream,
};
pub use ids::{BytecodeFunctionId, EnvironmentLayoutRef};
pub use instruction::{INSTRUCTION_WIDTH, Instruction};
pub use metadata::{
    ArgumentsMode, BytecodeFunctionFlags, BytecodeFunctionKind, CallRange, CaptureDescriptor,
    CaptureSource, ConstantValue, DeoptFrameValue, DeoptSnapshot, DeoptValueSource,
    DirectEvalLexicalScope, DirectEvalLexicalSite, DirectEvalSiteFlags, ExceptionHandler,
    ExceptionHandlerKind, FeedbackSiteDescriptor, FeedbackSiteKind, FeedbackSiteMetadata,
    LoopIterationEnvironmentSite, RuntimeStateCapture, SafepointDescriptor, SafepointKind,
    SourceMapEntry, ThisMode, WideAbcOperands, WideAbxOperands,
};
pub use opcode::{OPCODE_COUNT, Opcode};
