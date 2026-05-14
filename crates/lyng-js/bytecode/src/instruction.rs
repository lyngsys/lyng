use crate::{metadata::CallRange, Opcode};
use lyng_js_types::FeedbackSlotId;

pub const INSTRUCTION_WIDTH: usize = 4;
const WIDE_INSTRUCTION_WIDTH: usize = 8;
const FEEDBACK_SLOT_WIDTH: usize = 2;
const CALL_RANGE_WIDTH: usize = 4;
const ABC_EXTRA_WIDE_THRESHOLD: u16 = 0x01ff;

/// Logical instruction forms used by the bytecode decoder.
///
/// IC-shaped opcodes (anything where [`Opcode::has_feedback_slot`] is true) always carry
/// a mandatory trailing feedback slot, matching V8 / JSC's always-allocate IC design.
/// `Abc` / `Abx` / `Ax` are for non-IC opcodes; `AbcSlot` / `AbxSlot` / `CallRange` carry
/// the slot inline.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Instruction {
    Abc {
        opcode: Opcode,
        a: u16,
        b: u16,
        c: u16,
    },
    Abx {
        opcode: Opcode,
        a: u16,
        bx: u32,
    },
    Ax {
        opcode: Opcode,
        ax: i32,
    },
    AbcSlot {
        opcode: Opcode,
        a: u16,
        b: u16,
        c: u16,
        slot: FeedbackSlotId,
    },
    AbxSlot {
        opcode: Opcode,
        a: u16,
        bx: u32,
        slot: FeedbackSlotId,
    },
    CallRange {
        opcode: Opcode,
        a: u16,
        b: u16,
        c: u16,
        range: CallRange,
        slot: FeedbackSlotId,
    },
}

impl Instruction {
    #[inline]
    pub const fn abc(opcode: Opcode, a: u16, b: u16, c: u16) -> Self {
        Self::Abc { opcode, a, b, c }
    }

    #[inline]
    pub const fn abx(opcode: Opcode, a: u16, bx: u32) -> Self {
        Self::Abx { opcode, a, bx }
    }

    #[inline]
    pub const fn ax(opcode: Opcode, ax: i32) -> Self {
        Self::Ax { opcode, ax }
    }

    #[inline]
    pub const fn abc_slot(opcode: Opcode, a: u16, b: u16, c: u16, slot: FeedbackSlotId) -> Self {
        debug_assert!(
            opcode.has_feedback_slot(),
            "abc_slot requires an IC-shaped opcode"
        );
        Self::AbcSlot {
            opcode,
            a,
            b,
            c,
            slot,
        }
    }

    #[inline]
    pub const fn abx_slot(opcode: Opcode, a: u16, bx: u32, slot: FeedbackSlotId) -> Self {
        debug_assert!(
            opcode.has_feedback_slot(),
            "abx_slot requires an IC-shaped opcode"
        );
        Self::AbxSlot {
            opcode,
            a,
            bx,
            slot,
        }
    }

    #[inline]
    pub const fn call_range(
        opcode: Opcode,
        a: u16,
        b: u16,
        c: u16,
        range: CallRange,
        slot: FeedbackSlotId,
    ) -> Self {
        debug_assert!(
            opcode.has_call_range(),
            "call_range requires a Call / TailCall / Construct opcode"
        );
        Self::CallRange {
            opcode,
            a,
            b,
            c,
            range,
            slot,
        }
    }

    #[inline]
    pub const fn opcode(self) -> Opcode {
        match self {
            Self::Abc { opcode, .. }
            | Self::Abx { opcode, .. }
            | Self::Ax { opcode, .. }
            | Self::AbcSlot { opcode, .. }
            | Self::AbxSlot { opcode, .. }
            | Self::CallRange { opcode, .. } => opcode,
        }
    }

    #[inline]
    pub const fn feedback_slot(self) -> Option<FeedbackSlotId> {
        match self {
            Self::AbcSlot { slot, .. }
            | Self::AbxSlot { slot, .. }
            | Self::CallRange { slot, .. } => Some(slot),
            Self::Abc { .. } | Self::Abx { .. } | Self::Ax { .. } => None,
        }
    }

    #[inline]
    /// # Panics
    /// Panics if this instruction does not encode to the fixed 4-byte word format.
    pub fn encode_word(self) -> u32 {
        let bytes = self.encode_bytes();
        assert!(
            bytes.len() == INSTRUCTION_WIDTH,
            "encode_word requires exactly {INSTRUCTION_WIDTH} encoded bytes, got {}",
            bytes.len()
        );
        let mut word = [0; INSTRUCTION_WIDTH];
        word.copy_from_slice(&bytes);
        u32::from_le_bytes(word)
    }

    #[inline]
    pub const fn encoded_len(self) -> usize {
        match self {
            Self::Abc { opcode, a, b, c } => abc_encoded_len(opcode, a, b, c, false),
            Self::AbcSlot {
                opcode, a, b, c, ..
            } => abc_encoded_len(opcode, a, b, c, true),
            Self::Abx { opcode, a, bx } => abx_encoded_len(opcode, a, bx, false),
            Self::AbxSlot { opcode, a, bx, .. } => abx_encoded_len(opcode, a, bx, true),
            Self::Ax { opcode, .. } => ax_encoded_len(opcode),
            Self::CallRange { .. } => INSTRUCTION_WIDTH + CALL_RANGE_WIDTH + FEEDBACK_SLOT_WIDTH,
        }
    }

    #[inline]
    pub fn encode_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.encoded_len());
        self.write_bytes(&mut bytes);
        bytes
    }

    #[inline]
    /// # Panics
    /// Panics if a compact short-form instruction carries an operand outside its encoded range.
    pub fn write_bytes(self, bytes: &mut Vec<u8>) {
        match self {
            Self::Abc { opcode, a, b, c } => write_abc(bytes, opcode, a, b, c, None),
            Self::AbcSlot {
                opcode,
                a,
                b,
                c,
                slot,
            } => write_abc(bytes, opcode, a, b, c, Some(slot)),
            Self::Abx { opcode, a, bx } => write_abx(bytes, opcode, a, bx, None),
            Self::AbxSlot {
                opcode,
                a,
                bx,
                slot,
            } => write_abx(bytes, opcode, a, bx, Some(slot)),
            Self::Ax { opcode, ax } => write_ax(bytes, opcode, ax),
            Self::CallRange {
                opcode,
                a,
                b,
                c,
                range,
                slot,
            } => write_call_range(bytes, opcode, a, b, c, range, slot),
        }
    }

    #[inline]
    /// # Panics
    /// Panics if this instruction is not in the `Ax` form.
    pub fn patch_ax(&mut self, ax: i32) {
        match self {
            Self::Ax { ax: current, .. } => *current = ax,
            _ => panic!("only Ax instructions can be patched with a 24-bit immediate"),
        }
    }

    #[inline]
    /// # Panics
    /// Panics if this instruction is not in the `Abx` form.
    pub fn patch_bx(&mut self, bx: u32) {
        match self {
            Self::Abx { bx: current, .. } | Self::AbxSlot { bx: current, .. } => *current = bx,
            _ => panic!("only Abx instructions can be patched with a 32-bit immediate"),
        }
    }

    #[inline]
    pub const fn ax_value(self) -> Option<i32> {
        match self {
            Self::Ax { ax, .. } => Some(ax),
            _ => None,
        }
    }

    #[inline]
    pub const fn bx_value(self) -> Option<u32> {
        match self {
            Self::Abx { bx, .. } | Self::AbxSlot { bx, .. } => Some(bx),
            _ => None,
        }
    }
}

const fn abc_prefix(a: u16, b: u16, c: u16) -> Option<Opcode> {
    if a <= u8::MAX as u16 && b <= u8::MAX as u16 && c <= u8::MAX as u16 {
        None
    } else if a <= ABC_EXTRA_WIDE_THRESHOLD
        && b <= ABC_EXTRA_WIDE_THRESHOLD
        && c <= ABC_EXTRA_WIDE_THRESHOLD
    {
        Some(Opcode::Wide)
    } else {
        Some(Opcode::ExtraWide)
    }
}

const fn abx_prefix(a: u16, bx: u32) -> Option<Opcode> {
    if a <= u8::MAX as u16 && bx <= u16::MAX as u32 {
        None
    } else if bx <= 0x00ff_ffff {
        Some(Opcode::Wide)
    } else {
        Some(Opcode::ExtraWide)
    }
}

const fn abc_encoded_len(opcode: Opcode, a: u16, b: u16, c: u16, has_slot: bool) -> usize {
    let base = if matches!(
        opcode,
        Opcode::LdaUndefined
            | Opcode::LdaNull
            | Opcode::LdaTrue
            | Opcode::LdaFalse
            | Opcode::LdaZero
            | Opcode::LdaOne
            | Opcode::Star0
            | Opcode::Star1
            | Opcode::Star2
            | Opcode::Star3
            | Opcode::Star4
            | Opcode::Star5
            | Opcode::Star6
            | Opcode::Star7
    ) {
        1
    } else if matches!(opcode, Opcode::Ldar) {
        2
    } else if abc_prefix(a, b, c).is_some() {
        WIDE_INSTRUCTION_WIDTH
    } else {
        INSTRUCTION_WIDTH
    };
    base + if has_slot { FEEDBACK_SLOT_WIDTH } else { 0 }
}

const fn abx_encoded_len(opcode: Opcode, a: u16, bx: u32, has_slot: bool) -> usize {
    let base = if matches!(
        opcode,
        Opcode::LdaSmi8
            | Opcode::LdaConst8
            | Opcode::Jump8
            | Opcode::LoadLocal0
            | Opcode::LoadLocal1
            | Opcode::LoadLocal2
            | Opcode::LoadLocal3
            | Opcode::StoreLocal0
            | Opcode::StoreLocal1
            | Opcode::StoreLocal2
            | Opcode::StoreLocal3
    ) {
        2
    } else if matches!(
        opcode,
        Opcode::LoadSmi8 | Opcode::LoadConst8 | Opcode::JumpIfTrue8 | Opcode::JumpIfFalse8
    ) {
        3
    } else if abx_prefix(a, bx).is_some() {
        WIDE_INSTRUCTION_WIDTH
    } else {
        INSTRUCTION_WIDTH
    };
    base + if has_slot { FEEDBACK_SLOT_WIDTH } else { 0 }
}

const fn ax_encoded_len(opcode: Opcode) -> usize {
    if matches!(opcode, Opcode::Jump8) {
        2
    } else {
        INSTRUCTION_WIDTH
    }
}

fn write_abc(
    bytes: &mut Vec<u8>,
    opcode: Opcode,
    a: u16,
    b: u16,
    c: u16,
    slot: Option<FeedbackSlotId>,
) {
    assert!(
        !opcode.is_prefix(),
        "prefix opcodes cannot be semantic instructions"
    );
    match opcode {
        Opcode::LdaUndefined
        | Opcode::LdaNull
        | Opcode::LdaTrue
        | Opcode::LdaFalse
        | Opcode::LdaZero
        | Opcode::LdaOne
        | Opcode::Star0
        | Opcode::Star1
        | Opcode::Star2
        | Opcode::Star3
        | Opcode::Star4
        | Opcode::Star5
        | Opcode::Star6
        | Opcode::Star7 => bytes.push(opcode as u8),
        Opcode::Ldar => bytes.extend_from_slice(&[opcode as u8, narrow_u8(a, "Ldar register")]),
        _ => {
            if let Some(prefix) = abc_prefix(a, b, c) {
                bytes.extend_from_slice(&[
                    prefix as u8,
                    opcode as u8,
                    low_u8(a),
                    low_u8(b),
                    low_u8(c),
                    high_u8(a),
                    high_u8(b),
                    high_u8(c),
                ]);
            } else {
                bytes.extend_from_slice(&[opcode as u8, low_u8(a), low_u8(b), low_u8(c)]);
            }
        }
    }
    if let Some(slot) = slot {
        write_feedback_slot(bytes, slot);
    }
}

fn write_abx(bytes: &mut Vec<u8>, opcode: Opcode, a: u16, bx: u32, slot: Option<FeedbackSlotId>) {
    match opcode {
        Opcode::LdaSmi8 | Opcode::LdaConst8 => {
            bytes.extend_from_slice(&[opcode as u8, narrow_u8_u32(bx, "accumulator byte operand")]);
        }
        Opcode::LoadLocal0
        | Opcode::LoadLocal1
        | Opcode::LoadLocal2
        | Opcode::LoadLocal3
        | Opcode::StoreLocal0
        | Opcode::StoreLocal1
        | Opcode::StoreLocal2
        | Opcode::StoreLocal3 => {
            bytes.extend_from_slice(&[opcode as u8, narrow_u8(a, "local compact register")]);
        }
        Opcode::LoadSmi8 | Opcode::LoadConst8 | Opcode::JumpIfTrue8 | Opcode::JumpIfFalse8 => {
            bytes.extend_from_slice(&[
                opcode as u8,
                narrow_u8(a, "compact Abx target register"),
                narrow_u8_u32(bx, "compact Abx operand"),
            ]);
        }
        _ => {
            let bx_bytes = bx.to_le_bytes();
            if let Some(prefix) = abx_prefix(a, bx) {
                bytes.extend_from_slice(&[
                    prefix as u8,
                    opcode as u8,
                    low_u8(a),
                    bx_bytes[0],
                    bx_bytes[1],
                    high_u8(a),
                    bx_bytes[2],
                    if prefix == Opcode::ExtraWide {
                        bx_bytes[3]
                    } else {
                        0
                    },
                ]);
            } else {
                bytes.extend_from_slice(&[opcode as u8, low_u8(a), bx_bytes[0], bx_bytes[1]]);
            }
        }
    }
    if let Some(slot) = slot {
        write_feedback_slot(bytes, slot);
    }
}

fn write_ax(bytes: &mut Vec<u8>, opcode: Opcode, ax: i32) {
    if opcode == Opcode::Jump8 {
        assert!(
            (i32::from(i8::MIN)..=i32::from(i8::MAX)).contains(&ax),
            "Jump8 operand must fit in i8"
        );
        bytes.extend_from_slice(&[opcode as u8, ax.to_le_bytes()[0]]);
    } else {
        let raw = ax & 0x00ff_ffff;
        let raw = raw.to_le_bytes();
        bytes.extend_from_slice(&[opcode as u8, raw[0], raw[1], raw[2]]);
    }
}

fn write_call_range(
    bytes: &mut Vec<u8>,
    opcode: Opcode,
    a: u16,
    b: u16,
    c: u16,
    range: CallRange,
    slot: FeedbackSlotId,
) {
    assert!(
        opcode.has_call_range(),
        "only general call opcodes carry inline call ranges"
    );
    bytes.extend_from_slice(&[
        opcode as u8,
        narrow_u8(a, "call result/callee register"),
        narrow_u8(b, "call callee/this register"),
        narrow_u8(c, "call this register"),
    ]);
    let range = range.encode().to_le_bytes();
    bytes.extend_from_slice(&range);
    write_feedback_slot(bytes, slot);
}

fn write_feedback_slot(bytes: &mut Vec<u8>, slot: FeedbackSlotId) {
    assert!(
        u16::try_from(slot.get()).is_ok(),
        "feedback slot must fit in u16"
    );
    let slot = slot.get().to_le_bytes();
    bytes.extend_from_slice(&[slot[0], slot[1]]);
}

const fn low_u8(value: u16) -> u8 {
    value.to_le_bytes()[0]
}

const fn high_u8(value: u16) -> u8 {
    value.to_le_bytes()[1]
}

fn narrow_u8(value: u16, context: &str) -> u8 {
    u8::try_from(value).unwrap_or_else(|_| panic!("{context} must fit in u8"))
}

fn narrow_u8_u32(value: u32, context: &str) -> u8 {
    u8::try_from(value).unwrap_or_else(|_| panic!("{context} must fit in u8"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abc_words_encode_opcode_and_registers() {
        let word = Instruction::abc(Opcode::Move, 1, 2, 3).encode_word();
        assert_eq!(word.to_le_bytes(), [Opcode::Move as u8, 1, 2, 3]);
    }

    #[test]
    fn abx_words_encode_u16_payload() {
        let word = Instruction::abx(Opcode::LoadConst, 7, 0x1234).encode_word();
        assert_eq!(word.to_le_bytes(), [Opcode::LoadConst as u8, 7, 0x34, 0x12]);
    }

    #[test]
    #[should_panic(expected = "encode_word requires exactly 4 encoded bytes")]
    fn feedback_instruction_encode_word_panics_instead_of_truncating() {
        let slot = FeedbackSlotId::from_raw(1).expect("test slot should be non-zero");
        // Add is an IC-shaped ABX-arity opcode; its 6-byte encoding can't fit in a 4-byte word.
        Instruction::abx_slot(Opcode::LoadGlobal, 1, 2, slot).encode_word();
    }

    #[test]
    #[should_panic(expected = "compact Abx operand must fit in u8")]
    fn compact_abx_write_panics_when_payload_would_truncate() {
        Instruction::abx(Opcode::LoadSmi8, 0, u32::from(u8::MAX) + 1).encode_bytes();
    }

    #[test]
    #[should_panic(expected = "Jump8 operand must fit in i8")]
    fn jump8_write_panics_when_payload_would_truncate() {
        Instruction::ax(Opcode::Jump8, i32::from(i8::MAX) + 1).encode_bytes();
    }

    #[test]
    #[should_panic(expected = "feedback slot must fit in u16")]
    fn feedback_abc_write_panics_when_slot_overflows_u16() {
        let slot = FeedbackSlotId::from_raw(u32::from(u16::MAX) + 1)
            .expect("test slot should be non-zero");
        Instruction::abc_slot(Opcode::GetNamedProperty, 0, 1, 2, slot).encode_bytes();
    }

    #[test]
    fn wide_abc_prefix_inlines_high_operand_bytes() {
        // Move is a non-IC ABC opcode; no slot bytes follow.
        let bytes = Instruction::abc(Opcode::Move, 0x0123, 0x0045, 0x01ab).encode_bytes();
        assert_eq!(
            bytes,
            vec![
                Opcode::Wide as u8,
                Opcode::Move as u8,
                0x23,
                0x45,
                0xab,
                0x01,
                0x00,
                0x01
            ]
        );
    }

    #[test]
    fn extra_wide_abc_prefix_inlines_high_operand_bytes() {
        let slot = FeedbackSlotId::from_raw(1).expect("test slot should be non-zero");
        // IC-shaped Add now carries the slot directly without a *Profiled mirror.
        let bytes =
            Instruction::abc_slot(Opcode::Add, 0x0223, 0x0045, 0x01ab, slot).encode_bytes();
        assert_eq!(bytes[0], Opcode::ExtraWide as u8);
        assert_eq!(bytes[1], Opcode::Add as u8);
        assert_eq!(&bytes[8..], &[1, 0]);
    }

    #[test]
    fn call_range_is_inline() {
        let slot = FeedbackSlotId::from_raw(7).expect("test slot should be non-zero");
        let bytes =
            Instruction::call_range(Opcode::Call, 1, 2, 3, CallRange::new(4, 5), slot)
                .encode_bytes();
        assert_eq!(
            bytes,
            vec![Opcode::Call as u8, 1, 2, 3, 5, 0, 4, 0, 7, 0]
        );
    }

    #[test]
    fn ax_patch_updates_24_bit_immediate() {
        let mut inst = Instruction::ax(Opcode::Jump, 0);
        inst.patch_ax(-3);
        assert_eq!(inst.ax_value(), Some(-3));
    }

    #[test]
    fn abx_patch_updates_32_bit_immediate() {
        let mut inst = Instruction::abx(Opcode::JumpIfFalse, 4, 0);
        inst.patch_bx(0x1234);
        assert_eq!(inst.bx_value(), Some(0x1234));
    }
}
