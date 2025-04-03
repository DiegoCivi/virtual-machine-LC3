use std::env;

use error::VMError;
use hardware::{CondFlag, Memory, OpCode, Register, Registers};
use instructions::*;
use utils::{load_arguments, setup, shutdown};

mod error;
mod hardware;
mod instructions;
mod trap_routines;
mod utils;

const PC_START: u16 = 0x3000;

fn main() -> Result<(), VMError> {
    let mut args = env::args();
    // Hardware creation
    let mut regs = Registers::new();
    let mut mem = Memory::new();
    // Load Arguments
    load_arguments(&mut args, &mut mem)?;
    // Setup
    let termios = setup()?;
    regs[Register::Cond] = CondFlag::Zro.value();
    regs[Register::PC] = PC_START;

    // VM main loop
    let mut running_flag = true;
    while running_flag {
        let instr_addr = regs[Register::PC];
        regs[Register::PC] = regs[Register::PC].wrapping_add(1);
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
    shutdown(termios)?;
    Ok(())
}
