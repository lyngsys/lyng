/// Bytecode opcodes for the lyng-js register VM.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Opcode {
    Nop = 0,
    Move,
    LoadUndefined,
    LoadUninitializedLexical,
    LoadNull,
    LoadTrue,
    LoadFalse,
    LoadZero,
    LoadOne,
    LoadSmi,
    LoadConst,
    LoadEnvSlot,
    StoreEnvSlot,
    AssignEnvSlot,
    LoadGlobal,
    StoreGlobal,
    AssignGlobal,
    DeleteGlobal,
    LoadName,
    ResolveName,
    ResolveGlobal,
    AssignName,
    AssignVariableName,
    DeleteName,
    CaptureName,
    LoadCapturedName,
    LoadCapturedNameThis,
    AssignCapturedName,
    LoadThis,
    LoadCallee,
    LoadNewTarget,
    Add,
    AddSmi,
    Sub,
    SubSmi,
    Mul,
    MulSmi,
    Div,
    Mod,
    DivSmi,
    ModSmi,
    Exp,
    BitOr,
    BitXor,
    BitAnd,
    BitAndSmi,
    BitNot,
    ShiftLeft,
    ShiftRight,
    UnsignedShiftRight,
    Negate,
    Increment,
    Decrement,
    Equal,
    StrictEqual,
    EqualZero,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
    TypeOf,
    InstanceOf,
    In,
    Jump,
    JumpIfTrue,
    JumpIfFalse,
    LoopHeader,
    Return,
    ReturnUndefined,
    CreateObject,
    CreateArray,
    CheckObjectCoercible,
    ThrowIfUninitialized,
    DefineNamedProperty,
    DefineKeyedProperty,
    StoreDenseElement,
    LoadDenseElement,
    GetNamedProperty,
    SetNamedProperty,
    AssignNamedProperty,
    StrictAssignNamedProperty,
    GetKeyedProperty,
    SetKeyedProperty,
    AssignKeyedProperty,
    StrictAssignKeyedProperty,
    DeleteProperty,
    CopyDataProperties,
    SetFunctionName,
    ToPropertyKey,
    Call0,
    Call1,
    Call2,
    Call3,
    Call,
    CallMethod,
    TailCall,
    Construct,
    CreateClosure,
    CreateForIn,
    AdvanceForIn,
    CloseForIn,
    CreateIterator,
    AdvanceIterator,
    CloseIterator,
    SuspendGeneratorStart,
    Yield,
    Await,
    DelegateYield,
    LoadResumeKind,
    LoadResumeValue,
    PushClosureEnv,
    PopClosureEnv,
    EnterEnvScope,
    LeaveEnvScope,
    PushWithEnv,
    PopWithEnv,
    Throw,
    EnterHandler,
    LeaveHandler,
    LoadException,
    LoadSmi8,
    LoadConst8,
    Jump8,
    JumpIfTrue8,
    JumpIfFalse8,
    LoadLocal0,
    LoadLocal1,
    LoadLocal2,
    LoadLocal3,
    StoreLocal0,
    StoreLocal1,
    StoreLocal2,
    StoreLocal3,
    ProfiledAbc,
    ProfiledAbx,
}

pub const OPCODE_COUNT: u8 = Opcode::ProfiledAbx as u8 + 1;

const OPCODES: [Opcode; OPCODE_COUNT as usize] = [
    Opcode::Nop,
    Opcode::Move,
    Opcode::LoadUndefined,
    Opcode::LoadUninitializedLexical,
    Opcode::LoadNull,
    Opcode::LoadTrue,
    Opcode::LoadFalse,
    Opcode::LoadZero,
    Opcode::LoadOne,
    Opcode::LoadSmi,
    Opcode::LoadConst,
    Opcode::LoadEnvSlot,
    Opcode::StoreEnvSlot,
    Opcode::AssignEnvSlot,
    Opcode::LoadGlobal,
    Opcode::StoreGlobal,
    Opcode::AssignGlobal,
    Opcode::DeleteGlobal,
    Opcode::LoadName,
    Opcode::ResolveName,
    Opcode::ResolveGlobal,
    Opcode::AssignName,
    Opcode::AssignVariableName,
    Opcode::DeleteName,
    Opcode::CaptureName,
    Opcode::LoadCapturedName,
    Opcode::LoadCapturedNameThis,
    Opcode::AssignCapturedName,
    Opcode::LoadThis,
    Opcode::LoadCallee,
    Opcode::LoadNewTarget,
    Opcode::Add,
    Opcode::AddSmi,
    Opcode::Sub,
    Opcode::SubSmi,
    Opcode::Mul,
    Opcode::MulSmi,
    Opcode::Div,
    Opcode::Mod,
    Opcode::DivSmi,
    Opcode::ModSmi,
    Opcode::Exp,
    Opcode::BitOr,
    Opcode::BitXor,
    Opcode::BitAnd,
    Opcode::BitAndSmi,
    Opcode::BitNot,
    Opcode::ShiftLeft,
    Opcode::ShiftRight,
    Opcode::UnsignedShiftRight,
    Opcode::Negate,
    Opcode::Increment,
    Opcode::Decrement,
    Opcode::Equal,
    Opcode::StrictEqual,
    Opcode::EqualZero,
    Opcode::LessThan,
    Opcode::LessEqual,
    Opcode::GreaterThan,
    Opcode::GreaterEqual,
    Opcode::TypeOf,
    Opcode::InstanceOf,
    Opcode::In,
    Opcode::Jump,
    Opcode::JumpIfTrue,
    Opcode::JumpIfFalse,
    Opcode::LoopHeader,
    Opcode::Return,
    Opcode::ReturnUndefined,
    Opcode::CreateObject,
    Opcode::CreateArray,
    Opcode::CheckObjectCoercible,
    Opcode::ThrowIfUninitialized,
    Opcode::DefineNamedProperty,
    Opcode::DefineKeyedProperty,
    Opcode::StoreDenseElement,
    Opcode::LoadDenseElement,
    Opcode::GetNamedProperty,
    Opcode::SetNamedProperty,
    Opcode::AssignNamedProperty,
    Opcode::StrictAssignNamedProperty,
    Opcode::GetKeyedProperty,
    Opcode::SetKeyedProperty,
    Opcode::AssignKeyedProperty,
    Opcode::StrictAssignKeyedProperty,
    Opcode::DeleteProperty,
    Opcode::CopyDataProperties,
    Opcode::SetFunctionName,
    Opcode::ToPropertyKey,
    Opcode::Call0,
    Opcode::Call1,
    Opcode::Call2,
    Opcode::Call3,
    Opcode::Call,
    Opcode::CallMethod,
    Opcode::TailCall,
    Opcode::Construct,
    Opcode::CreateClosure,
    Opcode::CreateForIn,
    Opcode::AdvanceForIn,
    Opcode::CloseForIn,
    Opcode::CreateIterator,
    Opcode::AdvanceIterator,
    Opcode::CloseIterator,
    Opcode::SuspendGeneratorStart,
    Opcode::Yield,
    Opcode::Await,
    Opcode::DelegateYield,
    Opcode::LoadResumeKind,
    Opcode::LoadResumeValue,
    Opcode::PushClosureEnv,
    Opcode::PopClosureEnv,
    Opcode::EnterEnvScope,
    Opcode::LeaveEnvScope,
    Opcode::PushWithEnv,
    Opcode::PopWithEnv,
    Opcode::Throw,
    Opcode::EnterHandler,
    Opcode::LeaveHandler,
    Opcode::LoadException,
    Opcode::LoadSmi8,
    Opcode::LoadConst8,
    Opcode::Jump8,
    Opcode::JumpIfTrue8,
    Opcode::JumpIfFalse8,
    Opcode::LoadLocal0,
    Opcode::LoadLocal1,
    Opcode::LoadLocal2,
    Opcode::LoadLocal3,
    Opcode::StoreLocal0,
    Opcode::StoreLocal1,
    Opcode::StoreLocal2,
    Opcode::StoreLocal3,
    Opcode::ProfiledAbc,
    Opcode::ProfiledAbx,
];

impl Opcode {
    #[inline]
    pub fn from_byte(raw: u8) -> Option<Self> {
        OPCODES.get(usize::from(raw)).copied()
    }

    /// Encoded byte length of one instruction with this opcode, matching the layout
    /// produced by [`crate::Instruction::write_bytes`] and consumed by the VM dispatch
    /// loop. Mirrors the table in [`crate::Instruction::encoded_len`] without first
    /// materializing an `Instruction` enum value — used by the byte-stream dispatcher to
    /// advance the program counter after each opcode.
    #[inline]
    #[must_use]
    pub const fn encoded_len(self) -> u8 {
        match self {
            Self::Jump8
            | Self::LoadLocal0
            | Self::LoadLocal1
            | Self::LoadLocal2
            | Self::LoadLocal3
            | Self::StoreLocal0
            | Self::StoreLocal1
            | Self::StoreLocal2
            | Self::StoreLocal3 => 2,
            Self::LoadSmi8 | Self::LoadConst8 | Self::JumpIfTrue8 | Self::JumpIfFalse8 => 3,
            Self::ProfiledAbc | Self::ProfiledAbx => 7,
            _ => 4,
        }
    }

    #[inline]
    #[allow(
        clippy::too_many_lines,
        reason = "the opcode display name table mirrors the full opcode enum in one place"
    )]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Nop => "Nop",
            Self::Move => "Move",
            Self::LoadUndefined => "LoadUndefined",
            Self::LoadUninitializedLexical => "LoadUninitializedLexical",
            Self::LoadNull => "LoadNull",
            Self::LoadTrue => "LoadTrue",
            Self::LoadFalse => "LoadFalse",
            Self::LoadZero => "LoadZero",
            Self::LoadOne => "LoadOne",
            Self::LoadSmi => "LoadSmi",
            Self::LoadConst => "LoadConst",
            Self::LoadEnvSlot => "LoadEnvSlot",
            Self::StoreEnvSlot => "StoreEnvSlot",
            Self::AssignEnvSlot => "AssignEnvSlot",
            Self::LoadGlobal => "LoadGlobal",
            Self::StoreGlobal => "StoreGlobal",
            Self::AssignGlobal => "AssignGlobal",
            Self::DeleteGlobal => "DeleteGlobal",
            Self::LoadName => "LoadName",
            Self::ResolveName => "ResolveName",
            Self::ResolveGlobal => "ResolveGlobal",
            Self::AssignName => "AssignName",
            Self::AssignVariableName => "AssignVariableName",
            Self::DeleteName => "DeleteName",
            Self::CaptureName => "CaptureName",
            Self::LoadCapturedName => "LoadCapturedName",
            Self::LoadCapturedNameThis => "LoadCapturedNameThis",
            Self::AssignCapturedName => "AssignCapturedName",
            Self::LoadThis => "LoadThis",
            Self::LoadCallee => "LoadCallee",
            Self::LoadNewTarget => "LoadNewTarget",
            Self::Add => "Add",
            Self::AddSmi => "AddSmi",
            Self::Sub => "Sub",
            Self::SubSmi => "SubSmi",
            Self::Mul => "Mul",
            Self::MulSmi => "MulSmi",
            Self::Div => "Div",
            Self::Mod => "Mod",
            Self::DivSmi => "DivSmi",
            Self::ModSmi => "ModSmi",
            Self::Exp => "Exp",
            Self::BitOr => "BitOr",
            Self::BitXor => "BitXor",
            Self::BitAnd => "BitAnd",
            Self::BitAndSmi => "BitAndSmi",
            Self::BitNot => "BitNot",
            Self::ShiftLeft => "ShiftLeft",
            Self::ShiftRight => "ShiftRight",
            Self::UnsignedShiftRight => "UnsignedShiftRight",
            Self::Negate => "Negate",
            Self::Increment => "Increment",
            Self::Decrement => "Decrement",
            Self::Equal => "Equal",
            Self::StrictEqual => "StrictEqual",
            Self::EqualZero => "EqualZero",
            Self::LessThan => "LessThan",
            Self::LessEqual => "LessEqual",
            Self::GreaterThan => "GreaterThan",
            Self::GreaterEqual => "GreaterEqual",
            Self::TypeOf => "TypeOf",
            Self::InstanceOf => "InstanceOf",
            Self::In => "In",
            Self::Jump => "Jump",
            Self::JumpIfTrue => "JumpIfTrue",
            Self::JumpIfFalse => "JumpIfFalse",
            Self::LoopHeader => "LoopHeader",
            Self::Return => "Return",
            Self::ReturnUndefined => "ReturnUndefined",
            Self::CreateObject => "CreateObject",
            Self::CreateArray => "CreateArray",
            Self::CheckObjectCoercible => "CheckObjectCoercible",
            Self::ThrowIfUninitialized => "ThrowIfUninitialized",
            Self::DefineNamedProperty => "DefineNamedProperty",
            Self::DefineKeyedProperty => "DefineKeyedProperty",
            Self::StoreDenseElement => "StoreDenseElement",
            Self::LoadDenseElement => "LoadDenseElement",
            Self::GetNamedProperty => "GetNamedProperty",
            Self::SetNamedProperty => "SetNamedProperty",
            Self::AssignNamedProperty => "AssignNamedProperty",
            Self::StrictAssignNamedProperty => "StrictAssignNamedProperty",
            Self::GetKeyedProperty => "GetKeyedProperty",
            Self::SetKeyedProperty => "SetKeyedProperty",
            Self::AssignKeyedProperty => "AssignKeyedProperty",
            Self::StrictAssignKeyedProperty => "StrictAssignKeyedProperty",
            Self::DeleteProperty => "DeleteProperty",
            Self::CopyDataProperties => "CopyDataProperties",
            Self::SetFunctionName => "SetFunctionName",
            Self::ToPropertyKey => "ToPropertyKey",
            Self::Call0 => "Call0",
            Self::Call1 => "Call1",
            Self::Call2 => "Call2",
            Self::Call3 => "Call3",
            Self::Call => "Call",
            Self::CallMethod => "CallMethod",
            Self::TailCall => "TailCall",
            Self::Construct => "Construct",
            Self::CreateClosure => "CreateClosure",
            Self::CreateForIn => "CreateForIn",
            Self::AdvanceForIn => "AdvanceForIn",
            Self::CloseForIn => "CloseForIn",
            Self::CreateIterator => "CreateIterator",
            Self::AdvanceIterator => "AdvanceIterator",
            Self::CloseIterator => "CloseIterator",
            Self::SuspendGeneratorStart => "SuspendGeneratorStart",
            Self::Yield => "Yield",
            Self::Await => "Await",
            Self::DelegateYield => "DelegateYield",
            Self::LoadResumeKind => "LoadResumeKind",
            Self::LoadResumeValue => "LoadResumeValue",
            Self::PushClosureEnv => "PushClosureEnv",
            Self::PopClosureEnv => "PopClosureEnv",
            Self::EnterEnvScope => "EnterEnvScope",
            Self::LeaveEnvScope => "LeaveEnvScope",
            Self::PushWithEnv => "PushWithEnv",
            Self::PopWithEnv => "PopWithEnv",
            Self::Throw => "Throw",
            Self::EnterHandler => "EnterHandler",
            Self::LeaveHandler => "LeaveHandler",
            Self::LoadException => "LoadException",
            Self::LoadSmi8 => "LoadSmi8",
            Self::LoadConst8 => "LoadConst8",
            Self::Jump8 => "Jump8",
            Self::JumpIfTrue8 => "JumpIfTrue8",
            Self::JumpIfFalse8 => "JumpIfFalse8",
            Self::LoadLocal0 => "LoadLocal0",
            Self::LoadLocal1 => "LoadLocal1",
            Self::LoadLocal2 => "LoadLocal2",
            Self::LoadLocal3 => "LoadLocal3",
            Self::StoreLocal0 => "StoreLocal0",
            Self::StoreLocal1 => "StoreLocal1",
            Self::StoreLocal2 => "StoreLocal2",
            Self::StoreLocal3 => "StoreLocal3",
            Self::ProfiledAbc => "ProfiledAbc",
            Self::ProfiledAbx => "ProfiledAbx",
        }
    }

    #[inline]
    pub const fn is_jump(self) -> bool {
        matches!(
            self,
            Self::Jump
                | Self::JumpIfTrue
                | Self::JumpIfFalse
                | Self::Jump8
                | Self::JumpIfTrue8
                | Self::JumpIfFalse8
        )
    }

    #[inline]
    pub const fn small_call_arity(self) -> Option<u8> {
        match self {
            Self::Call0 => Some(0),
            Self::Call1 => Some(1),
            Self::Call2 => Some(2),
            Self::Call3 => Some(3),
            _ => None,
        }
    }

    #[inline]
    pub const fn local_load_index(self) -> Option<u16> {
        match self {
            Self::LoadLocal0 => Some(0),
            Self::LoadLocal1 => Some(1),
            Self::LoadLocal2 => Some(2),
            Self::LoadLocal3 => Some(3),
            _ => None,
        }
    }

    #[inline]
    pub const fn local_store_index(self) -> Option<u16> {
        match self {
            Self::StoreLocal0 => Some(0),
            Self::StoreLocal1 => Some(1),
            Self::StoreLocal2 => Some(2),
            Self::StoreLocal3 => Some(3),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opcode_count_matches_last_discriminant() {
        assert_eq!(Opcode::ProfiledAbx as u8 + 1, OPCODE_COUNT);
    }

    #[test]
    fn jump_classification_matches_branch_family() {
        assert!(Opcode::Jump.is_jump());
        assert!(Opcode::JumpIfTrue.is_jump());
        assert!(Opcode::JumpIfFalse.is_jump());
        assert!(Opcode::Jump8.is_jump());
        assert!(Opcode::JumpIfTrue8.is_jump());
        assert!(Opcode::JumpIfFalse8.is_jump());
        assert!(!Opcode::Return.is_jump());
    }

    #[test]
    fn from_byte_round_trips_opcode_discriminants() {
        for raw in 0..OPCODE_COUNT {
            let opcode = Opcode::from_byte(raw).expect("all in-range opcode bytes should decode");
            assert_eq!(opcode as u8, raw);
        }
        assert_eq!(Opcode::from_byte(OPCODE_COUNT), None);
    }
}
