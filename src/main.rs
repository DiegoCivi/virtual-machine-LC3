use std::env;

use error::VMError;
use utils::{setup, shutdown};
use vm::VM;

mod error;
mod hardware;
mod trap_code;
mod utils;
mod vm;

fn main() -> Result<(), VMError> {
    let mut args = env::args();
    // Virtual Machine creation
    let mut vm = VM::new();
    // Read the file with the instructions to execute into the VM's memory
    vm.load_arguments(&mut args)?;
    // Setup of Terminal
    let termios = setup()?;

    // VM main loop
    vm.run()?;

    // Reset the terminal to its original settings
    shutdown(termios)?;
    Ok(())
}
