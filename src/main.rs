use hardware::{CondFlag, Memory, Register, Registers};
use instructions::add;
use memory_access::mem_read;

mod hardware;
mod utils;
mod instructions;
mod error;
mod memory_access;

fn main() {
    // Load Arguments
    /* TODO! */
    // Setup
    /* TODO! */

    // Hardware creation
    let mut regs = Registers::new();
    let mut mem = Memory::new();
    // The COND register needs to be initialized at something
    regs[Register::Cond] = CondFlag::Zro.value();
    // Set the PC to the default val
    regs[Register::PC] = 0x3000;

    let running = true;
    while running {
        // Get the instruction to execute
        let instr_addr = regs[Register::PC];
        let instr = mem_read(instr_addr, &mut mem);
        let op_code = instr_addr >> 12;
    }

}
