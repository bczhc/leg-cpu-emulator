use std::cell::RefCell;
use crate::assembler::INST_LENGTH;
use crate::components;
use crate::components::jump_condition;
use crate::instruction::{
    Opcode, OpcodeType, COPY_STATIC_HEADER, OPCODE_SUBTYPE_MASK, OPCODE_TYPE_MASK,
};
use anyhow::anyhow;
use num_traits::{AsPrimitive, WrappingAdd};
use std::ops::{AddAssign, Deref};
use yeet_ops::yeet;

#[derive(Default, Debug)]
pub struct Emulator {
    pub program: Vec<u8>,
    pub pc: WrappingNum<u16>,
    pub ram: Vec<u8>,
    pub stack: Vec<u8>,
    pub f_call_stack: Vec<u16>,
    pub f_args_stack: Vec<u8>,
    pub registers: Registers,
    pub halted: bool,
    pub output: Option<Output>,
    pub input: RefCell<Vec<u8>>,
}

#[derive(Debug)]
pub struct Registers {
    /// The 16 registers, represented as a 4-bit number in the operand byte.
    ///
    /// Tier1 registers can be set via `cp`, `st`, `ld`, `add` etc.
    ///
    /// Note: this is just the place allocated for them, and some of them
    /// may be useless. Use `tier1[regN]` to index.
    tier1: Vec<u8>,
    /// Carry flag.
    ///
    /// This can only be retrieved via `mvc`.
    carry: bool,
    /// Code-jump address. LEG supports 16bit program addressing.
    ///
    /// This can only be set via `jamv`.
    jump_address: u16,
}

impl Default for Registers {
    fn default() -> Self {
        Self {
            tier1: vec![0_u8; 16],
            carry: false,
            jump_address: 0,
        }
    }
}

impl Registers {
    fn carry_u8(&self) -> u8 {
        self.carry.into()
    }
}

impl Emulator {
    pub fn new(binary: impl Into<Vec<u8>>) -> anyhow::Result<Self> {
        let mut emulator = Self {
            program: binary.into(),
            pc: 0.into(),
            ram: vec![0; u8::MAX as usize],
            stack: vec![0; u8::MAX as usize],
            f_call_stack: vec![0; u8::MAX as usize],
            f_args_stack: vec![0; u8::MAX as usize],
            registers: Registers::default(),
            halted: false,
            output: None,
            input: vec![].into(),
        };
        emulator.parse_header()?;
        Ok(emulator)
    }

    pub fn set_input(&mut self, input: impl Into<Vec<u8>>) -> &mut Self {
        // because of taking input uses `pop`, the order is reversed
        let mut vec = input.into();
        vec.reverse();
        self.input = vec.into();
        self
    }

    fn parse_header(&mut self) -> anyhow::Result<()> {
        let header = &self.program[..4];
        if header[0] != COPY_STATIC_HEADER {
            yeet!(anyhow!("Invalid header: {:?}", header));
        }

        let data_len = header[1] as usize;
        let mem_start = header[2] as usize;
        let entrypoint: u16 = header[3] as u16;

        let static_data = &self.program[4..(4 + data_len)];
        self.ram[mem_start..(mem_start + data_len)].copy_from_slice(static_data);
        self.pc = entrypoint.into();

        Ok(())
    }

    pub fn tick(&mut self) -> anyhow::Result<()> {
        if self.halted {
            yeet!(anyhow!("CPU is halted"));
        }

        macro end_not_add_pc() {{
            return Ok(());
        }}

        macro end() {{
            self.pc += INST_LENGTH as u16;
            end_not_add_pc!();
        }}

        // every tick, reset the output.
        // output is only valid if enabled in Turing Complete
        self.output = None;

        let inst = if self.pc.usize() + INST_LENGTH as usize - 1 > self.program.len() {
            // PC goes beyond the available program area
            // this may happen if jumping to an invalid program address,
            // or program runs without a `halt`.
            // just issue [0, 0, 0, 0] if this happens.
            &NULL_INSTRUCTION
        } else {
            &self.program[self.pc.usize()..(self.pc.usize() + INST_LENGTH as usize)]
        };

        let opcode_u8 = inst[0] & 0b00111111;
        let Ok(_opcode) = Opcode::try_from(opcode_u8) else {
            // skip unknown opcodes
            end!()
        };

        // immediate number masks
        let imm1 = inst[0] & 0b10000000 == 0b10000000;
        let imm2 = inst[0] & 0b01000000 == 0b01000000;

        // just skip unknown instructions
        let Ok(opcode_type) = OpcodeType::try_from((opcode_u8 & OPCODE_TYPE_MASK) >> 3) else {
            end!()
        };

        macro get_operand($imm:expr, $inst_index:expr) {
            if $imm {
                inst[$inst_index]
            } else {
                let Ok(reg) = crate::instruction::OperandSymbol::try_from(inst[$inst_index]) else {
                    end!()
                };
                self.reg_fetch(reg as u8)
            }
        }
        let operand1 = get_operand!(imm1, 1);
        let operand2 = get_operand!(imm2, 2);

        let opcode_subtype = opcode_u8 & OPCODE_SUBTYPE_MASK;

        match opcode_type {
            OpcodeType::Compute => {
                let out = components::alu(opcode_u8, operand1, operand2);
                self.reg_write(inst[3], out.out);
                self.registers.carry = out.carry;
            }
            OpcodeType::ConditionalJumping => {
                let condition = jump_condition(opcode_u8, operand1, operand2);
                if condition {
                    // jump to the value of jump-address register
                    self.pc = self.registers.jump_address.into();
                    end_not_add_pc!();
                }
            }
            OpcodeType::Memory => {
                match opcode_subtype {
                    0b000 => {
                        // load
                        let v = self.ram[operand1 as usize];
                        self.reg_write(inst[2], v);
                    }
                    0b001 => {
                        // store
                        let value = self.reg_fetch(inst[2]);
                        self.ram[operand1 as usize] = value;
                    }
                    _ => {}
                }
            }
            OpcodeType::Stack => {
                match opcode_subtype {
                    0b000 => {
                        // push
                        self.stack.push(operand1);
                    }
                    0b001 => {
                        // pop
                        let value = self.stack.pop().unwrap_or_default();
                        self.reg_write(inst[1], value);
                    }
                    _ => {}
                }
            }
            OpcodeType::Functions => {
                match opcode_subtype {
                    0b000 => {
                        // call
                        // push the address of the next instruction (known as the return address)
                        self.f_call_stack.push(*self.pc + 4);
                        let call_addr = u16::from_le_bytes([inst[2], inst[3]]);
                        // jump to function
                        self.pc = call_addr.into();
                        end_not_add_pc!();
                    }
                    0b001 => {
                        // return
                        // pop the return-address and set the PC
                        let addr = self.f_call_stack.pop().unwrap_or_default();
                        self.pc = addr.into();
                        end_not_add_pc!();
                    }
                    0b010 => {
                        // fpush
                        self.f_args_stack.push(operand1);
                    }
                    0b011 => {
                        // fpop
                        let value = self.f_args_stack.pop().unwrap_or_default();
                        self.reg_write(inst[1], value);
                    }
                    _ => {}
                }
            }
            OpcodeType::Shifts => {
                let out = components::shift(opcode_u8, operand1, operand2);
                self.reg_write(inst[3], out);
            }
            OpcodeType::ArithmeticSupplementary => {
                match opcode_subtype {
                    0b000 => {
                        // div
                        self.reg_write(inst[3], operand1 / operand2);
                    }
                    0b001 => {
                        // mod
                        self.reg_write(inst[3], operand1 % operand2);
                    }
                    0b010 => {
                        // carry-add
                        let (r1, c1) = operand1.carrying_add(operand2, false);
                        let (r2, c2) = r1.carrying_add(self.registers.carry_u8(), false);
                        self.reg_write(inst[3], r2);
                        // also set the carry bit
                        self.registers.carry = c1 || c2;
                    }
                    0b011 => {
                        // add-no-carry
                        let value = operand1.wrapping_add(operand2);
                        self.reg_write(inst[3], value);
                    }
                    0b100 => {
                        // sub-no-carry
                        let value = operand1.wrapping_sub(operand2);
                        self.reg_write(inst[3], value);
                    }
                    0b101 => {
                        // move-carry
                        let value = self.registers.carry_u8();
                        self.reg_write(inst[3], value);
                    }
                    _ => {}
                }
            }
            OpcodeType::Miscellaneous => {
                match opcode_subtype {
                    0b010 => {
                        // halt
                        self.halted = true;
                    }
                    0b011 => {
                        // copy
                        self.reg_write(inst[3], operand1);
                    }
                    0b100 => {
                        // jump-address move
                        let addr = u16::from_le_bytes([inst[2], inst[3]]);
                        self.registers.jump_address = addr;
                    }
                    0b101 => {
                        // no-op
                    }
                    _ => {}
                }
            }
        };

        end!()
    }

    fn reg_fetch(&self, reg: u8) -> u8 {
        match reg {
            // r0 to r11
            _ if reg <= 11 => self.registers.tier1[reg as usize],
            12 => {
                // read input
                self.input.borrow_mut().pop().unwrap_or(0)
            }
            // always one
            13 => 1,
            // always zero
            14 => 0,
            // function stack start
            15 => self.registers.tier1[reg as usize],
            // do not handle
            _ => 0,
        }
    }

    fn reg_write(&mut self, reg: u8, n: u8) {
        match reg {
            _ if reg <= 11 || reg == 15 => {
                self.registers.tier1[reg as usize] = n;
            }
            12 => {
                // output
                self.output = Some(n.into());
            }
            // writing to always x registers takes no effect
            13 => {}
            14 => {}
            // do not handle
            _ => {}
        }
    }

    pub fn run_to_halt(&mut self) -> anyhow::Result<Vec<u8>> {
        let mut output = Vec::new();
        loop {
            self.tick()?;
            if self.halted {
                break;
            }
            if let Some(x) = self.output {
                output.push(*x);
            }
        }
        Ok(output)
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct Output(u8);

impl Deref for Output {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u8> for Output {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct WrappingNum<T>(T)
where
    T: WrappingAdd;

impl<T> AddAssign<T> for WrappingNum<T>
where
    T: WrappingAdd,
{
    fn add_assign(&mut self, rhs: T) {
        self.0 = self.0.wrapping_add(&rhs);
    }
}

impl<T> Default for WrappingNum<T>
where
    T: WrappingAdd + Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T> Deref for WrappingNum<T>
where
    T: WrappingAdd,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> From<T> for WrappingNum<T>
where
    T: WrappingAdd,
{
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> WrappingNum<T>
where
    T: AsPrimitive<usize> + WrappingAdd,
{
    pub fn usize(&self) -> usize {
        self.0.as_()
    }
}

pub static NULL_INSTRUCTION: [u8; 4] = [0, 0, 0, 0];

#[cfg(test)]
mod test {
    use crate::emulator::{Emulator, WrappingNum};

    #[test]
    fn wrapping_add_assign() {
        let mut a = WrappingNum(0_u8);
        a += 255;
        assert_eq!(*a, 255);
        a += 2;
        assert_eq!(*a, 1);
    }

    #[test]
    fn size() {
        println!("{}", size_of::<Emulator>());
    }
}
