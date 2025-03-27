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
enum CondFlag {
    POS = 1 << 0,
    ZRO = 1 << 1,
    NEG = 1 << 2,
}

