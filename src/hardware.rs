<<<<<<< HEAD
use std::ops::{Index, IndexMut};

const MEMORY_MAX: u16 = u16::MAX;
=======
const MEMORY_MAX: usize = 65535;
>>>>>>> hardware_abstractions
const REGS_COUNT: usize = 10;

/// Abstraction of the memory.
/// It has 65,536 memory locations.
struct Memory {
    inner: [u16; MEMORY_MAX],
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
    /// Allows to transform an u16 into a corresponding Register
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
    /// Creates a new instance of Registers with all the values of the registers
    /// set to 0
    pub fn new() -> Self {
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
    Br = 0,
    Add = 1,
    Ld = 2,
    St = 3,
    Jsr = 4,
    And = 5,
    Ldr = 6,
    Str = 7,
    Rti = 8,
    Not = 9,
    Ldi = 10,
    Sti = 11,
    Jmp = 12,
    Res = 13,
    Lea = 14,
    Trap = 15,
}

/// Condition flags that indicate
/// the result of the previous calculation
enum CondFlag {
    Pos = 1 << 0,
    Zro = 1 << 1,
    Neg = 1 << 2,
}

/// Registers that are located on the memory
/// - KBSR = Keyboard status
/// - KBDR = Keyboard data
#[derive(Clone, Copy)]
enum MemoryRegisters {
    KeyboardStatus,
    KeyboardData 
}

impl MemoryRegisters {
    fn address(&self) -> u16 {
        match self {
            MemoryRegisters::KeyboardStatus => 0xFE00,
            MemoryRegisters::KeyboardData => 0xFE02,
        }
    }
}

/// Allows us to compare an u16 with a MemoryRegister
impl PartialEq<MemoryRegisters> for u16 {
    fn eq(&self, mem_reg: &MemoryRegisters) -> bool {
        *self == mem_reg.address()
    }
}

/// Allows us to compare a MemoryRegister with an u16
impl PartialEq<u16> for MemoryRegisters {
    fn eq(&self, num: &u16) -> bool {
        self.address() == *num
    }
}
