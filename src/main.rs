use error::VMError;
use hardware::{Memory, Register, Registers};
use instructions::add;
use utils::{setup, shutdown};

mod error;
mod hardware;
mod instructions;
mod trap_routines;
mod utils;

const PC_START: u16 = 0x3000;

fn main() -> Result<(), VMError> {
    // Hardware creation
    let mut regs = Registers::new();
    let mut mem = Memory::new();
    // Load Arguments
    /* TODO! */
    // Setup
    let termios = setup()?;
    regs[Register::PC] = PC_START;

    // VM main loop
    /* TODO! */

    // Shutdown
    shutdown(termios)?;
    Ok(())
}
