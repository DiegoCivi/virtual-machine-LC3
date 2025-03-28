use crate::{error::VMError, hardware::{Register, Registers}, utils::{mem_read, sign_extend, update_flags}};

/// Adds to values and stores the result in a register
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
pub fn add(instr: u16, regs: &mut Registers) -> Result<(), VMError> {
    // Destination register
    let dr: Register = Register::from_u16((instr >> 9) & 0x7)?;
    // First operand
    let sr1: Register = Register::from_u16((instr >> 6) & 0x7)?;
    // Check the bit 5 to see if we are in immediate mode
    let imm_flag = (instr >> 5) & 0x1;

    if imm_flag == 1 {
        // Get the 5 bits of the imm5 section (first 5 bits) and sign extend them
        let mut imm5 = instr & 0x1F;
        imm5 = sign_extend(imm5, 5)?;
        regs[dr] = regs[sr1].wrapping_add(imm5);
    } else {
        // Since the immediate flag was off, we only need the SR2 section (first 3 bits).
        // This section contains the register containing the value to add.
        let sr2 = Register::from_u16(instr & 0x7)?;
        regs[dr] = regs[sr1].wrapping_add(regs[sr2]);
    }

    update_flags(dr, regs);
    Ok(())
}

/// Loads a value from a location in memory into a register
/// 
/// ### Arguments
/// 
/// - `instr`: An u16 that has the encoding of the whole instruction to execute.
/// - `regs`: A Registers struct that handles each register.
fn load_indirect(instr: u16, regs: &mut Registers) -> Result<(), VMError> {
    // Destination register
    let dr = Register::from_u16((instr >> 9) & 0x7)?;
    // PCoffset 9 section
    let mut pc_offset = instr & 0xFF; 
    pc_offset = sign_extend(pc_offset, 9)?;
    // Add the number that was on PCoffset 9 section to get the 
    // memory location we need to look at for the final address
    let address_of_final_address = regs[Register::PC].wrapping_add(pc_offset);
    let final_address = mem_read(address_of_final_address);
    let value = mem_read(final_address);
    regs[dr] = value;
    update_flags(dr, regs);
    Ok(())
}

/// Does the bitwise 'AND' between two values and stores the result
/// in a register.
/// 
/// This instruction can happen in two different ways. One is
/// by doing it with two elements that are in a register each, 
/// this is called register mode. The other mode is to use the value
/// of a register with the one that is embedded in the instruction itself.
/// 
/// ### Arguments
/// 
/// - `instr`: An u16 that has the encoding of the whole instruction to execute.
/// - `regs`: A Registers struct that handles each register.
fn and(instr: u16, regs: &mut Registers) -> Result<(), VMError> {
    // Destination register
    let dr = Register::from_u16((instr >> 9) & 0x7)?;
    // SR1 section
    let sr1 = Register::from_u16((instr >> 6) & 0x1)?;
    // Imm flag
    let imm_flag = (instr >> 5) & 0x1;
    
    if imm_flag == 1 {
        // Get the imm5 section, then do the bitwise and with the content on R1.
        let mut imm5 = instr & 0x1F;
        imm5 = sign_extend(imm5, 5)?;
        regs[dr] = regs[Register::R1] & imm5;
    } else {
        // Get the SR2 section, then do the bitwise and with the content on R1.
        let sr2 = Register::from_u16(instr & 0x7)?;
        regs[dr] = regs[sr1] & regs[sr2];
    }

    update_flags(sr1, regs);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::hardware::CondFlag;

    use super::*;

    #[test]
    /// Test result when adding two values in register mode. 
    /// One value will be in R1 and the other in R2, while
    /// the destination register will be R0.
    fn add_with_register_mode() {
        let sr1 = 0x0001;
        let sr2 = 0x0002;
        let result = 0x0003;
        // Create the registers and set the values on R1 and R2
        let mut registers = Registers::new();
        registers[Register::R1] = sr1;
        registers[Register::R2] = sr2;
        // The instruction will have the following encoding:
        // 0 0 0 1 0 0 0 0 0 1 0 0 0 0 1 0
        let instr = 0x1042;
        let _ = add(instr, &mut registers);

        // Check if in R0 we have the desired result
        assert_eq!(registers[Register::R0], result);
    }

    #[test]
    /// Test result when adding the two values in immediate mode. 
    /// One value will be in R1 and the other will be embedded in
    /// the instruction encoding, while the destination 
    /// register will be R0.
    fn add_with_immediate_mode() {
        let sr1 = 0x0001;
        let result = 0x003; 
        // Create the registers and set the value on R1
        let mut registers = Registers::new();
        registers[Register::R1] = sr1;
        // The instruction will have the following encoding:
        // 0 0 0 1 0 0 0 0 0 1 1 0 0 0 1 0
        let instr = 0x1062;
        let _ = add(instr, &mut registers);

        // Check if in R0 we have the desired result
        assert_eq!(registers[Register::R0], result);
    }

    #[test]
    /// Test result when adding one positve value with a 
    /// negative one. To do this adding we need to use immediate mode.
    fn add_with_negative_value() {
        let sr1 = 0x0001;
        let result = 0x000; 
        // Create the registers and set the value on R1
        let mut registers = Registers::new();
        registers[Register::R1] = sr1;
        // The instruction will have the following encoding:
        // 0 0 0 1 0 0 0 0 0 1 1 1 1 1 1 1
        let instr = 0x107F;
        let _ = add(instr, &mut registers);

        // Check if in R0 we have the desired result
        assert_eq!(registers[Register::R0], result);
    }

    #[test]
    /// Test if the condition flag was set to POS when the
    /// result of an addition is a positive number.
    fn add_updates_cond_flag_to_pos() {
        let sr1 = 0x0001;
        let sr2 = 0x0002;
        // Create the registers and set the values on R1 and R2
        let mut registers = Registers::new();
        registers[Register::R1] = sr1;
        registers[Register::R2] = sr2;
        // The instruction will have the following encoding:
        // 0 0 0 1 0 0 0 0 0 1 0 0 0 0 1 0
        let instr = 0x1042;
        let _ = add(instr, &mut registers);

        assert_eq!(registers[Register::Cond], CondFlag::Pos.value());
    }

    #[test]
    /// Test if the condition flag was set to ZRO when the
    /// result of an addition is a 0.
    fn add_updates_cond_flag_to_zro() {
        let sr1 = 0x0001;
        // Create the registers and set the value on R1
        let mut registers = Registers::new();
        registers[Register::R1] = sr1;
        // The instruction will have the following encoding:
        // 0 0 0 1 0 0 0 0 0 1 1 1 1 1 1 1
        let instr = 0x107F;
        let _ = add(instr, &mut registers);

        assert_eq!(registers[Register::Cond], CondFlag::Zro.value());
    }

    #[test]
    fn and_with_register_mode() {
        let sr1 = 0xFFFF;
        let sr2 = 0x0000;
        let result = 0x0000;
        // Create the registers and set the values on R1 and R2
        let mut registers = Registers::new();
        registers[Register::R1] = sr1;
        registers[Register::R2] = sr2;
        // The instruction will have the following encoding:
        // 0 1 0 1 0 0 0 0 0 1 0 0 0 0 1 0
        let instr = 0x5042;
        let _ = and(instr, &mut registers);

        // Check if in R0 we have the desired result
        assert_eq!(registers[Register::R0], result);
    }

    #[test]
    fn and_with_immediate_mode() {
        let sr1 = 0xFFFF;
        let result = 0x0000; 
        // Create the registers and set the value on R1
        let mut registers = Registers::new();
        registers[Register::R1] = sr1;
        // The instruction will have the following encoding:
        // 0 1 0 1  0 0 0 0  0 1 0 0  0 0 0 0
        let instr = 0x5040;
        let _ = and(instr, &mut registers);

        // Check if in R0 we have the desired result
        assert_eq!(registers[Register::R0], result);
    }
}
