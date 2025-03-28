use hardware::{Register, Registers};
use instructions::add;

mod hardware;
mod utils;
mod instructions;
mod error;

fn main() {
    let sr1 = 0x0001;
    // Create the registers and set the value on R1
    let mut registers = Registers::new();
    registers[Register::R1] = sr1;
    // The instruction will have the following encoding:
    // 0 0 0 1 0 0 0 0 0 1 1 0 0 0 1 0
    let instr = 0x1062;
    let _ = add(instr, &mut registers);

}
