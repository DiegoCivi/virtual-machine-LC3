use crate::{hardware::{Register, Registers}, utils::{mem_read, sign_extend, update_flags}};

/// Adds to values
/// 
/// This instruction can happen in two different ways. One is
/// by adding two elements that are in a register each, this is called
/// register mode. The other mode is to add the value of a register
/// with the one that is embedded in the instruction itself.
/// 
/// ### Arguments
/// 
/// - `instr`: An u16 that has the encoding of the whole instruction to execute.
/// - `regs`: A Registers struct that handles each register.
fn add(instr: u16, regs: &mut Registers) {
    // Destination register
    let dr = Register::from((instr >> 9) & 0x7);
    // First operand
    let r1: Register = Register::from((instr >> 6) & 0x7);
    // Check the bit 5 to see if we are in immediate mode
    let imm_flag = (instr >> 5) & 0x1;

    if imm_flag == 1 {
        // Get the 5 bits of the imm5 section (first 5 bits) and sign extend them
        let mut imm5 = instr & 0x1;
        imm5 = sign_extend(instr, 5);
        regs[dr] = regs[r1] + imm5;
    } else {
        // Since the immediate flag was off, we only need the SR2 section (first 3 bits).
        // This section contains the register containing the value to add.
        let r2 = Register::from(instr & 0x7);
        regs[dr] = regs[r1] + regs[r2];
    }

    update_flags(dr, regs);
}

fn load_indirect(instr: u16, regs: &mut Registers) {
    // Destination register
    let dr = Register::from((instr >> 9) & 0x7);
    // PCoffset 9 section
    let mut pc_offset = instr & 0xFF; 
    pc_offset = sign_extend(pc_offset, 9);
    // Add the number that was on PCoffset 9 section to get the 
    // memory location we need to look at for the final address
    let address_of_final_address = regs[Register::PC] + pc_offset;
    let final_address = mem_read(address_of_final_address);
    let value = mem_read(final_address);
    regs[dr] = value;
    update_flags(dr, regs);
}
