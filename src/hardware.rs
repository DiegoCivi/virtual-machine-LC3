const MEMORY_MAX: usize = 65535;
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
enum Register {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    PC,
    COND,
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
            Register::COND => 9,
        }
    }
}

/// Abstraction of the registers storage.
struct Registers {
    inner: [Register; REGS_COUNT],
}

/// Opcodes that identify an operation
/// thet the VM supports.
enum OpCode {
    Br,
    Add,
    Ld,
    St,
    Jsr,
    And,
    Ldr,
    Str,
    Rti,
    Not,
    Ldi,
    Sti,
    Jmp,
    Res,
    Lea,
    Trap,
}

impl OpCode {
    fn index(&self) -> usize {
        match self {
            OpCode::Br => 0,
            OpCode::Add => 1,
            OpCode::Ld => 2,
            OpCode::St => 3,
            OpCode::Jsr => 4,
            OpCode::And => 5,
            OpCode::Ldr => 6,
            OpCode::Str => 7,
            OpCode::Rti => 8,
            OpCode::Not => 9,
            OpCode::Ldi => 10,
            OpCode::Sti => 11,
            OpCode::Jmp => 12,
            OpCode::Res => 13,
            OpCode::Lea => 14,
            OpCode::Trap => 15,
        }
    }
}

/// Condition flags that indicate
/// the result of the previous calculation
enum CondFlag {
    Pos = 1 << 0,
    Zro = 1 << 1,
    Neg = 1 << 2,
}

impl CondFlag {
    fn value(&self) -> u16 {
        match self {
            CondFlag::Pos => 1 << 0,
            CondFlag::Zro => 1 << 1,
            CondFlag::Neg => 1 << 2,
        }
    }
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
