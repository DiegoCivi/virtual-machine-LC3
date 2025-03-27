use std::ops::{Index, IndexMut};

const MEMORY_MAX: u16 = u16::MAX;
const REGS_COUNT: usize = 10;

/// Abstraction of the memory.
/// It has 65,536 memory locations.
struct Memory {
    inner: [u16; MEMORY_MAX as usize], // TODO: Check if MEMORY_MAX should be an usize
}

/// Abstraction of a single register.
/// We have:
/// - 8 general purpose registers (R0-R7)
/// - 1 program counter register (PC)
/// - 1 condition flags register (COND)
#[derive(Clone, Copy)]
pub enum Register {
    R0 = 0,
    R1 = 1,
    R2 = 2,
    R3 = 3,
    R4 = 4,
    R5 = 5,
    R6 = 6,
    R7 = 7,
    PC = 8,
    COND = 9,
}

impl From<u16> for Register {
    fn from(value: u16) -> Self {
        match value {
            0 => Self::R0,
            1 => Self::R1,
            2 => Self::R2,
            3 => Self::R3,
            4 => Self::R4,
            5 => Self::R5,
            6 => Self::R6,
            7 => Self::R7,
            8 => Self::PC,
            9 => Self::COND,
            _ => panic!("Value out of bounds"), // TODO: Check how to handle this case
        }
    }
}

/// Abstraction of the registers storage.
pub struct Registers {
    inner: [u16; REGS_COUNT],
}

impl Registers {
    fn new() -> Self {
        Self { inner: [0; REGS_COUNT] }
    }
}

impl Index<Register> for Registers {
    type Output = u16;

    fn index(&self, reg: Register) -> &Self::Output {
        let index = reg as usize;
        &self.inner[index]
    }
}

impl IndexMut<Register> for Registers {
    fn index_mut(&mut self, reg: Register) -> &mut Self::Output {
        let index = reg as usize;
        &mut self.inner[index]
    }
}

/// Opcodes that identify an operation
/// that the VM supports.
enum OpCode {
    BR = 0,
    ADD = 1,
    LD = 2,
    ST = 3,
    JSR = 4,
    AND = 5,
    LDR = 6,
    STR = 7,
    RTI = 8,
    NOT = 9,
    LDI = 10,
    STI = 11,
    JMP = 12,
    RES = 13,
    LEA = 14,
    TRAP = 15,
}

/// Condition flags that indicate
/// the result of the previous calculation
pub enum CondFlag {
    POS = 1 << 0,
    ZRO = 1 << 1,
    NEG = 1 << 2,
}

