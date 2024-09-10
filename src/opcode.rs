use crate::DIGITS;
use anyhow::anyhow;
use std::str::FromStr;
use strum_macros::EnumString;

pub const COPY_STATIC_HEADER: u8 = 0b00000001;

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum Opcode {
    /* Compute */
    Add = 0b00001000,
    Sub = 0b00001001,
    And = 0b00001010,
    Or = 0b00001011,
    Not = 0b00001100,
    Xor = 0b00001101,
    #[strum(serialize = "mull")]
    MulLow = 0b00001110, /* unused */
    #[strum(serialize = "mulh")]
    MulHigh = 0b00001111, /* unused */
    /* Conditional jumping */
    JpEq = 0b00100001,
    JpGe = 0b00100110,
    JpGt = 0b00100111,
    JpLe = 0b00100011,
    JpLt = 0b00100010,
    JpNe = 0b00100101,
    Jp = 0b00100100,
    /* Memory */
    #[strum(serialize = "ld")]
    Load = 0b00101000,
    #[strum(serialize = "st")]
    Store = 0b00101001,
    /* Stack */
    Push = 0b00110000,
    Pop = 0b00110001,
    /* Functions */
    Call = 0b00111000,
    #[strum(serialize = "ret")]
    Return = 0b00111001,
    FPush = 0b00111010,
    FPop = 0b00111011,
    /* Shifts */
    Shl = 0b00010000,
    Shr = 0b00010001,
    /// Wrapping shift left
    WShl = 0b00010010,
    /// Wrapping shift right
    WShr = 0b00010011,
    /* Arithmetic Supplementary */
    Div = 0b00011000,
    Mod = 0b00011001,
    CAdd = 0b00011010,
    /* Miscellaneous */
    Halt = 0b00000010,
    #[strum(serialize = "cp")]
    Copy = 0b00000011,
    #[strum(serialize = "jamv")]
    JumpAddrMove = 0b00000100,
    Nop = 0b00000101,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum OperandSymbol {
    R0 = 0,
    R1 = 1,
    R2 = 2,
    R3 = 3,
    R4 = 4,
    R5 = 5,
    Pc = 6,
    #[strum(serialize = "in", serialize = "out")]
    InOut = 7,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Operand {
    Immediate(u8),
    Symbol(OperandSymbol),
}

impl FromStr for Operand {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with(DIGITS) {
            Ok(Self::Immediate(s.parse::<u8>()?))
        } else {
            Ok(Operand::Symbol(OperandSymbol::from_str(s)?))
        }
    }
}

impl Operand {
    pub fn to_u8(self) -> u8 {
        match self {
            Operand::Immediate(x) => x,
            Operand::Symbol(x) => x as u8,
        }
    }

    pub fn is_symbol(&self) -> bool {
        match self {
            Operand::Immediate(_) => false,
            Operand::Symbol(_) => true,
        }
    }

    pub fn is_immediate(&self) -> bool {
        match self {
            Operand::Immediate(_) => true,
            Operand::Symbol(_) => false,
        }
    }
}

impl Opcode {
    /// Returns a tuple where the first is the operand count,
    /// the second is a mapping to asm code and the generated
    /// binary.
    ///
    /// Due to LEG uses fixed-length instruction, positions
    /// that no operand takes will be padded with zeros.
    ///
    /// Example:
    /// ASM code `cp 123 r1` (move u8 immediate '123' to register 1)
    /// will be processed to `<cp-opcode-binary> 123 <zero-pad> <r1-code>`
    /// namely the binary `0b10000011 123 0 0b001`.
    ///
    /// Say we have asm\[3\] = \['cp', 123, 'r1'\], and inst\[4\] = {0},
    /// the mapping is:
    ///
    /// - inst\[1\] <- asm\[1\]
    /// - inst\[2\] <- 0
    /// - inst\[3\] <- asm\[2\]
    ///
    /// So this method should return (2, \[1, 0, 2\]), positions
    /// with no operand taking should use `0`s.
    fn binary_asm_indices_mapping(&self) -> (usize, [usize; 3]) {
        match self {
            Opcode::Add => (3, [1, 2, 3]),
            Opcode::Sub => (3, [1, 2, 3]),
            Opcode::And => (3, [1, 2, 3]),
            Opcode::Or => (3, [1, 2, 3]),
            Opcode::Not => (3, [1, 2, 3]),
            Opcode::Xor => (3, [1, 2, 3]),
            Opcode::MulLow => (3, [1, 2, 3]),
            Opcode::MulHigh => (3, [1, 2, 3]),
            Opcode::JpEq => (2, [1, 2, 0]),
            Opcode::JpGe => (2, [1, 2, 0]),
            Opcode::JpGt => (2, [1, 2, 0]),
            Opcode::JpLe => (2, [1, 2, 0]),
            Opcode::JpLt => (2, [1, 2, 0]),
            Opcode::JpNe => (2, [1, 2, 0]),
            Opcode::Jp => (0, [0, 0, 0]),
            Opcode::Load => (2, [1, 2, 0]),
            Opcode::Store => (2, [1, 2, 0]),
            Opcode::Push => (1, [1, 0, 0]),
            Opcode::Pop => (1, [1, 0, 0]),
            Opcode::Call => (2, [0, 1, 2]),
            Opcode::Return => (0, [0, 0, 0]),
            Opcode::FPush => (1, [1, 0, 0]),
            Opcode::FPop => (1, [1, 0, 0]),
            Opcode::Shl => (3, [1, 2, 3]),
            Opcode::Shr => (3, [1, 2, 3]),
            Opcode::WShl => (3, [1, 2, 3]),
            Opcode::WShr => (3, [1, 2, 3]),
            Opcode::Div => (3, [1, 2, 3]),
            Opcode::Mod => (3, [1, 2, 3]),
            Opcode::Halt => (0, [0, 0, 0]),
            Opcode::Copy => (2, [1, 0, 2]),
            Opcode::JumpAddrMove => (2, [0, 1, 2]),
            Opcode::Nop => (0, [0, 0, 0]),
            Opcode::CAdd => (3, [1, 2, 3]),
        }
    }

    pub fn binary(&self, operands: &[Operand]) -> anyhow::Result<[u8; 4]> {
        let indices_mapping = self.binary_asm_indices_mapping();
        assert_eq!(
            indices_mapping.1.iter().filter(|&&x| x != 0).count(),
            indices_mapping.0
        );
        let mut inst = [0_u8; 4];

        let inst_operand = |inst_index: usize| match indices_mapping.1[inst_index - 1] {
            0 => Ok(None),
            i => operands
                .get(i - 1)
                .ok_or(anyhow!("Missing operand: index: {}", i - 1))
                .map(Some),
        };

        // opcode
        inst[0] = *self as u8;
        // operands #1, #2, #3
        for i in 1..=3 {
            inst[i] = inst_operand(i)?.map(|x| x.to_u8()).unwrap_or(0);
        }

        // add immediate masks
        let mut immediate_mask = 0b00000000_u8;
        if inst_operand(1)?.map(|x| x.is_immediate()).unwrap_or(false) {
            immediate_mask |= 0b10000000;
        }
        if inst_operand(2)?.map(|x| x.is_immediate()).unwrap_or(false) {
            immediate_mask |= 0b01000000;
        }
        inst[0] = (inst[0] & 0b00111111) | immediate_mask;

        Ok(inst)
    }
}
