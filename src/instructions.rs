// use crate::{
//     error::VMError,
//     hardware::{Memory, Register, Registers},
//     trap_routines::{TrapCode, get_c, halt, out, puts, puts_p, trap_in},
//     utils::sign_extend,
// };
// use std::io::{stdin, stdout};

// const ONE_BIT_MASK: u16 = 0b1;
// const THREE_BIT_MASK: u16 = 0b111;
// const FIVE_BIT_MASK: u16 = 0b11111;
// const SIX_BIT_MASK: u16 = 0b11_1111;
// const EIGHT_BIT_MASK: u16 = 0b1111_1111;
// const NINE_BIT_MASK: u16 = 0b1_1111_1111;
// const ELEVEN_BIT_MASK: u16 = 0b111_1111_1111;


























// #[cfg(test)]
// mod tests {
//     use crate::hardware::CondFlag;

//     use super::*;

    

    

    

    

    

//     #[test]
//     /// Test if doing the bitwise 'AND' with register mode
//     /// gets the correct result
//     fn and_with_register_mode() {
//         let sr1 = 0xFFFF;
//         let sr2 = 0x0000;
//         let result = 0x0000;
//         // Create the registers and set the values on R1 and R2
//         let mut registers = Registers::new();
//         registers[Register::R1] = sr1;
//         registers[Register::R2] = sr2;
//         // The instruction will have the following encoding:
//         // 0 1 0 1 0 0 0 0 0 1 0 0 0 0 1 0
//         let instr = 0x5042;
//         let _ = and(instr, &mut registers);

//         // Check if in R0 we have the desired result
//         assert_eq!(registers[Register::R0], result);
//     }

//     #[test]
//     /// Test if doing the bitwise 'AND' with immediate mode
//     /// gets the correct result
//     fn and_with_immediate_mode() {
//         let sr1 = 0xFFFF;
//         let result = 0x0000;
//         // Create the registers and set the value on R1
//         let mut registers = Registers::new();
//         registers[Register::R1] = sr1;
//         // The instruction will have the following encoding:
//         // 0 1 0 1 0 0 0 0 0 1 0 0 0 0 0 0
//         let instr = 0x5040;
//         let _ = and(instr, &mut registers);

//         // Check if in R0 we have the desired result
//         assert_eq!(registers[Register::R0], result);
//     }

//     #[test]
//     /// Test if bitwise 'NOT' actually negates all the bits
//     /// in a register
//     fn bitwise_not_negates_all_bits() {
//         let sr = 0xFFFF;
//         let result = 0x0000;
//         // Create the registers and set the value on R1
//         let mut registers = Registers::new();
//         registers[Register::R1] = sr;
//         // The instruction will have the following encoding:
//         // 1 0 0 1 0 0 0 0 0 1 1 1 1 1 1 1
//         let instr = 0x907F;
//         let _ = not(instr, &mut registers);

//         // Check if in R0 we have the desired result
//         assert_eq!(registers[Register::R0], result);
//     }

//     #[test]
//     /// Test if branch changes the PC for condition
//     /// flag set to positive
//     fn branch_changes_pc_with_pos_cond_flag() {
//         // Create the registers and set the value on register Cond
//         let mut registers = Registers::new();
//         registers[Register::Cond] = CondFlag::Pos.value();
//         // The instruction will have the following encoding:
//         // 0 0 0 0 0 0 1 0 0 0 0 0 0 0 0 1
//         let instr = 0x0201;
//         let _ = branch(instr, &mut registers);

//         // Check if the PC register was set to 1
//         assert_eq!(registers[Register::PC], 0x0001);
//     }

//     #[test]
//     /// Test if branch changes the PC for condition
//     /// flag set to zero
//     fn branch_changes_pc_with_zro_cond_flag() {
//         // Create the registers and set the value on register Cond
//         let mut registers = Registers::new();
//         registers[Register::Cond] = CondFlag::Zro.value();
//         // The instruction will have the following encoding:
//         // 0 0 0 0 0 1 0 0 0 0 0 0 0 0 0 1
//         let instr = 0x0401;
//         let _ = branch(instr, &mut registers);

//         // Check if the PC register was set to 1
//         assert_eq!(registers[Register::PC], 0x0001);
//     }

//     #[test]
//     /// Test if branch changes the PC for condition
//     /// flag set to negative
//     fn branch_changes_pc_with_neg_cond_flag() {
//         // Create the registers and set the value on register Cond
//         let mut registers = Registers::new();
//         registers[Register::Cond] = CondFlag::Neg.value();
//         // The instruction will have the following encoding:
//         // 0 0 0 0 1 0 0 0 0 0 0 0 0 0 0 1
//         let instr = 0x0801;
//         let _ = branch(instr, &mut registers);

//         // Check if the PC register was set to 1
//         assert_eq!(registers[Register::PC], 0x0001);
//     }

//     #[test]
//     /// Test if jump changes the value of the PC is set
//     /// to the value of the register specified in the
//     /// encoding
//     fn jump_changes_pc() {
//         // Create the registers and set the value of R1 register
//         let mut registers = Registers::new();
//         let result = 0xFFFF;
//         registers[Register::R1] = result;
//         // The instruction will have the following encoding:
//         // 1 1 0 0  0 0 0 0  0 1 0 0  0 0 0 0
//         let instr = 0xC040;
//         let _ = jump(instr, &mut registers);

//         // Check if the PC was set with the value that R1 had
//         assert_eq!(registers[Register::PC], result);
//     }

//     #[test]
//     /// Test if the PC register adds the desired value to the PC
//     /// when using the jump register instruction with the long
//     /// flag set. With this flag being set, means we have 11 bits
//     /// to choose a value for our PC register.
//     fn jump_register_adds_pc_with_long_flag() {
//         let mut registers = Registers::new();
//         let result = 0x03FF;
//         // The instruction will have the following encoding:
//         // 0 1 0 0  1 0 1 1  1 1 1 1  1 1 1 1
//         let instr = 0x4BFF;
//         let _ = jump_register(instr, &mut registers);

//         // Check if the PC register was set to the value
//         // embedded on the encoding
//         assert_eq!(registers[Register::PC], result);
//     }

//     #[test]
//     /// Test if the PC register substracts the desired value to the PC
//     /// when using the jump register instruction with the long
//     /// flag set and a negative value in the PCoffset11 section.
//     /// With this flag being set, means we have 11 bits
//     /// to choose a value for our PC register.
//     ///
//     /// In this case we will set the PC to 10 and we will substract 3
//     /// so it sets to 7.
//     fn jump_register_substracts_pc_with_long_flag() {
//         // Create the registers and set the pc to 10
//         let mut registers = Registers::new();
//         registers[Register::PC] = 0x000A;
//         let result = 0x0007;
//         // The instruction will have the following encoding:
//         // 0 1 0 0  1 1 1 1  1 1 1 1  1 1 0 1
//         let instr = 0x4FFD;
//         let _ = jump_register(instr, &mut registers);

//         // Check if the PC register was set to the value
//         // embedded on the encoding
//         assert_eq!(registers[Register::PC], result);
//     }

//     #[test]
//     /// Test if the PC register changes to the desired value
//     /// when using the jum register instruction without the long
//     /// flag set. This means that the value for the PC register
//     /// will be take from one of the registers.
//     fn jump_register_changes_pc_without_long_flag() {
//         // Create the registers and set the for the PC on R1
//         let mut registers = Registers::new();
//         let result = 0x07FF;
//         registers[Register::R1] = result;
//         // The instruction will have the following encoding:
//         // 0 1 0 0  0 0 0 0  0 1 0 0  0 0 0 0
//         let instr = 0x4040;
//         let _ = jump_register(instr, &mut registers);

//         // Check if the PC register was set to the value
//         // embedded on the encoding
//         assert_eq!(registers[Register::PC], result);
//     }

//     #[test]
//     /// Test if the jump register instruction stores the PC
//     /// value into the R7 register before it is changed to a
//     /// new value.
//     fn jump_register_stores_pc_on_r7() {
//         // Create the registers and set the value of the PC
//         let result = 0x000A;
//         let mut registers = Registers::new();
//         registers[Register::PC] = result;

//         // Check if R7 has a different value before calling the instruction
//         assert_ne!(registers[Register::R7], result);
//         // Run the instruction
//         let _ = jump_register(0x4040, &mut registers);
//         // Check if R7 changed its value to the one the PC had
//         assert_eq!(registers[Register::R7], result);
//     }

//     #[test]
//     /// Test if load indirect instruction changes the value of a register
//     /// with one that was in a place in memory.
//     ///
//     /// We seted in the PCoffset9 section the value 5 and the PC to 10.
//     /// So when adding this we get the memory address 15 where we
//     /// seted to be the address where the result will be found. This
//     /// address will be 20, so the instruction reads that memory address
//     /// and loads the value of it to the register indicated on the instruction
//     /// (register 1 in this case)
//     fn load_indirect_changes_register_value() {
//         // Create the memory and set the values for the addresses
//         let mut memory = Memory::new();
//         let first_address: u16 = 0x000F;
//         let result_address = 0x0014;
//         let result = 0x0001;
//         let _ = memory.write(first_address, result_address);
//         let _ = memory.write(result_address, result);
//         // Create the registers and set the value of pc to 10.
//         let mut registers = Registers::new();
//         registers[Register::PC] = 0x000A;
//         // The instruction will have the following encoding:
//         // 1 0 1 0  0 0 1 0  0 0 0 0  0 1 0 1
//         let instr = 0xA205;
//         let _ = load_indirect(instr, &mut registers, &mut memory);

//         // Check if R1 has the value that was on memory in 'result_address'
//         assert_eq!(registers[Register::R1], result);
//     }

//     #[test]
//     /// Test if load (this time without indirection) instruction changes the
//     /// value of the desired register to the one on a memory address.
//     ///
//     /// This time there is no indirection, so we set the pc to 10, add the value
//     /// 5 into the PCoffset9 section and with that we get the address where
//     /// we will find the value for our register.
//     fn load_changes_register_value() {
//         // Create the memory and set the values for the address
//         let mut memory = Memory::new();
//         let result = 0x0001;
//         let address: u16 = 0x000F;
//         let _ = memory.write(address, result);
//         // Create the registers and set the value of pc to 10.
//         let mut registers = Registers::new();
//         registers[Register::PC] = 0x000A;
//         // The instruction will have the following encoding:
//         // 0 0 1 0  0 0 1 0  0 0 0 0  0 1 0 1
//         let instr = 0x2205;
//         let _ = load(instr, &mut registers, &mut memory);

//         // Check if R1 has the value that was on memory in 'result_address'
//         assert_eq!(registers[Register::R1], result);
//     }

//     #[test]
//     /// Test if load register instruction changes the value of the desired
//     /// register to the one on a memory address that is formed from the value
//     /// in a register plus the one on the offset6 section.
//     ///
//     /// Register 0 will have the value 10 while in the offset 6 section we will
//     /// have the value 5. So then in memory address 15 we will have the value
//     /// 1 that will be set to the register 1.
//     fn load_register_changes_register_value() {
//         // Create the memory and set the values for the address
//         let mut memory = Memory::new();
//         let result = 0x0001;
//         let address: u16 = 0x000F;
//         let _ = memory.write(address, result);
//         // Create the registers and set the value of pc to 10.
//         let mut registers = Registers::new();
//         registers[Register::R0] = 0x000A;
//         // The instruction will have the following encoding:
//         // 0 1 1 0  0 0 1 0  0 0 0 0  0 1 0 1
//         let instr = 0x6205;
//         let _ = load_register(instr, &mut registers, &mut memory);

//         // Check if R1 has the value that was on memory in 'result_address'
//         assert_eq!(registers[Register::R1], result);
//     }

//     #[test]
//     /// Test if load effective address instruction changes the
//     /// value of the desired register to the one that comes from
//     /// the addition of the PC and the PCoffset9 section.
//     ///
//     /// PC will have the value 10 and PCoffset9 will have the value 5
//     /// so after the call of the instruction, register 1 should
//     /// have the value 15
//     fn load_effective_changes_register_value() {
//         // Create the registers and set the value of pc to 10.
//         let mut registers = Registers::new();
//         registers[Register::PC] = 0x000A;
//         let result: u16 = 0x000A + 0x0005;
//         // The instruction will have the following encoding:
//         // 1 1 1 0  0 0 1 0  0 0 0 0  0 1 0 1
//         let instr = 0x6205;
//         let _ = load_effective_address(instr, &mut registers);

//         // Check if R1 has the value of PC + PCoffset9
//         assert_eq!(registers[Register::R1], result);
//     }

//     #[test]
//     /// Test if store instruction changes the value in memory.
//     ///
//     /// Register 1 will have the value 1, while PC and PCoffset9 section
//     /// will both have the value 5. So memory address = PC + PCoffset9 = 10
//     /// will have the value of register 1
//     fn store_changes_value_in_memory() {
//         let pc_val = 0x0005;
//         let affected_address: u16 = pc_val + 0x0005; // PC + PCoffset9
//         // Create the memory
//         let mut memory = Memory::new();
//         // Create the registers and set the values for R1 and PC
//         let mut registers = Registers::new();
//         registers[Register::PC] = 0x0005;
//         registers[Register::R1] = 0x0001;
//         // The instruction will have the following encoding:
//         // 0 0 1 1  0 0 1 0  0 0 0 0  0 1 0 1
//         let instr = 0x3205;
//         let _ = store(instr, &mut registers, &mut memory);

//         // Check if memory[PC + PCoffset9] = registers[R1]
//         assert_eq!(
//             memory.read(affected_address).unwrap(),
//             registers[Register::R1]
//         );
//     }

//     #[test]
//     /// Test if store indirect instruction changes the value in memory.
//     ///
//     /// Register 1 will have the value 1, address 0x000A will have the value
//     /// 0x000F, while PC and PCoffset9 section will both have the value 5.
//     /// So the first memory address = PC + PCoffset9 = 10 = 0x000A. When we
//     /// read that address we will get the final address = 0x000F and thats
//     /// the one that will be written with the value of register 1.
//     fn store_indirect_changes_value_in_memory() {
//         // Create the memory and set the value of the address 0x000A to 0x000F
//         let mut memory = Memory::new();
//         let first_address: u16 = 0x000A;
//         let final_address: u16 = 0x000F;
//         let _ = memory.write(first_address, final_address);
//         // Create the registers and set the values of PC and R1
//         let mut registers = Registers::new();
//         registers[Register::R1] = 0x0001;
//         registers[Register::PC] = 0x0005;
//         // The instruction will have the following encoding:
//         // 1 0 1 1  0 0 1 0  0 0 0 0  0 1 0 1
//         let instr = 0xB205;
//         let _ = store_indirect(instr, &mut registers, &mut memory);

//         // Check if 0x000F has the value of register R1
//         assert_eq!(memory.read(final_address).unwrap(), registers[Register::R1]);
//     }

//     #[test]
//     /// Test if store register instruction changes the value in memory.
//     ///
//     /// Register 1 will have value 1, register 0 and offset6 section will both
//     /// have value 5. The address = offset6 + register 0 = 0x000A will be written
//     /// with the value of register 1.
//     fn store_register_changes_value_in_memory() {
//         let mut memory = Memory::new();
//         // Create the registers and set the values of R1 and R0
//         let mut registers = Registers::new();
//         registers[Register::R0] = 0x0005;
//         registers[Register::R1] = 0x0001;
//         // The instruction will have the following encoding:
//         // 0 1 1 1  0 0 1  0 0 0  0 0 0 1 0 1
//         let instr = 0x7205;
//         let _ = store_register(instr, &mut registers, &mut memory);

//         // Check if address 0x000A = R0 + offset6 was written with R1's value
//         let affected_address: u16 = 0x000A;
//         assert_eq!(
//             memory.read(affected_address).unwrap(),
//             registers[Register::R1]
//         );
//     }

//     #[test]
//     /// Test if by using the trap instruction, the register R7 gets the value
//     /// of the PC
//     fn trap_sets_register_7_with_pc_value() {
//         let mut running_flag = false;
//         let mut regs = Registers::new();
//         let mut mem = Memory::new();
//         let pc_val = 0x0A0A;
//         regs[Register::PC] = pc_val;
//         // The instruction will have the following encoding:
//         // 1 1 1 1  0 0 0 0  0 0 1 0  0 1 0 1
//         let instr = 0xF025;

//         let _ = trap(instr, &mut regs, &mut mem, &mut running_flag);

//         assert_eq!(regs[Register::R7], pc_val);
//     }

//     #[test]
//     /// Test trap instruction calls the halt trap routine
//     fn trap_calls_correct_subroutine() {
//         let mut running_flag = true;
//         let mut regs = Registers::new();
//         let mut mem = Memory::new();
//         let pc_val = 0x0A0A;
//         regs[Register::PC] = pc_val;
//         // The instruction will have the following encoding:
//         // 1 1 1 1  0 0 0 0  0 0 1 0  0 1 0 1
//         let instr = 0xF025;

//         let _ = trap(instr, &mut regs, &mut mem, &mut running_flag);

//         // The running flag should change to false
//         assert!(!running_flag);
//     }
// }
