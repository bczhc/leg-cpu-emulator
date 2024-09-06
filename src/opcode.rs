use crate::DIGITS;
use std::str::FromStr;
use strum_macros::EnumString;

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
    /* Divisions */
    Div = 0b00011000,
    Mod = 0b00011001,
    /* Miscellaneous */
    CopyStatic = 0b00000001,
    Halt = 0b00000010,
    #[strum(serialize = "cp")]
    Copy = 0b00000011,
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
