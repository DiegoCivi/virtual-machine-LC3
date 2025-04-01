use std::{env, net::Shutdown};

use error::VMError;
use hardware::{CondFlag, Memory, OpCode, Register, Registers};
use instructions::*;
use utils::load_arguments;

mod error;
mod hardware;
mod instructions;
mod trap_routines;
mod utils;

fn main() -> Result<(), VMError> {
    let args = env::args();
    // Hardware creation
    let mut regs = Registers::new();
    let mut mem = Memory::new();
    // Load Arguments
    /* TODO! */
    load_arguments(args, &mut mem)?;
    // Setup
    /* TODO! */

    // The COND register needs to be initialized at something
    regs[Register::Cond] = CondFlag::Zro.value();
    // Set the PC to the default val
    regs[Register::PC] = 0x3000;

    let mut running_flag = true;
    while running_flag {
        // Get the instruction to execute
        let instr_addr = regs[Register::PC].wrapping_add(1);
        let instr = mem.read(instr_addr)?;
        let op_code = OpCode::try_from(instr >> 12)?;

        match op_code {
            OpCode::Br => branch(instr, &mut regs)?,
            OpCode::Add => add(instr, &mut regs)?,
            OpCode::Ld => load(instr, &mut regs, &mut mem)?,
            OpCode::St => store(instr, &mut regs, &mut mem)?,
            OpCode::Jsr => jump_register(instr, &mut regs)?,
            OpCode::And => and(instr, &mut regs)?,
            OpCode::Ldr => load_register(instr, &mut regs, &mut mem)?,
            OpCode::Str => store_register(instr, &mut regs, &mut mem)?,
            OpCode::Not => not(instr, &mut regs)?,
            OpCode::Ldi => load_indirect(instr, &mut regs, &mut mem)?,
            OpCode::Sti => store_indirect(instr, &mut regs, &mut mem)?,
            OpCode::Jmp => jump(instr, &mut regs)?,
            OpCode::Lea => load_effective_address(instr, &mut regs)?,
            OpCode::Trap => trap(instr, &mut regs, &mut mem, &mut running_flag)?,
        }
    }

    // Shutdown
    /* TODO! */
    Ok(())
}
