use crate::Opcode;
use lyng_js_types::FeedbackSlotId;

pub const INSTRUCTION_WIDTH: usize = 4;
pub const PROFILED_INSTRUCTION_WIDTH: usize = 7;

/// Logical instruction forms used by the bytecode decoder.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Instruction {
    Abc {
        opcode: Opcode,
        a: u8,
        b: u8,
        c: u8,
    },
    Abx {
        opcode: Opcode,
        a: u8,
        bx: u16,
    },
    Ax {
        opcode: Opcode,
        ax: i32,
    },
    ProfiledAbc {
        opcode: Opcode,
        a: u8,
        b: u8,
        c: u8,
        slot: FeedbackSlotId,
    },
    ProfiledAbx {
        opcode: Opcode,
        a: u8,
        bx: u16,
        slot: FeedbackSlotId,
    },
}

impl Instruction {
    #[inline]
    pub const fn abc(opcode: Opcode, a: u8, b: u8, c: u8) -> Self {
        Self::Abc { opcode, a, b, c }
    }

    #[inline]
    pub const fn abx(opcode: Opcode, a: u8, bx: u16) -> Self {
        Self::Abx { opcode, a, bx }
    }

    #[inline]
    pub const fn ax(opcode: Opcode, ax: i32) -> Self {
        Self::Ax { opcode, ax }
    }

    #[inline]
    pub const fn profiled_abc(opcode: Opcode, a: u8, b: u8, c: u8, slot: FeedbackSlotId) -> Self {
        Self::ProfiledAbc {
            opcode,
            a,
            b,
            c,
            slot,
        }
    }

    #[inline]
    pub const fn profiled_abx(opcode: Opcode, a: u8, bx: u16, slot: FeedbackSlotId) -> Self {
        Self::ProfiledAbx {
            opcode,
            a,
            bx,
            slot,
        }
    }

    #[inline]
    pub const fn opcode(self) -> Opcode {
        match self {
            Self::Abc { opcode, .. }
            | Self::Abx { opcode, .. }
            | Self::Ax { opcode, .. }
            | Self::ProfiledAbc { opcode, .. }
            | Self::ProfiledAbx { opcode, .. } => opcode,
        }
    }

    #[inline]
    pub const fn feedback_slot(self) -> Option<FeedbackSlotId> {
        match self {
            Self::ProfiledAbc { slot, .. } | Self::ProfiledAbx { slot, .. } => Some(slot),
            Self::Abc { .. } | Self::Abx { .. } | Self::Ax { .. } => None,
        }
    }

    #[inline]
    pub const fn without_feedback_slot(self) -> Self {
        match self {
            Self::ProfiledAbc {
                opcode, a, b, c, ..
            } => Self::Abc { opcode, a, b, c },
            Self::ProfiledAbx { opcode, a, bx, .. } => Self::Abx { opcode, a, bx },
            Self::Abc { .. } | Self::Abx { .. } | Self::Ax { .. } => self,
        }
    }

    #[inline]
    pub const fn with_feedback_slot(self, slot: FeedbackSlotId) -> Option<Self> {
        match self {
            Self::Abc { opcode, a, b, c }
            | Self::ProfiledAbc {
                opcode, a, b, c, ..
            } => Some(Self::ProfiledAbc {
                opcode,
                a,
                b,
                c,
                slot,
            }),
            Self::Abx { opcode, a, bx } | Self::ProfiledAbx { opcode, a, bx, .. } => {
                Some(Self::ProfiledAbx {
                    opcode,
                    a,
                    bx,
                    slot,
                })
            }
            Self::Ax { .. } => None,
        }
    }

    #[inline]
    pub fn encode_word(self) -> u32 {
        let bytes = self.encode_bytes();
        let mut word = [0; INSTRUCTION_WIDTH];
        word[..bytes.len().min(INSTRUCTION_WIDTH)]
            .copy_from_slice(&bytes[..bytes.len().min(INSTRUCTION_WIDTH)]);
        u32::from_le_bytes(word)
    }

    #[inline]
    pub const fn encoded_len(self) -> usize {
        if matches!(self, Self::ProfiledAbc { .. } | Self::ProfiledAbx { .. }) {
            return PROFILED_INSTRUCTION_WIDTH;
        }
        match self.opcode() {
            Opcode::Jump8
            | Opcode::LoadLocal0
            | Opcode::LoadLocal1
            | Opcode::LoadLocal2
            | Opcode::LoadLocal3
            | Opcode::StoreLocal0
            | Opcode::StoreLocal1
            | Opcode::StoreLocal2
            | Opcode::StoreLocal3 => 2,
            Opcode::LoadSmi8 | Opcode::LoadConst8 | Opcode::JumpIfTrue8 | Opcode::JumpIfFalse8 => 3,
            _ => INSTRUCTION_WIDTH,
        }
    }

    #[inline]
    pub fn encode_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.encoded_len());
        self.write_bytes(&mut bytes);
        bytes
    }

    #[inline]
    pub fn write_bytes(self, bytes: &mut Vec<u8>) {
        match self {
            Self::Abc { opcode, a, b, c } => bytes.extend_from_slice(&[opcode as u8, a, b, c]),
            Self::Abx { opcode, a, bx } => {
                if matches!(
                    opcode,
                    Opcode::LoadLocal0
                        | Opcode::LoadLocal1
                        | Opcode::LoadLocal2
                        | Opcode::LoadLocal3
                        | Opcode::StoreLocal0
                        | Opcode::StoreLocal1
                        | Opcode::StoreLocal2
                        | Opcode::StoreLocal3
                ) {
                    bytes.extend_from_slice(&[opcode as u8, a]);
                } else if matches!(
                    opcode,
                    Opcode::LoadSmi8
                        | Opcode::LoadConst8
                        | Opcode::JumpIfTrue8
                        | Opcode::JumpIfFalse8
                ) {
                    debug_assert!(u8::try_from(bx).is_ok());
                    bytes.extend_from_slice(&[opcode as u8, a, bx.to_le_bytes()[0]]);
                } else {
                    let bx = bx.to_le_bytes();
                    bytes.extend_from_slice(&[opcode as u8, a, bx[0], bx[1]]);
                }
            }
            Self::Ax { opcode, ax } => {
                if opcode == Opcode::Jump8 {
                    debug_assert!((i32::from(i8::MIN)..=i32::from(i8::MAX)).contains(&ax));
                    bytes.extend_from_slice(&[opcode as u8, ax.to_le_bytes()[0]]);
                } else {
                    let raw = ax & 0x00ff_ffff;
                    let raw = raw.to_le_bytes();
                    bytes.extend_from_slice(&[opcode as u8, raw[0], raw[1], raw[2]]);
                }
            }
            Self::ProfiledAbc {
                opcode,
                a,
                b,
                c,
                slot,
            } => {
                debug_assert!(u16::try_from(slot.get()).is_ok());
                let slot = slot.get().to_le_bytes();
                bytes.extend_from_slice(&[
                    Opcode::ProfiledAbc as u8,
                    opcode as u8,
                    a,
                    b,
                    c,
                    slot[0],
                    slot[1],
                ]);
            }
            Self::ProfiledAbx {
                opcode,
                a,
                bx,
                slot,
            } => {
                debug_assert!(u16::try_from(slot.get()).is_ok());
                let bx = bx.to_le_bytes();
                let slot = slot.get().to_le_bytes();
                bytes.extend_from_slice(&[
                    Opcode::ProfiledAbx as u8,
                    opcode as u8,
                    a,
                    bx[0],
                    bx[1],
                    slot[0],
                    slot[1],
                ]);
            }
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
    pub fn patch_bx(&mut self, bx: u16) {
        match self {
            Self::Abx { bx: current, .. } => *current = bx,
            _ => panic!("only Abx instructions can be patched with a 16-bit immediate"),
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
    pub const fn bx_value(self) -> Option<u16> {
        match self {
            Self::Abx { bx, .. } | Self::ProfiledAbx { bx, .. } => Some(bx),
            _ => None,
        }
    }
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
    fn ax_patch_updates_24_bit_immediate() {
        let mut inst = Instruction::ax(Opcode::Jump, 0);
        inst.patch_ax(-3);
        assert_eq!(inst.ax_value(), Some(-3));
    }

    #[test]
    fn abx_patch_updates_16_bit_immediate() {
        let mut inst = Instruction::abx(Opcode::JumpIfFalse, 4, 0);
        inst.patch_bx(0x1234);
        assert_eq!(inst.bx_value(), Some(0x1234));
    }
}
