use crate::Opcode;

/// Fixed-width 4-byte instruction forms reserved by the Phase 4 bytecode contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Instruction {
    Abc { opcode: Opcode, a: u8, b: u8, c: u8 },
    Abx { opcode: Opcode, a: u8, bx: u16 },
    Ax { opcode: Opcode, ax: i32 },
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
    pub const fn opcode(self) -> Opcode {
        match self {
            Self::Abc { opcode, .. } | Self::Abx { opcode, .. } | Self::Ax { opcode, .. } => opcode,
        }
    }

    #[inline]
    pub fn encode_word(self) -> u32 {
        let bytes = match self {
            Self::Abc { opcode, a, b, c } => [opcode as u8, a, b, c],
            Self::Abx { opcode, a, bx } => {
                let bx = bx.to_le_bytes();
                [opcode as u8, a, bx[0], bx[1]]
            }
            Self::Ax { opcode, ax } => {
                let raw = ax & 0x00ff_ffff;
                let raw = raw.to_le_bytes();
                [opcode as u8, raw[0], raw[1], raw[2]]
            }
        };

        u32::from_le_bytes(bytes)
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
            Self::Abx { bx, .. } => Some(bx),
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
