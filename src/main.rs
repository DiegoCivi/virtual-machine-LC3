use error::VMError;
use hardware::{CondFlag, Memory, OpCode, Register, Registers};
use instructions::*;
use std::env;
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
    println!("Load arguments");
    load_arguments(&mut args, &mut mem).unwrap();
    // Setup
    println!("Setup");
    setup().unwrap();

    // The COND register needs to be initialized at something
    regs[Register::Cond] = CondFlag::Zro.value();
    // Set the PC to the default val
    regs[Register::PC] = PC_START;
    // println!("Entro al loop");
    let mut running_flag = true;
    while running_flag {
        // println!("loop");
        // Get the instruction to execute
        let instr_addr = regs[Register::PC];
        regs[Register::PC] = regs[Register::PC].wrapping_add(1);
        let instr = mem.read(instr_addr).unwrap();
        println!("instr: {:x}", instr);
        let op_code = OpCode::try_from(instr >> 12).unwrap();
        // println!("{:x} => {:?}", instr, op_code);

        match op_code {
            OpCode::Br => branch(instr, &mut regs).unwrap(),
            OpCode::Add => add(instr, &mut regs).unwrap(),
            OpCode::Ld => load(instr, &mut regs, &mut mem).unwrap(),
            OpCode::St => store(instr, &mut regs, &mut mem).unwrap(),
            OpCode::Jsr => jump_register(instr, &mut regs).unwrap(),
            OpCode::And => and(instr, &mut regs).unwrap(),
            OpCode::Ldr => load_register(instr, &mut regs, &mut mem).unwrap(),
            OpCode::Str => store_register(instr, &mut regs, &mut mem).unwrap(),
            OpCode::Not => not(instr, &mut regs).unwrap(),
            OpCode::Ldi => load_indirect(instr, &mut regs, &mut mem).unwrap(),
            OpCode::Sti => store_indirect(instr, &mut regs, &mut mem).unwrap(),
            OpCode::Jmp => jump(instr, &mut regs).unwrap(),
            OpCode::Lea => load_effective_address(instr, &mut regs).unwrap(),
            OpCode::Trap => trap(instr, &mut regs, &mut mem, &mut running_flag).unwrap(),
        }
        // println!();
        // println!();
        // println!();

    }

    // Shutdown
    shutdown().unwrap();
    Ok(())
}
