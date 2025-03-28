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

/// Abstraction of the registers storage.
struct Registers {
    inner: [Register; REGS_COUNT],
}

/// Opcodes that identify an operation
/// thet the VM supports.
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
