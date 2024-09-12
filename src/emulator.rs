use crate::assembler::INST_LENGTH;
use crate::components;
use crate::components::jump_condition;
use crate::instruction::{
    Opcode, OpcodeType, COPY_STATIC_HEADER, OPCODE_SUBTYPE_MASK, OPCODE_TYPE_MASK,
};
use anyhow::anyhow;
use num_traits::{AsPrimitive, WrappingAdd};
use std::hint;
use std::ops::{Add, AddAssign, Deref};
use yeet_ops::yeet;

#[derive(Default, Debug)]
pub struct Emulator {
    program: Vec<u8>,
    pc: WrappingNum<u16>,
    ram: Vec<u8>,
    stack: Vec<u8>,
    f_call_stack: Vec<u16>,
    f_args_stack: Vec<u8>,
    registers: Registers,
}

#[derive(Debug)]
struct Registers {
    /// The 16 registers, represented as a 4-bit number in the operand byte.
    ///
    /// Tier1 registers can be set via `cp`, `st`, `ld`, `add` etc.
    ///
    /// Note: this is just the place allocated for them, do not fetch it
    /// directly via `[]` indexing, instead, for read/write operations,
    /// use [`Registers::fetch`] and [`Registers::write`].
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
    fn fetch(&self, reg: u8) -> u8 {
        match reg {
            // r0 to r11
            _ if reg <= 11 => self.tier1[reg as usize],
            // in/out
            12 => {
                0 /* TODO */
            }
            // always one
            13 => 1,
            // always zero
            14 => 0,
            // function stack start
            15 => self.tier1[reg as usize],
            // do not handle
            _ => 0,
        }
    }

    fn write(&mut self, reg: u8, n: u8) {
        match reg {
            _ if reg <= 11 || reg == 15 => {
                self.tier1[reg as usize] = n;
            }
            12 => { /* TODO */ }
            // writing to always x registers takes no effect
            13 => {}
            14 => {}
            // do not handle
            _ => {}
        }
    }

    fn carry_u8(&self) -> u8 {
        self.carry.into()
    }
}

impl Emulator {
    pub fn new(binary: Vec<u8>) -> anyhow::Result<Self> {
        let mut emulator = Self {
            program: binary,
            pc: 0.into(),
            ram: vec![0; u8::MAX as usize],
            stack: vec![0; u8::MAX as usize],
            f_call_stack: vec![0; u8::MAX as usize],
            f_args_stack: vec![0; u8::MAX as usize],
            registers: Registers::default(),
        };
        emulator.parse_header()?;
        Ok(emulator)
    }

    fn parse_header(&mut self) -> anyhow::Result<()> {
        let header = &self.program[..4];
        if header[0] != COPY_STATIC_HEADER {
            yeet!(anyhow!("Invalid header: {:?}", header));
        }

        let data_len = header[1] as usize;
        let mem_start = header[0] as usize;
        let entrypoint: u16 = header[2] as u16;

        let static_data = &self.program[4..(4 + data_len)];
        self.ram[mem_start..(mem_start + data_len)].copy_from_slice(static_data);
        self.pc = entrypoint.into();

        Ok(())
    }

    pub fn tick(&mut self) -> anyhow::Result<Option<Output>> {
        let inst = &self.program[self.pc.usize()..(self.pc.usize() + INST_LENGTH as usize)];

        macro end_not_add_pc() {{
            return Ok(None); /* TODO */
        }}

        macro end() {{
            self.pc += INST_LENGTH as u16;
            return end_not_add_pc!();
        }}

        let opcode_u8 = inst[0];
        let Ok(opcode) = Opcode::try_from(opcode_u8) else {
            yeet!(anyhow!("Unknown opcode: 0x{:02x?}", inst[0]))
        };

        // immediate number masks
        let imm1 = inst[0] & 0b10000000 == 0b10000000;
        let imm2 = inst[0] & 0b01000000 == 0b01000000;

        // just skip unknown instructions
        let Ok(opcode_type) = OpcodeType::try_from(opcode_u8 & OPCODE_TYPE_MASK) else {
            end!()
        };

        macro get_operand($imm:expr, $inst_index:expr) {
            if $imm {
                inst[$inst_index]
            } else {
                let Ok(reg) = crate::instruction::OperandSymbol::try_from(inst[$inst_index]) else {
                    end!()
                };
                self.registers.fetch(reg as u8)
            }
        }
        let operand1 = get_operand!(imm1, 1);
        let operand2 = get_operand!(imm2, 2);

        let opcode_subtype = opcode_u8 & OPCODE_SUBTYPE_MASK;

        match opcode_type {
            OpcodeType::Compute => {
                let out = components::alu(opcode_u8, operand1, operand2);
                self.registers.write(inst[3], out.out);
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
                        self.registers.write(inst[2], v);
                    }
                    0b001 => {
                        // store
                        let value = self.registers.fetch(inst[2]);
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
                        self.registers.write(inst[1], value);
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
                        self.registers.write(inst[1], value);
                    }
                    _ => {}
                }
            }
            OpcodeType::Shifts => {
                let out = components::shift(opcode_u8, operand1, operand2);
                self.registers.write(inst[3], out);
            }
            OpcodeType::ArithmeticSupplementary => {
                match opcode_subtype {
                    0b000 => {
                        // div
                        self.registers.write(inst[3], operand1 / operand2);
                    }
                    0b001 => {
                        // mod
                        self.registers.write(inst[3], operand1 % operand2);
                    }
                    0b010 => {
                        // carry-add
                        let value = operand1
                            .wrapping_add(operand2)
                            .wrapping_add(self.registers.carry_u8());
                        self.registers.write(inst[3], value);
                        // also set the carry bit
                    }
                    0b011 => {
                        // add-no-carry
                        let value = operand1.wrapping_add(operand2);
                        self.registers.write(inst[3], value);
                    }
                    0b100 => {
                        // sub-no-carry
                        let value = operand1.wrapping_sub(operand2);
                        self.registers.write(inst[3], value);
                    }
                    0b101 => {
                        // move-carry
                        let value = self.registers.carry_u8();
                        self.registers.write(inst[3], value);
                    }
                    _ => {}
                }
            }
            OpcodeType::Miscellaneous => {
                match opcode_subtype {
                    0b010 => {
                        // halt
                        // TODO: maybe a more elegant handling method
                        println!("Halt!");
                        loop {
                            hint::spin_loop()
                        }
                    }
                    0b011 => {
                        // copy
                        self.registers.write(inst[3], operand1);
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
}

#[repr(transparent)]
pub struct Output(u8);

impl From<u8> for Output {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
#[repr(transparent)]
struct WrappingNum<T>(T)
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
