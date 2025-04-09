use crate::{
    error::VMError,
    hardware::{Memory, Register, Registers},
    trap_routines::{TrapCode, get_c, halt, out, puts, puts_p, trap_in},
    utils::{sign_extend, update_flags},
};
use std::io::{stdin, stdout};

const ONE_BIT_MASK: u16 = 0b1;
const THREE_BIT_MASK: u16 = 0b111;
const FIVE_BIT_MASK: u16 = 0b11111;
const SIX_BIT_MASK: u16 = 0b11_1111;
const EIGHT_BIT_MASK: u16 = 0b1111_1111;
const NINE_BIT_MASK: u16 = 0b1_1111_1111;
const ELEVEN_BIT_MASK: u16 = 0b111_1111_1111;

/// Adds to values and stores the result in a register
///
/// This instruction can happen in two different ways. One is
/// by adding two elements that are in a register each, this is called
/// register mode. The other mode, known as immediate mode, is to add the value of a register
/// with the one that is embedded in the instruction itself.
///
/// ### Arguments
///
/// - `instr`: An u16 that has the encoding of the whole instruction to execute.
/// - `regs`: A Registers struct that handles each register.
pub fn add(instr: u16, regs: &mut Registers) -> Result<(), VMError> {
    // Destination register
    let dr: Register = Register::from_u16((instr >> 9) & THREE_BIT_MASK)?;
    // First operand
    let sr1: Register = Register::from_u16((instr >> 6) & THREE_BIT_MASK)?;
    // Check the bit 5 to see if we are in immediate mode
    let imm_flag = (instr >> 5) & ONE_BIT_MASK;

    if imm_flag == 1 {
        // Get the 5 bits of the imm5 section (first 5 bits) and sign extend them
        let mut imm5 = instr & FIVE_BIT_MASK;
        imm5 = sign_extend(imm5, 5)?;
        regs[dr] = regs[sr1].wrapping_add(imm5);
    } else {
        // Since the immediate flag was off, we only need the SR2 section (first 3 bits).
        // This section contains the register containing the value to add.
        let sr2 = Register::from_u16(instr & THREE_BIT_MASK)?;
        regs[dr] = regs[sr1].wrapping_add(regs[sr2]);
    }

    update_flags(dr, regs);
    Ok(())
}

/// Does the bitwise 'NOT' for a value stored in a register.
///
/// ### Arguments
///
/// - `instr`: An u16 that has the encoding of the whole instruction to execute.
/// - `regs`: A Registers struct that handles each register.
pub fn not(instr: u16, regs: &mut Registers) -> Result<(), VMError> {
    let dr = Register::from_u16((instr >> 9) & THREE_BIT_MASK)?;
    let sr = Register::from_u16((instr >> 6) & THREE_BIT_MASK)?;

    regs[dr] = !regs[sr];
    update_flags(dr, regs);
    Ok(())
}

/// Does the bitwise 'AND' between two values and stores the result
/// in a register.
///
/// This instruction can be executed in two different ways:
/// - register mode: Bitwise and with two elements that are in a register each. 
/// - immediate mode: Uses the value of a register with the one that 
/// is embedded in the instruction itself.
///
/// ### Arguments
///
/// - `instr`: An u16 that has the encoding of the whole instruction to execute.
/// - `regs`: A Registers struct that handles each register.
pub fn and(instr: u16, regs: &mut Registers) -> Result<(), VMError> {
    // Destination register
    let dr = Register::from_u16((instr >> 9) & THREE_BIT_MASK)?;
    // SR1 section
    let sr1 = Register::from_u16((instr >> 6) & THREE_BIT_MASK)?;
    // Imm flag
    let imm_flag = (instr >> 5) & ONE_BIT_MASK;

    if imm_flag == 1 {
        // Get the imm5 section, then do the bitwise and with the content on R1.
        let mut imm5 = instr & FIVE_BIT_MASK;
        imm5 = sign_extend(imm5, 5)?;
        regs[dr] = regs[sr1] & imm5;
    } else {
        // Get the SR2 section, then do the bitwise and with the content on R1.
        let sr2 = Register::from_u16(instr & THREE_BIT_MASK)?;
        regs[dr] = regs[sr1] & regs[sr2];
    }

    update_flags(dr, regs);
    Ok(())
}

/// Changes the PC register if the Cond register is set to the
/// flag that is selected on the encoding of the instruction
///
/// ### Arguments
///
///
/// - `instr`: An u16 that has the encoding of the whole instruction to execute.
/// - `regs`: A Registers struct that handles each register.
pub fn branch(instr: u16, regs: &mut Registers) -> Result<(), VMError> {
    // Get the PCOffset9 section
    let mut pc_offset = instr & NINE_BIT_MASK;
    pc_offset = sign_extend(pc_offset, 9)?;
    // Get the Condition Flag and check if it is the same
    // as the one selected on the instruction
    let cond_flag = (instr >> 9) & THREE_BIT_MASK;
    let coincides = cond_flag & regs[Register::Cond];
    if coincides != 0 {
        regs[Register::PC] = regs[Register::PC].wrapping_add(pc_offset);
    }
    Ok(())
}

/// Changes the PC with the value of a register specified in the instruction
/// itself
pub fn jump(instr: u16, regs: &mut Registers) -> Result<(), VMError> {
    // Get the BaseR section
    let baser_r = Register::from_u16((instr >> 6) & THREE_BIT_MASK)?;
    regs[Register::PC] = regs[baser_r];
    Ok(())
}

/// Changes the PC register with the value on a register that is specified
/// on the R1 section of the encoding or in the same instruction encoding.
/// This depends on the long flag situated in the bit 11. The long flag
/// being set means it can be a value of eleven bits. If the flags is not
/// set, the value is taken from a register.
pub fn jump_register(instr: u16, regs: &mut Registers) -> Result<(), VMError> {
    let long_flag = (instr >> 11) & 1;
    regs[Register::R7] = regs[Register::PC];
    if long_flag == 1 {
        let mut long_pc_offset = instr & ELEVEN_BIT_MASK;
        long_pc_offset = sign_extend(long_pc_offset, 11)?;
        regs[Register::PC] = regs[Register::PC].wrapping_add(long_pc_offset);
    } else {
        let r1 = Register::from_u16((instr >> 6) & THREE_BIT_MASK)?;
        regs[Register::PC] = regs[r1];
    }
    Ok(())
}

/// Loads a value from a location in memory that is pointed by another memory
/// location, into a register
///
/// ### Arguments
///
/// - `instr`: An u16 that has the encoding of the whole instruction to execute.
/// - `regs`: A Registers struct that handles each register.
/// - `memory`: A Memory struct that handles reads and writes on the vm memory.
pub fn load_indirect(instr: u16, regs: &mut Registers, memory: &mut Memory) -> Result<(), VMError> {
    // Destination register
    let dr = Register::from_u16((instr >> 9) & THREE_BIT_MASK)?;
    // PCoffset 9 section
    let mut pc_offset = instr & NINE_BIT_MASK;
    pc_offset = sign_extend(pc_offset, 9)?;
    // Add the number that was on PCoffset 9 section to PC to get the
    // memory location we need to look at for the final address
    let address_of_final_address = regs[Register::PC].wrapping_add(pc_offset);
    let final_address = memory.read(address_of_final_address)?;
    regs[dr] = memory.read(final_address)?;
    update_flags(dr, regs);
    Ok(())
}

/// Loads a value from a location in memory and stores the loaded value into a register
pub fn load(instr: u16, regs: &mut Registers, memory: &mut Memory) -> Result<(), VMError> {
    // Destination register
    let dr = Register::from_u16((instr >> 9) & THREE_BIT_MASK)?;
    // PCoffset 9 section
    let mut pc_offset = instr & NINE_BIT_MASK;
    pc_offset = sign_extend(pc_offset, 9)?;
    // Calculate the memory address to read
    let address = regs[Register::PC].wrapping_add(pc_offset);
    regs[dr] = memory.read(address)?;
    update_flags(dr, regs);
    Ok(())
}

/// Loads a value that is located in a memory address, formed by the addition
/// of the value on a register and in the offset6 section, into a desired register
pub fn load_register(instr: u16, regs: &mut Registers, memory: &mut Memory) -> Result<(), VMError> {
    // Destination Register
    let dr = Register::from_u16((instr >> 9) & THREE_BIT_MASK)?;
    // BaseR section
    let r1 = Register::from_u16((instr >> 6) & THREE_BIT_MASK)?;
    // Offset6 section
    let mut offset6 = instr & SIX_BIT_MASK;
    offset6 = sign_extend(offset6, 6)?;
    // Calculate the memory address to read
    let address = regs[r1].wrapping_add(offset6);
    regs[dr] = memory.read(address)?;
    update_flags(dr, regs);
    Ok(())
}

/// Loads a value created from the addition of the value of the PC and the
/// one in the PCoffset9 section, into a register
pub fn load_effective_address(instr: u16, regs: &mut Registers) -> Result<(), VMError> {
    // Destination Register
    let dr = Register::from_u16((instr >> 9) & THREE_BIT_MASK)?;
    // PCoffset9 section
    let mut pc_offset = instr & NINE_BIT_MASK;
    pc_offset = sign_extend(pc_offset, 9)?;
    // Set the new value for the destination register
    regs[dr] = regs[Register::PC].wrapping_add(pc_offset);
    update_flags(dr, regs);
    Ok(())
}

/// Reads a value from a register and stores it into memory. This address
/// is created from the addition of the PC and the PCoffset9 section
pub fn store(instr: u16, regs: &mut Registers, memory: &mut Memory) -> Result<(), VMError> {
    // Source Register
    let sr = Register::from_u16((instr >> 9) & THREE_BIT_MASK)?;
    // PCoffset9 section
    let mut pc_offset = instr & NINE_BIT_MASK;
    pc_offset = sign_extend(pc_offset, 9)?;
    // Calculate the address
    let address = regs[Register::PC].wrapping_add(pc_offset);
    let new_val = regs[sr];
    memory.write(address, new_val)
}

/// Stores the value that is in a register into an address in memory. This address
/// is taken indirectly from the instruction. By adding the PC and the PCoffset9 section
/// we get the first memory address, then if we read it we get the final address. That
/// final address is the one that is going to get written.
pub fn store_indirect(
    instr: u16,
    regs: &mut Registers,
    memory: &mut Memory,
) -> Result<(), VMError> {
    // Source Register
    let sr = Register::from_u16((instr >> 9) & THREE_BIT_MASK)?;
    // PCoffset9 section
    let mut pc_offset = instr & NINE_BIT_MASK;
    pc_offset = sign_extend(pc_offset, 9)?;
    // Get the first address
    let first_address = regs[Register::PC].wrapping_add(pc_offset);
    // Read the first address, get the second one and write on it
    let final_address = memory.read(first_address)?;
    let new_val = regs[sr];
    memory.write(final_address, new_val)
}

/// Stores the value that is in a register into an address in memory. By adding
/// the value on the register specified in the BaseR section and the value in the
/// offset6 section we get the memory address. That address is the one that is going to get written.
pub fn store_register(
    instr: u16,
    regs: &mut Registers,
    memory: &mut Memory,
) -> Result<(), VMError> {
    // Source Register
    let sr = Register::from_u16((instr >> 9) & THREE_BIT_MASK)?;
    // BaseR section
    let r1 = Register::from_u16((instr >> 6) & THREE_BIT_MASK)?;
    // Offset 6 section
    let mut offset = instr & SIX_BIT_MASK;
    offset = sign_extend(offset, 6)?;
    // Calculate the address
    let address = regs[r1].wrapping_add(offset);
    let new_val = regs[sr];
    memory.write(address, new_val)
}

pub fn trap(
    instr: u16,
    regs: &mut Registers,
    memory: &mut Memory,
    running_flag: &mut bool,
) -> Result<(), VMError> {
    regs[Register::R7] = regs[Register::PC];
    let trap_code = TrapCode::try_from(instr & EIGHT_BIT_MASK)?;
    let mut std_in = stdin().lock();
    let mut std_out = stdout().lock();
    match trap_code {
        TrapCode::GetC => get_c(regs, &mut std_in)?,
        TrapCode::Out => out(regs, &mut std_out)?,
        TrapCode::Puts => puts(regs, memory, &mut std_out)?,
        TrapCode::In => trap_in(regs, &mut std_out, &mut std_in)?,
        TrapCode::PutsP => puts_p(regs, memory, &mut std_out)?,
        TrapCode::Halt => halt(running_flag, &mut std_out)?,
    }
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
    /// Test if doing the bitwise 'AND' with register mode
    /// gets the correct result
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
    /// Test if doing the bitwise 'AND' with immediate mode
    /// gets the correct result
    fn and_with_immediate_mode() {
        let sr1 = 0xFFFF;
        let result = 0x0000;
        // Create the registers and set the value on R1
        let mut registers = Registers::new();
        registers[Register::R1] = sr1;
        // The instruction will have the following encoding:
        // 0 1 0 1 0 0 0 0 0 1 0 0 0 0 0 0
        let instr = 0x5040;
        let _ = and(instr, &mut registers);

        // Check if in R0 we have the desired result
        assert_eq!(registers[Register::R0], result);
    }

    #[test]
    /// Test if bitwise 'NOT' actually negates all the bits
    /// in a register
    fn bitwise_not_negates_all_bits() {
        let sr = 0xFFFF;
        let result = 0x0000;
        // Create the registers and set the value on R1
        let mut registers = Registers::new();
        registers[Register::R1] = sr;
        // The instruction will have the following encoding:
        // 1 0 0 1 0 0 0 0 0 1 1 1 1 1 1 1
        let instr = 0x907F;
        let _ = not(instr, &mut registers);

        // Check if in R0 we have the desired result
        assert_eq!(registers[Register::R0], result);
    }

    #[test]
    /// Test if branch changes the PC for condition
    /// flag set to positive
    fn branch_changes_pc_with_pos_cond_flag() {
        // Create the registers and set the value on register Cond
        let mut registers = Registers::new();
        registers[Register::Cond] = CondFlag::Pos.value();
        // The instruction will have the following encoding:
        // 0 0 0 0 0 0 1 0 0 0 0 0 0 0 0 1
        let instr = 0x0201;
        let _ = branch(instr, &mut registers);

        // Check if the PC register was set to 1
        assert_eq!(registers[Register::PC], 0x0001);
    }

    #[test]
    /// Test if branch changes the PC for condition
    /// flag set to zero
    fn branch_changes_pc_with_zro_cond_flag() {
        // Create the registers and set the value on register Cond
        let mut registers = Registers::new();
        registers[Register::Cond] = CondFlag::Zro.value();
        // The instruction will have the following encoding:
        // 0 0 0 0 0 1 0 0 0 0 0 0 0 0 0 1
        let instr = 0x0401;
        let _ = branch(instr, &mut registers);

        // Check if the PC register was set to 1
        assert_eq!(registers[Register::PC], 0x0001);
    }

    #[test]
    /// Test if branch changes the PC for condition
    /// flag set to negative
    fn branch_changes_pc_with_neg_cond_flag() {
        // Create the registers and set the value on register Cond
        let mut registers = Registers::new();
        registers[Register::Cond] = CondFlag::Neg.value();
        // The instruction will have the following encoding:
        // 0 0 0 0 1 0 0 0 0 0 0 0 0 0 0 1
        let instr = 0x0801;
        let _ = branch(instr, &mut registers);

        // Check if the PC register was set to 1
        assert_eq!(registers[Register::PC], 0x0001);
    }

    #[test]
    /// Test if jump changes the value of the PC is set
    /// to the value of the register specified in the
    /// encoding
    fn jump_changes_pc() {
        // Create the registers and set the value of R1 register
        let mut registers = Registers::new();
        let result = 0xFFFF;
        registers[Register::R1] = result;
        // The instruction will have the following encoding:
        // 1 1 0 0  0 0 0 0  0 1 0 0  0 0 0 0
        let instr = 0xC040;
        let _ = jump(instr, &mut registers);

        // Check if the PC was set with the value that R1 had
        assert_eq!(registers[Register::PC], result);
    }

    #[test]
    /// Test if the PC register adds the desired value to the PC
    /// when using the jump register instruction with the long
    /// flag set. With this flag being set, means we have 11 bits
    /// to choose a value for our PC register.
    fn jump_register_adds_pc_with_long_flag() {
        let mut registers = Registers::new();
        let result = 0x03FF;
        // The instruction will have the following encoding:
        // 0 1 0 0  1 0 1 1  1 1 1 1  1 1 1 1
        let instr = 0x4BFF;
        let _ = jump_register(instr, &mut registers);

        // Check if the PC register was set to the value
        // embedded on the encoding
        assert_eq!(registers[Register::PC], result);
    }

    #[test]
    /// Test if the PC register substracts the desired value to the PC
    /// when using the jump register instruction with the long
    /// flag set and a negative value in the PCoffset11 section.
    /// With this flag being set, means we have 11 bits
    /// to choose a value for our PC register.
    ///
    /// In this case we will set the PC to 10 and we will substract 3
    /// so it sets to 7.
    fn jump_register_substracts_pc_with_long_flag() {
        // Create the registers and set the pc to 10
        let mut registers = Registers::new();
        registers[Register::PC] = 0x000A;
        let result = 0x0007;
        // The instruction will have the following encoding:
        // 0 1 0 0  1 1 1 1  1 1 1 1  1 1 0 1
        let instr = 0x4FFD;
        let _ = jump_register(instr, &mut registers);

        // Check if the PC register was set to the value
        // embedded on the encoding
        assert_eq!(registers[Register::PC], result);
    }

    #[test]
    /// Test if the PC register changes to the desired value
    /// when using the jum register instruction without the long
    /// flag set. This means that the value for the PC register
    /// will be take from one of the registers.
    fn jump_register_changes_pc_without_long_flag() {
        // Create the registers and set the for the PC on R1
        let mut registers = Registers::new();
        let result = 0x07FF;
        registers[Register::R1] = result;
        // The instruction will have the following encoding:
        // 0 1 0 0  0 0 0 0  0 1 0 0  0 0 0 0
        let instr = 0x4040;
        let _ = jump_register(instr, &mut registers);

        // Check if the PC register was set to the value
        // embedded on the encoding
        assert_eq!(registers[Register::PC], result);
    }

    #[test]
    /// Test if the jump register instruction stores the PC
    /// value into the R7 register before it is changed to a
    /// new value.
    fn jump_register_stores_pc_on_r7() {
        // Create the registers and set the value of the PC
        let result = 0x000A;
        let mut registers = Registers::new();
        registers[Register::PC] = result;

        // Check if R7 has a different value before calling the instruction
        assert_ne!(registers[Register::R7], result);
        // Run the instruction
        let _ = jump_register(0x4040, &mut registers);
        // Check if R7 changed its value to the one the PC had
        assert_eq!(registers[Register::R7], result);
    }

    #[test]
    /// Test if load indirect instruction changes the value of a register
    /// with one that was in a place in memory.
    ///
    /// We seted in the PCoffset9 section the value 5 and the PC to 10.
    /// So when adding this we get the memory address 15 where we
    /// seted to be the address where the result will be found. This
    /// address will be 20, so the instruction reads that memory address
    /// and loads the value of it to the register indicated on the instruction
    /// (register 1 in this case)
    fn load_indirect_changes_register_value() {
        // Create the memory and set the values for the addresses
        let mut memory = Memory::new();
        let first_address: u16 = 0x000F;
        let result_address = 0x0014;
        let result = 0x0001;
        let _ = memory.write(first_address, result_address);
        let _ = memory.write(result_address, result);
        // Create the registers and set the value of pc to 10.
        let mut registers = Registers::new();
        registers[Register::PC] = 0x000A;
        // The instruction will have the following encoding:
        // 1 0 1 0  0 0 1 0  0 0 0 0  0 1 0 1
        let instr = 0xA205;
        let _ = load_indirect(instr, &mut registers, &mut memory);

        // Check if R1 has the value that was on memory in 'result_address'
        assert_eq!(registers[Register::R1], result);
    }

    #[test]
    /// Test if load (this time without indirection) instruction changes the
    /// value of the desired register to the one on a memory address.
    ///
    /// This time there is no indirection, so we set the pc to 10, add the value
    /// 5 into the PCoffset9 section and with that we get the address where
    /// we will find the value for our register.
    fn load_changes_register_value() {
        // Create the memory and set the values for the address
        let mut memory = Memory::new();
        let result = 0x0001;
        let address: u16 = 0x000F;
        let _ = memory.write(address, result);
        // Create the registers and set the value of pc to 10.
        let mut registers = Registers::new();
        registers[Register::PC] = 0x000A;
        // The instruction will have the following encoding:
        // 0 0 1 0  0 0 1 0  0 0 0 0  0 1 0 1
        let instr = 0x2205;
        let _ = load(instr, &mut registers, &mut memory);

        // Check if R1 has the value that was on memory in 'result_address'
        assert_eq!(registers[Register::R1], result);
    }

    #[test]
    /// Test if load register instruction changes the value of the desired
    /// register to the one on a memory address that is formed from the value
    /// in a register plus the one on the offset6 section.
    ///
    /// Register 0 will have the value 10 while in the offset 6 section we will
    /// have the value 5. So then in memory address 15 we will have the value
    /// 1 that will be set to the register 1.
    fn load_register_changes_register_value() {
        // Create the memory and set the values for the address
        let mut memory = Memory::new();
        let result = 0x0001;
        let address: u16 = 0x000F;
        let _ = memory.write(address, result);
        // Create the registers and set the value of pc to 10.
        let mut registers = Registers::new();
        registers[Register::R0] = 0x000A;
        // The instruction will have the following encoding:
        // 0 1 1 0  0 0 1 0  0 0 0 0  0 1 0 1
        let instr = 0x6205;
        let _ = load_register(instr, &mut registers, &mut memory);

        // Check if R1 has the value that was on memory in 'result_address'
        assert_eq!(registers[Register::R1], result);
    }

    #[test]
    /// Test if load effective address instruction changes the
    /// value of the desired register to the one that comes from
    /// the addition of the PC and the PCoffset9 section.
    ///
    /// PC will have the value 10 and PCoffset9 will have the value 5
    /// so after the call of the instruction, register 1 should
    /// have the value 15
    fn load_effective_changes_register_value() {
        // Create the registers and set the value of pc to 10.
        let mut registers = Registers::new();
        registers[Register::PC] = 0x000A;
        let result: u16 = 0x000A + 0x0005;
        // The instruction will have the following encoding:
        // 1 1 1 0  0 0 1 0  0 0 0 0  0 1 0 1
        let instr = 0x6205;
        let _ = load_effective_address(instr, &mut registers);

        // Check if R1 has the value of PC + PCoffset9
        assert_eq!(registers[Register::R1], result);
    }

    #[test]
    /// Test if store instruction changes the value in memory.
    ///
    /// Register 1 will have the value 1, while PC and PCoffset9 section
    /// will both have the value 5. So memory address = PC + PCoffset9 = 10
    /// will have the value of register 1
    fn store_changes_value_in_memory() {
        let pc_val = 0x0005;
        let affected_address: u16 = pc_val + 0x0005; // PC + PCoffset9
        // Create the memory
        let mut memory = Memory::new();
        // Create the registers and set the values for R1 and PC
        let mut registers = Registers::new();
        registers[Register::PC] = 0x0005;
        registers[Register::R1] = 0x0001;
        // The instruction will have the following encoding:
        // 0 0 1 1  0 0 1 0  0 0 0 0  0 1 0 1
        let instr = 0x3205;
        let _ = store(instr, &mut registers, &mut memory);

        // Check if memory[PC + PCoffset9] = registers[R1]
        assert_eq!(
            memory.read(affected_address).unwrap(),
            registers[Register::R1]
        );
    }

    #[test]
    /// Test if store indirect instruction changes the value in memory.
    ///
    /// Register 1 will have the value 1, address 0x000A will have the value
    /// 0x000F, while PC and PCoffset9 section will both have the value 5.
    /// So the first memory address = PC + PCoffset9 = 10 = 0x000A. When we
    /// read that address we will get the final address = 0x000F and thats
    /// the one that will be written with the value of register 1.
    fn store_indirect_changes_value_in_memory() {
        // Create the memory and set the value of the address 0x000A to 0x000F
        let mut memory = Memory::new();
        let first_address: u16 = 0x000A;
        let final_address: u16 = 0x000F;
        let _ = memory.write(first_address, final_address);
        // Create the registers and set the values of PC and R1
        let mut registers = Registers::new();
        registers[Register::R1] = 0x0001;
        registers[Register::PC] = 0x0005;
        // The instruction will have the following encoding:
        // 1 0 1 1  0 0 1 0  0 0 0 0  0 1 0 1
        let instr = 0xB205;
        let _ = store_indirect(instr, &mut registers, &mut memory);

        // Check if 0x000F has the value of register R1
        assert_eq!(memory.read(final_address).unwrap(), registers[Register::R1]);
    }

    #[test]
    /// Test if store register instruction changes the value in memory.
    ///
    /// Register 1 will have value 1, register 0 and offset6 section will both
    /// have value 5. The address = offset6 + register 0 = 0x000A will be written
    /// with the value of register 1.
    fn store_register_changes_value_in_memory() {
        let mut memory = Memory::new();
        // Create the registers and set the values of R1 and R0
        let mut registers = Registers::new();
        registers[Register::R0] = 0x0005;
        registers[Register::R1] = 0x0001;
        // The instruction will have the following encoding:
        // 0 1 1 1  0 0 1  0 0 0  0 0 0 1 0 1
        let instr = 0x7205;
        let _ = store_register(instr, &mut registers, &mut memory);

        // Check if address 0x000A = R0 + offset6 was written with R1's value
        let affected_address: u16 = 0x000A;
        assert_eq!(
            memory.read(affected_address).unwrap(),
            registers[Register::R1]
        );
    }

    #[test]
    /// Test if by using the trap instruction, the register R7 gets the value
    /// of the PC
    fn trap_sets_register_7_with_pc_value() {
        let mut running_flag = false;
        let mut regs = Registers::new();
        let mut mem = Memory::new();
        let pc_val = 0x0A0A;
        regs[Register::PC] = pc_val;
        // The instruction will have the following encoding:
        // 1 1 1 1  0 0 0 0  0 0 1 0  0 1 0 1
        let instr = 0xF025;

        let _ = trap(instr, &mut regs, &mut mem, &mut running_flag);

        assert_eq!(regs[Register::R7], pc_val);
    }

    #[test]
    /// Test trap instruction calls the halt trap routine
    fn trap_calls_correct_subroutine() {
        let mut running_flag = true;
        let mut regs = Registers::new();
        let mut mem = Memory::new();
        let pc_val = 0x0A0A;
        regs[Register::PC] = pc_val;
        // The instruction will have the following encoding:
        // 1 1 1 1  0 0 0 0  0 0 1 0  0 1 0 1
        let instr = 0xF025;

        let _ = trap(instr, &mut regs, &mut mem, &mut running_flag);

        // The running flag should change to false
        assert!(!running_flag);
    }
}
