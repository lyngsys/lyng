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
    Wide,
    ExtraWide,
    LdaUndefined,
    LdaNull,
    LdaTrue,
    LdaFalse,
    LdaZero,
    LdaOne,
    LdaSmi8,
    LdaConst8,
    Ldar,
    Star0,
    Star1,
    Star2,
    Star3,
    Star4,
    Star5,
    Star6,
    Star7,
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
}

pub const OPCODE_COUNT: u8 = Opcode::StoreLocal3 as u8 + 1;

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
    Opcode::Wide,
    Opcode::ExtraWide,
    Opcode::LdaUndefined,
    Opcode::LdaNull,
    Opcode::LdaTrue,
    Opcode::LdaFalse,
    Opcode::LdaZero,
    Opcode::LdaOne,
    Opcode::LdaSmi8,
    Opcode::LdaConst8,
    Opcode::Ldar,
    Opcode::Star0,
    Opcode::Star1,
    Opcode::Star2,
    Opcode::Star3,
    Opcode::Star4,
    Opcode::Star5,
    Opcode::Star6,
    Opcode::Star7,
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
];

impl Opcode {
    #[inline]
    pub fn from_byte(raw: u8) -> Option<Self> {
        OPCODES.get(usize::from(raw)).copied()
    }

    /// Encoded byte length of one narrow instruction with this opcode, matching the layout
    /// produced by [`crate::Instruction::write_bytes`] and consumed by the VM dispatch
    /// loop. Mirrors the table in [`crate::Instruction::encoded_len`] without first
    /// materializing an `Instruction` enum value — used by the byte-stream dispatcher to
    /// advance the program counter after each opcode. `Wide` / `ExtraWide` forms add their
    /// prefix and additional operand bytes on top of this length.
    #[inline]
    #[must_use]
    #[allow(
        clippy::match_same_arms,
        reason = "opcode families stay grouped per encoded length to keep the table auditable"
    )]
    pub const fn encoded_len(self) -> u8 {
        match self {
            Self::LdaUndefined
            | Self::LdaNull
            | Self::LdaTrue
            | Self::LdaFalse
            | Self::LdaZero
            | Self::LdaOne
            | Self::Star0
            | Self::Star1
            | Self::Star2
            | Self::Star3
            | Self::Star4
            | Self::Star5
            | Self::Star6
            | Self::Star7 => 1,
            Self::Jump8
            | Self::LdaSmi8
            | Self::LdaConst8
            | Self::Ldar
            | Self::LoadLocal0
            | Self::LoadLocal1
            | Self::LoadLocal2
            | Self::LoadLocal3
            | Self::StoreLocal0
            | Self::StoreLocal1
            | Self::StoreLocal2
            | Self::StoreLocal3 => 2,
            Self::LoadSmi8 | Self::LoadConst8 | Self::JumpIfTrue8 | Self::JumpIfFalse8 => 3,
            // Call / TailCall / Construct: 4-byte ABC + 4-byte CallRange + 2-byte slot.
            Self::Call | Self::TailCall | Self::Construct => 10,
            Self::Wide | Self::ExtraWide => 1,
            // IC-shaped ABC/ABX opcodes: 4-byte ABC/ABX + 2-byte mandatory feedback slot.
            opcode if opcode.has_feedback_slot() => 6,
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
            Self::Wide => "Wide",
            Self::ExtraWide => "ExtraWide",
            Self::LdaUndefined => "LdaUndefined",
            Self::LdaNull => "LdaNull",
            Self::LdaTrue => "LdaTrue",
            Self::LdaFalse => "LdaFalse",
            Self::LdaZero => "LdaZero",
            Self::LdaOne => "LdaOne",
            Self::LdaSmi8 => "LdaSmi8",
            Self::LdaConst8 => "LdaConst8",
            Self::Ldar => "Ldar",
            Self::Star0 => "Star0",
            Self::Star1 => "Star1",
            Self::Star2 => "Star2",
            Self::Star3 => "Star3",
            Self::Star4 => "Star4",
            Self::Star5 => "Star5",
            Self::Star6 => "Star6",
            Self::Star7 => "Star7",
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
    pub const fn is_prefix(self) -> bool {
        matches!(self, Self::Wide | Self::ExtraWide)
    }

    /// True for opcodes that carry a mandatory trailing 2-byte feedback slot operand.
    ///
    /// Every IC-shaped opcode (arithmetic, comparison, global/named/keyed property
    /// access, calls, construct) emits a slot at compile time, mirroring V8 / JSC's
    /// always-allocate IC design. Non-IC opcodes (moves, loads, jumps, scope ops, etc.)
    /// have no slot.
    #[inline]
    #[allow(
        clippy::too_many_lines,
        reason = "feedback-capable opcodes are explicit to keep encoding lengths auditable"
    )]
    pub const fn has_feedback_slot(self) -> bool {
        matches!(
            self,
            Self::Negate
                | Self::BitNot
                | Self::Increment
                | Self::Decrement
                | Self::Add
                | Self::AddSmi
                | Self::Sub
                | Self::SubSmi
                | Self::Mul
                | Self::MulSmi
                | Self::Div
                | Self::DivSmi
                | Self::Mod
                | Self::ModSmi
                | Self::Exp
                | Self::BitOr
                | Self::BitXor
                | Self::BitAnd
                | Self::BitAndSmi
                | Self::ShiftLeft
                | Self::ShiftRight
                | Self::UnsignedShiftRight
                | Self::Equal
                | Self::StrictEqual
                | Self::EqualZero
                | Self::LessThan
                | Self::LessEqual
                | Self::GreaterThan
                | Self::GreaterEqual
                | Self::LoadGlobal
                | Self::StoreGlobal
                | Self::AssignGlobal
                | Self::GetNamedProperty
                | Self::SetNamedProperty
                | Self::AssignNamedProperty
                | Self::StrictAssignNamedProperty
                | Self::GetKeyedProperty
                | Self::SetKeyedProperty
                | Self::AssignKeyedProperty
                | Self::StrictAssignKeyedProperty
                | Self::Call0
                | Self::Call1
                | Self::Call2
                | Self::Call3
                | Self::Call
                | Self::TailCall
                | Self::Construct
        )
    }

    #[inline]
    pub const fn has_call_range(self) -> bool {
        matches!(self, Self::Call | Self::TailCall | Self::Construct)
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

    #[inline]
    pub const fn accumulator_store_index(self) -> Option<u16> {
        match self {
            Self::Star0 => Some(0),
            Self::Star1 => Some(1),
            Self::Star2 => Some(2),
            Self::Star3 => Some(3),
            Self::Star4 => Some(4),
            Self::Star5 => Some(5),
            Self::Star6 => Some(6),
            Self::Star7 => Some(7),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opcode_count_matches_last_discriminant() {
        assert_eq!(Opcode::StoreLocal3 as u8 + 1, OPCODE_COUNT);
    }

    #[test]
    fn has_feedback_slot_implies_six_byte_narrow_encoding() {
        // IC-shaped ABC/ABX opcodes encode 4-byte base + 2-byte feedback slot.
        assert_eq!(Opcode::Add.encoded_len(), 6);
        assert_eq!(Opcode::GetNamedProperty.encoded_len(), 6);
        assert_eq!(Opcode::LoadGlobal.encoded_len(), 6);
        // Call / TailCall / Construct: 4-byte ABC + 4-byte CallRange + 2-byte slot.
        assert_eq!(Opcode::Call.encoded_len(), 10);
        assert_eq!(Opcode::TailCall.encoded_len(), 10);
        assert_eq!(Opcode::Construct.encoded_len(), 10);
        // Non-IC opcodes retain their original encoding.
        assert_eq!(Opcode::Move.encoded_len(), 4);
        assert_eq!(Opcode::Jump.encoded_len(), 4);
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
