use crate::instruction::OPCODE_SUBTYPE_MASK;
use num_enum::TryFromPrimitive;

fn neg(n: u8) -> u8 {
    (!n).wrapping_add(1)
}

fn u8_multiply(n1: u8, n2: u8) -> (u8, u8) {
    let product = n1 as u16 * n2 as u16;
    let low = (product & 0b00001111) as u8;
    let high = (product >> 4) as u8;
    (low, high)
}

pub fn alu(opcode: u8, n1: u8, n2: u8) -> AluOutput {
    let low3 = opcode & 0b111;
    let alu_opcode = AluOpcode::try_from(low3).unwrap() /* already coerced by taking the low 3 bits*/;
    let mut carry = false;
    let out = match alu_opcode {
        AluOpcode::Add => {
            let x = n1.carrying_add(n2, false);
            carry = x.1;
            x.0
        }
        AluOpcode::Sub => {
            let x = n1.carrying_add(neg(n2), false);
            carry = x.1;
            x.0
        }
        AluOpcode::And => n1 & n2,
        AluOpcode::Or => n1 | n2,
        AluOpcode::Not => !n1,
        AluOpcode::Xor => n1 ^ n2,
        AluOpcode::MulLow => u8_multiply(n1, n2).0,
        AluOpcode::MulHigh => u8_multiply(n1, n2).1,
    };
    AluOutput { out, carry }
}

pub struct AluOutput {
    pub out: u8,
    pub carry: bool,
}

#[repr(u8)]
#[derive(Debug, TryFromPrimitive)]
enum AluOpcode {
    Add = 0b000,
    Sub = 0b001,
    And = 0b010,
    Or = 0b011,
    Not = 0b100,
    Xor = 0b101,
    MulLow = 0b110,
    MulHigh = 0b111,
}

pub fn jump_condition(opcode: u8, n1: u8, n2: u8) -> bool {
    let cond_opcode = opcode & OPCODE_SUBTYPE_MASK;
    let not_bit = cond_opcode & 0b100 == 0b100;

    let mut out = match cond_opcode & 0b011 {
        0b000 => false,
        0b001 => n1 == n2,
        0b010 => n1 < n2,
        0b011 => n1 <= n2,
        _ => unreachable!(),
    };
    if not_bit {
        out = !out;
    }

    out
}

pub fn shift(opcode: u8, n1: u8, n2: u8) -> u8 {
    match opcode & OPCODE_SUBTYPE_MASK {
        0b000 => {
            // shl
            n1 << n2
        }
        0b001 => {
            // shr
            n1 >> n2
        }
        0b010 => {
            // wrapping shl
            n1.wrapping_shl(n2 as u32)
        }
        0b011 => {
            // wrapping shr
            n1.wrapping_shr(n2 as u32)
        }
        _ => n1,
    }
}
