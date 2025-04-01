use std::{
    io::stdin,
    ops::{Index, IndexMut},
};

use crate::{
    error::VMError,
    utils::{check_key, getchar},
};

const MEMORY_MAX: usize = 65535;
const REGS_COUNT: usize = 10;

/// Abstraction of the memory.
/// It has 65,536 memory locations.
pub struct Memory {
    inner: [u16; MEMORY_MAX],
}

impl Memory {
    pub fn new() -> Self {
        Self {
            inner: [0; MEMORY_MAX],
        }
    }

    /// Sets a new val the memory address
    ///
    /// ### Arguments
    ///
    /// - `address`: An u16 representing the memory address to read.
    /// - `Memory`: A Mmeory struct that handles the memory in the system
    ///
    /// ### Returns
    ///
    /// A Result indicating whether the operation failed or not
    pub fn write<T: Into<usize>>(&mut self, mem_address: T, new_val: u16) -> Result<(), VMError> {
        let index: usize = mem_address.into();
        if let Some(val) = self.inner.get_mut(index) {
            *val = new_val;
            return Ok(());
        }
        Err(VMError::InvalidIndex)
    }

    /// Reads a memory address. Also checks whether a key was pressed and it
    /// that case it stores which key was pressed.
    ///
    /// ### Arguments
    ///
    /// - `address`: An u16 representing the memory address to read.
    /// - `Memory`: A Mmeory struct that handles the memory in the system
    ///
    /// ### Returns
    ///
    /// A Result containing the data in the memory address, or a VMError if
    /// the operation failed
    pub fn read(&mut self, addr: u16) -> Result<u16, VMError> {
        if addr == MemoryRegister::KeyboardStatus {
            if check_key() {
                self.write(MemoryRegister::KeyboardStatus, 1 << 15)?;
                let mut reader = stdin();
                let buffer = getchar(&mut reader)?;
                let char: u16 = buffer[0].into();
                self.write(MemoryRegister::KeyboardData, char)?;
            } else {
                self.write(MemoryRegister::KeyboardStatus, 0)?;
            }
        }
        // Get the value
        let index: usize = addr.into();
        if let Some(val) = self.inner.get(index) {
            return Ok(*val);
        }
        Err(VMError::InvalidIndex)
    }
}

/// Abstraction of a single register.
/// We have:
/// - 8 general purpose registers (R0-R7)
/// - 1 program counter register (PC)
/// - 1 condition flags register (COND)
#[derive(Clone, Copy)]
pub enum Register {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    PC,
    Cond,
}

impl Register {
    fn index(&self) -> usize {
        match self {
            Register::R0 => 0,
            Register::R1 => 1,
            Register::R2 => 2,
            Register::R3 => 3,
            Register::R4 => 4,
            Register::R5 => 5,
            Register::R6 => 6,
            Register::R7 => 7,
            Register::PC => 8,
            Register::Cond => 9,
        }
    }

    pub fn from_u16(n: u16) -> Result<Self, VMError> {
        match n {
            0 => Ok(Register::R0),
            1 => Ok(Register::R1),
            2 => Ok(Register::R2),
            3 => Ok(Register::R3),
            4 => Ok(Register::R4),
            5 => Ok(Register::R5),
            6 => Ok(Register::R6),
            7 => Ok(Register::R7),
            8 => Ok(Register::PC),
            9 => Ok(Register::Cond),
            _ => Err(VMError::Conversion),
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
        Self {
            inner: [0; REGS_COUNT],
        }
    }
}

impl Index<Register> for Registers {
    type Output = u16;

    fn index(&self, reg: Register) -> &Self::Output {
        let index = reg.index();
        &self.inner[index]
    }
}

impl IndexMut<Register> for Registers {
    fn index_mut(&mut self, reg: Register) -> &mut Self::Output {
        let index = reg.index();
        &mut self.inner[index]
    }
}

/// Opcodes that identify an operation
/// that the VM supports.
pub enum OpCode {
    Br = 0x0000,
    Add = 0x0001,
    Ld = 0x0010,
    St = 0x0011,
    Jsr = 0x0100,
    And = 0x0101,
    Ldr = 0x0110,
    Str = 0x0111,
    // Rti, Unused op code
    Not = 0x1001,
    Ldi = 0x1010,
    Sti = 0x1011,
    Jmp = 0x1100,
    // Res, Unused op code
    Lea = 0x1110,
    Trap = 0x1111,
}

impl TryFrom<u16> for OpCode {
    type Error = VMError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0x0000 => Ok(OpCode::Br),
            0x0001 => Ok(OpCode::Add),
            0x0010 => Ok(OpCode::Ld),
            0x0011 => Ok(OpCode::St),
            0x0100 => Ok(OpCode::Jsr),
            0x0101 => Ok(OpCode::And),
            0x0110 => Ok(OpCode::Ldr),
            0x0111 => Ok(OpCode::Str),
            0x1001 => Ok(OpCode::Not),
            0x1010 => Ok(OpCode::Ldi),
            0x1011 => Ok(OpCode::Sti),
            0x1100 => Ok(OpCode::Jmp),
            0x1110 => Ok(OpCode::Lea),
            0x1111 => Ok(OpCode::Trap),
            _ => Err(VMError::Conversion),
        }
    }
}

/// Condition flags that indicate
/// the result of the previous calculation
pub enum CondFlag {
    Pos = 1 << 0,
    Zro = 1 << 1,
    Neg = 1 << 2,
}

impl CondFlag {
    pub fn value(&self) -> u16 {
        match self {
            CondFlag::Pos => 1 << 0,
            CondFlag::Zro => 1 << 1,
            CondFlag::Neg => 1 << 2,
        }
    }
}

/// Registers that are located on the memory
#[derive(Clone, Copy)]
pub enum MemoryRegister {
    KeyboardStatus,
    KeyboardData,
}

impl MemoryRegister {
    fn address(&self) -> u16 {
        match self {
            MemoryRegister::KeyboardStatus => 0xFE00,
            MemoryRegister::KeyboardData => 0xFE02,
        }
    }
}

impl From<MemoryRegister> for usize {
    fn from(value: MemoryRegister) -> Self {
        value.address().into()
    }
}

/// Allows us to compare an u16 with a MemoryRegister
impl PartialEq<MemoryRegister> for u16 {
    fn eq(&self, mem_reg: &MemoryRegister) -> bool {
        *self == mem_reg.address()
    }
}

/// Allows us to compare a MemoryRegister with an u16
impl PartialEq<u16> for MemoryRegister {
    fn eq(&self, num: &u16) -> bool {
        self.address() == *num
    }
}
