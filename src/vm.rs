use std::{env::Args, fs, io::{stdin, stdout, Error, Read, Write}, num::TryFromIntError, process::exit};

use crate::{
    error::VMError, hardware::{CondFlag, Memory, OpCode, Register, Registers}, trap_code::*, utils::{getchar, sign_extend, stdout_flush, stdout_write}
};

const NULL: u16 = 0x0000;
const PC_START: u16 = 0x3000;
const ONE_BIT_MASK: u16 = 0b1;
const THREE_BIT_MASK: u16 = 0b111;
const FIVE_BIT_MASK: u16 = 0b11111;
const SIX_BIT_MASK: u16 = 0b11_1111;
const EIGHT_BIT_MASK: u16 = 0b1111_1111;
const NINE_BIT_MASK: u16 = 0b1_1111_1111;
const ELEVEN_BIT_MASK: u16 = 0b111_1111_1111;

pub struct VM {
    mem: Memory,
    regs: Registers,
    running: bool,
}

impl VM {
    /// Creates a new instance of the VM abstraction
    pub fn new() -> Self {
        let mut regs = Registers::new();
        let mem = Memory::new();
        // Initialize the registers Cond and PC to standard values
        regs[Register::Cond] = CondFlag::Zro.value();
        regs[Register::PC] = PC_START;

        Self {
            regs,
            mem,
            running: true,
        }
    }

    /// Loads the file into the vm memory
    pub fn load_arguments(&mut self, args: &mut Args) -> Result<(), VMError> {
        if args.len() < 2 {
            println!("lc3 [image-file1] ...");
            exit(2);
        }
        // We skip the first element of the args since it is not an image
        args.next();
        for path in args {
            if self.read_image(path.clone()).is_err() {
                println!("failed to load image: {path}");
                exit(1);
            }
        }
        Ok(())
    }

    /// Reads bytes from file and send them to get into memory
    fn read_image(&mut self, path: String) -> Result<(), VMError> {
        let mut f = fs::read(path).map_err(|e: Error| VMError::OpenFile(e.to_string()))?;
        self.read_image_file(&mut f)?;
        Ok(())
    }

    /// Writes a file encoded in bytes into memory.
    ///
    /// ### Arguments
    ///
    /// - `file_bytes`: A vector of u8 which represent each byte of the file with the file that will be written in memory.
    fn read_image_file(&mut self, file_bytes: &mut Vec<u8>) -> Result<(), VMError> {
        // Get the first 2 bytes and join them in reverse order to get the origin
        let byte0 = file_bytes.remove(0);
        let byte1 = file_bytes.remove(0);
        let origin = u16::from_be_bytes([byte0, byte1]);

        // Get chunks of 2 bytes and join them in reverse order so we get the data.
        // This data starts to get written from memory address = origin
        let mut mem_addr = origin;
        for chunk in file_bytes.chunks(2) {
            let mut chunk_iter = chunk.iter();
            let byte0 = *chunk_iter
                .next()
                .ok_or(VMError::NoMoreBytes(String::from("No byte0 on chunk")))?;
            let byte1 = *chunk_iter
                .next()
                .ok_or(VMError::NoMoreBytes(String::from("No byte1 in chunk")))?;
            let data = u16::from_be_bytes([byte0, byte1]);

            self.mem.write(mem_addr, data)?;
            mem_addr = mem_addr.wrapping_add(1);
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), VMError> {
        while self.running {
            let instr_addr = self.regs[Register::PC];
            self.regs[Register::PC] = self.regs[Register::PC].wrapping_add(1);
            let instr = self.mem.read(instr_addr)?;
            let op_code = OpCode::try_from(instr >> 12)?;
            match op_code {
                OpCode::Br => self.branch(instr)?,
                OpCode::Add => self.add(instr)?,
                OpCode::Ld => self.load(instr)?,
                OpCode::St => self.store(instr)?,
                OpCode::Jsr => self.jump_register(instr)?,
                OpCode::And => self.and(instr)?,
                OpCode::Ldr => self.load_register(instr)?,
                OpCode::Str => self.store_register(instr)?,
                OpCode::Not => self.not(instr)?,
                OpCode::Ldi => self.load_indirect(instr)?,
                OpCode::Sti => self.store_indirect(instr)?,
                OpCode::Jmp => self.jump(instr)?,
                OpCode::Lea => self.load_effective_address(instr)?,
                OpCode::Trap => self.trap(instr)?,
            }
        }
        Ok(())
    }

    /// Updates the register COND where we have the condition flag
    pub fn update_flags(&mut self, r: Register) {
        if self.regs[r] == 0 {
            self.regs[Register::Cond] = CondFlag::Zro.value();
        } else if self.regs[r] >> 15 == 1 {
            self.regs[Register::Cond] = CondFlag::Neg.value();
        } else {
            self.regs[Register::Cond] = CondFlag::Pos.value();
        }
    }

    /// Adds two values and stores the result in a register
    ///
    /// This instruction can be executed in two different ways:
    ///
    /// - Register mode: Adds two values taken from the registers storage.
    /// - Immediate mode: Adds a value taken from the registers storage with one taken from the instruction itself. The second operand of the addition is taken from a range of bits of the executed instruction.
    ///
    /// ### Arguments
    ///
    /// - `instr`: An u16 that has the encoding of the whole instruction to execute.
    pub fn add(&mut self, instr: u16) -> Result<(), VMError> {
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
            self.regs[dr] = self.regs[sr1].wrapping_add(imm5);
        } else {
            // Since the immediate flag was off, we only need the SR2 section (first 3 bits).
            // This section contains the register containing the value to add.
            let sr2 = Register::from_u16(instr & THREE_BIT_MASK)?;
            self.regs[dr] = self.regs[sr1].wrapping_add(self.regs[sr2]);
        }

        self.update_flags(dr);
        Ok(())
    }

    /// Does the bitwise 'NOT' for a value stored in a register.
    ///
    /// ### Arguments
    ///
    /// - `instr`: An u16 that has the encoding of the whole instruction to execute.
    pub fn not(&mut self, instr: u16) -> Result<(), VMError> {
        let dr = Register::from_u16((instr >> 9) & THREE_BIT_MASK)?;
        let sr = Register::from_u16((instr >> 6) & THREE_BIT_MASK)?;

        self.regs[dr] = !self.regs[sr];
        self.update_flags(dr);
        Ok(())
    }

    /// Does the bitwise 'AND' between two values and stores the result
    /// in a register.
    ///
    /// This instruction can be executed in two different ways:
    /// - register mode: Bitwise and with two elements that are in a register each.
    /// - immediate mode: Uses the value of a register with the one that is embedded
    ///   in the instruction itself.
    ///
    /// ### Arguments
    ///
    /// - `instr`: An u16 that has the encoding of the whole instruction to execute.
    /// - `regs`: A Registers struct that handles each register.
    pub fn and(&mut self, instr: u16) -> Result<(), VMError> {
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
            self.regs[dr] = self.regs[sr1] & imm5;
        } else {
            // Get the SR2 section, then do the bitwise and with the content on R1.
            let sr2 = Register::from_u16(instr & THREE_BIT_MASK)?;
            self.regs[dr] = self.regs[sr1] & self.regs[sr2];
        }

        self.update_flags(dr);
        Ok(())
    }

    /// Changes the PC register value depending on the value of the Cond register.
    /// If the Cond register is set to the same value taken from the bits 9 to 11 of the instruction,
    /// this function adds the PC register to the value taken from the first 9 bits of the instruction.
    ///
    /// ### Arguments
    ///
    ///
    /// - `instr`: An u16 that has the encoding of the whole instruction to execute.
    /// - `regs`: A Registers struct that handles each register.
    pub fn branch(&mut self, instr: u16) -> Result<(), VMError> {
        // Get the PCOffset9 section
        let mut pc_offset = instr & NINE_BIT_MASK;
        pc_offset = sign_extend(pc_offset, 9)?;
        // Get the Condition Flag and check if it is the same
        // as the one selected on the instruction
        let cond_flag = (instr >> 9) & THREE_BIT_MASK;
        let coincides = cond_flag & self.regs[Register::Cond];
        if coincides != 0 {
            self.regs[Register::PC] = self.regs[Register::PC].wrapping_add(pc_offset);
        }
        Ok(())
    }

    /// Changes the PC with the value of a register specified in the instruction
    /// itself
    pub fn jump(&mut self, instr: u16) -> Result<(), VMError> {
        // Get the BaseR section
        let baser_r = Register::from_u16((instr >> 6) & THREE_BIT_MASK)?;
        self.regs[Register::PC] = self.regs[baser_r];
        Ok(())
    }

    /// Changes the PC register with the value on a register that is specified
    /// on the R1 section of the encoding or in the same instruction encoding.
    /// This depends on the long flag situated in the bit 11, which is taken by shifting right
    /// the instruction 11 times and checking if it is a 1 or a 0. The long flag
    /// being 1 means it can be a value of eleven bits. If the flag is a 0,
    /// the value is taken from a register.
    pub fn jump_register(&mut self, instr: u16) -> Result<(), VMError> {
        let long_flag = (instr >> 11) & 1;
        self.regs[Register::R7] = self.regs[Register::PC];
        if long_flag == 1 {
            let mut long_pc_offset = instr & ELEVEN_BIT_MASK;
            long_pc_offset = sign_extend(long_pc_offset, 11)?;
            self.regs[Register::PC] = self.regs[Register::PC].wrapping_add(long_pc_offset);
        } else {
            let r1 = Register::from_u16((instr >> 6) & THREE_BIT_MASK)?;
            self.regs[Register::PC] = self.regs[r1];
        }
        Ok(())
    }

    /// Loads a value into a register using indirect addressing.
    /// First, it computes a memory address by adding a PC offset to the current PC,
    /// and reads the value at that memory address. This value is treated as the final memory address.
    /// Then, it loads the value stored at that final memory address into the destination register.
    ///
    /// ### Arguments
    ///
    /// - `instr`: An u16 that has the encoding of the whole instruction to execute.
    /// - `regs`: A Registers struct that handles each register.
    /// - `memory`: A Memory struct that handles reads and writes on the vm memory.
    pub fn load_indirect(&mut self, instr: u16) -> Result<(), VMError> {
        // Destination register
        let dr = Register::from_u16((instr >> 9) & THREE_BIT_MASK)?;
        // PCoffset 9 section
        let mut pc_offset = instr & NINE_BIT_MASK;
        pc_offset = sign_extend(pc_offset, 9)?;
        // Add the number that was on PCoffset 9 section to PC to get the
        // memory location we need to look at for the final address
        let address_of_final_address = self.regs[Register::PC].wrapping_add(pc_offset);
        let final_address = self.mem.read(address_of_final_address)?;
        self.regs[dr] = self.mem.read(final_address)?;
        self.update_flags(dr);
        Ok(())
    }

    /// Loads a value from a location in memory and stores the loaded value into a register
    pub fn load(&mut self, instr: u16) -> Result<(), VMError> {
        // Destination register
        let dr = Register::from_u16((instr >> 9) & THREE_BIT_MASK)?;
        // PCoffset 9 section
        let mut pc_offset = instr & NINE_BIT_MASK;
        pc_offset = sign_extend(pc_offset, 9)?;
        // Calculate the memory address to read
        let address = self.regs[Register::PC].wrapping_add(pc_offset);
        self.regs[dr] = self.mem.read(address)?;
        self.update_flags(dr);
        Ok(())
    }

    /// Loads a value that is located in a memory address.  This value is formed by
    /// adding the value on a register and the one in the offset6 section. Then,
    /// memory is read at this value and that is set into a desired register.
    pub fn load_register(&mut self, instr: u16) -> Result<(), VMError> {
        // Destination Register
        let dr = Register::from_u16((instr >> 9) & THREE_BIT_MASK)?;
        // BaseR section
        let r1 = Register::from_u16((instr >> 6) & THREE_BIT_MASK)?;
        // Offset6 section
        let mut offset6 = instr & SIX_BIT_MASK;
        offset6 = sign_extend(offset6, 6)?;
        // Calculate the memory address to read
        let address = self.regs[r1].wrapping_add(offset6);
        self.regs[dr] = self.mem.read(address)?;
        self.update_flags(dr);
        Ok(())
    }

    /// Loads a value into a register. This value is crated by adding the value of the PC and the
    /// one in the PCoffset9 section, which is formed by the 9 rightmost bits in the intruction encoding.
    pub fn load_effective_address(&mut self, instr: u16) -> Result<(), VMError> {
        // Destination Register
        let dr = Register::from_u16((instr >> 9) & THREE_BIT_MASK)?;
        // PCoffset9 section
        let mut pc_offset = instr & NINE_BIT_MASK;
        pc_offset = sign_extend(pc_offset, 9)?;
        // Set the new value for the destination register
        self.regs[dr] = self.regs[Register::PC].wrapping_add(pc_offset);
        self.update_flags(dr);
        Ok(())
    }

    /// Reads a value from a register and stores it into memory. This address
    /// is created from the addition of the PC and the PCoffset9 section
    pub fn store(&mut self, instr: u16) -> Result<(), VMError> {
        // Source Register
        let sr = Register::from_u16((instr >> 9) & THREE_BIT_MASK)?;
        // PCoffset9 section
        let mut pc_offset = instr & NINE_BIT_MASK;
        pc_offset = sign_extend(pc_offset, 9)?;
        // Calculate the address
        let address = self.regs[Register::PC].wrapping_add(pc_offset);
        let new_val = self.regs[sr];
        self.mem.write(address, new_val)
    }

    /// Reads a value from a register and stores it into memory. This address
    /// is taken indirectly from the instruction. By adding the PC and the PCoffset9 section
    /// (the rightmost 9 bits of the instruction enconding) we get the first memory
    /// address, then if we read it we get the final address. That
    /// final address is the one that is going to get written.
    pub fn store_indirect(
        &mut self,
        instr: u16,
    ) -> Result<(), VMError> {
        // Source Register
        let sr = Register::from_u16((instr >> 9) & THREE_BIT_MASK)?;
        // PCoffset9 section
        let mut pc_offset = instr & NINE_BIT_MASK;
        pc_offset = sign_extend(pc_offset, 9)?;
        // Get the first address
        let first_address = self.regs[Register::PC].wrapping_add(pc_offset);
        // Read the first address, get the second one and write on it
        let final_address = self.mem.read(first_address)?;
        let new_val = self.regs[sr];
        self.mem.write(final_address, new_val)
    }

    /// Reads a value from a register and stores it into memory. By adding
    /// the value on the register specified in the BaseR section and the value in the
    /// offset6 section we get the memory address. That address is the one that
    /// is going to get written.
    ///
    /// The BaseR and the offset6 sections can be found on the instruction enconding.
    /// The first holds a register to use, the second one holds and embedded value.
    pub fn store_register(
        &mut self,
        instr: u16,
    ) -> Result<(), VMError> {
        // Source Register
        let sr = Register::from_u16((instr >> 9) & THREE_BIT_MASK)?;
        // BaseR section
        let r1 = Register::from_u16((instr >> 6) & THREE_BIT_MASK)?;
        // Offset 6 section
        let mut offset = instr & SIX_BIT_MASK;
        offset = sign_extend(offset, 6)?;
        // Calculate the address
        let address = self.regs[r1].wrapping_add(offset);
        let new_val = self.regs[sr];
        self.mem.write(address, new_val)
    }

    /// Executes the desired trap routine. In the instruction encoding the
    /// trapvect8 section can be found in the 8 rightmost bits, and from there
    /// we can get the trap code that will tell us which of the trap routines
    /// we have to execute.
    pub fn trap(
        &mut self,
        instr: u16,
    ) -> Result<(), VMError> {
        self.regs[Register::R7] = self.regs[Register::PC];
        let trap_code = TrapCode::try_from(instr & EIGHT_BIT_MASK)?;
        let mut std_in = stdin().lock();
        let mut std_out = stdout().lock();
        match trap_code {
            TrapCode::GetC => self.get_c(&mut std_in)?,
            TrapCode::Out => self.out(&mut std_out)?,
            TrapCode::Puts => self.puts(&mut std_out)?,
            TrapCode::In => self.trap_in(&mut std_out, &mut std_in)?,
            TrapCode::PutsP => self.puts_p(&mut std_out)?,
            TrapCode::Halt => self.halt(&mut std_out)?,
        }
        Ok(())
    }

    /// Reads one character from the stdin.
    pub fn get_c(&mut self, reader: &mut impl Read) -> Result<(), VMError> {
        let buffer = getchar(reader)?;
        let char: u16 = buffer[0].into();
        self.regs[Register::R0] = char;
        self.update_flags(Register::R0);
        Ok(())
    }

    /// Writes a single character into stdout.
    pub fn out(&mut self, writer: &mut impl Write) -> Result<(), VMError> {
        let c: u8 = self.regs[Register::R0]
            .try_into()
            .map_err(|e: TryFromIntError| VMError::Conversion(e.to_string()))?;
        stdout_write(&[c], writer)?;
        Ok(())
    }

    /// Prompts for input character from the stdin.
    pub fn trap_in(
        &mut self,
        writer: &mut impl Write,
        reader: &mut impl Read,
    ) -> Result<(), VMError> {
        print!("Enter a character: ");
        let buffer = getchar(reader)?;
        stdout_write(&buffer, writer)?;
        stdout_flush(writer)?;
        self.regs[Register::R0] = buffer[0].into();
        self.update_flags(Register::R0);
        Ok(())
    }

    /// Writes a null-terminated string into stdout. The characters are contained in consecutive memory locations,
    /// one character per memory location, starting with the address specified in R0. Writing
    /// terminates with the occurrence of x0000 in a memory location.
    pub fn puts(
        &mut self,
        writer: &mut impl Write,
    ) -> Result<(), VMError> {
        // Get the address of the first character and read it
        let mut c_addr = self.regs[Register::R0];
        let mut c = self.mem.read(c_addr)?;
        while c != NULL {
            // Parse it into a u8, write it and pass to the next memory location
            let char: u8 = c
                .try_into()
                .map_err(|e: TryFromIntError| VMError::Conversion(e.to_string()))?;
            stdout_write(&[char], writer)?;
            c_addr = c_addr.wrapping_add(1);
            c = self.mem.read(c_addr)?;
        }
        stdout_flush(writer)?;
        Ok(())
    }

    /// Writes a null-terminated string into stdout. The characters are contained in consecutive memory locations,
    /// but this time there are two characters per memory location, starting with the address specified in R0. Writing
    /// terminates with the occurrence of x0000 in a memory location.
    pub fn puts_p(
        &mut self,
        writer: &mut impl Write,
    ) -> Result<(), VMError> {
        // Get the address of the first characters and read them
        let mut c_addr = self.regs[Register::R0];
        let mut c = self.mem.read(c_addr)?;
        while c != NULL {
            // Get the first character in the memory location (the 8 leftmost bits)
            let char1 = (c & 0xFF)
                .try_into()
                .map_err(|e: TryFromIntError| VMError::Conversion(e.to_string()))?;
            stdout_write(&[char1], writer)?;
            // Get the second character in the same memory location (the 8 rightmost bits)
            let char2 = (c >> 8)
                .try_into()
                .map_err(|e: TryFromIntError| VMError::Conversion(e.to_string()))?;
            if char2 != 0x00 {
                stdout_write(&[char2], writer)?;
            }
            c_addr = c_addr.wrapping_add(1);
            // Get the next memory location
            c = self.mem.read(c_addr)?;
        }
        stdout_flush(writer)?;
        Ok(())
    }

    /// Writes on stdout th word 'HALT' to notify the user that the program is stopping
    /// and changes the 'running' flag to false. This is the flag that is used in the
    /// main loop to know if the program needs to continue processing instructions or not.
    pub fn halt(&mut self, writer: &mut impl Write) -> Result<(), VMError> {
        let s = "HALT\n".as_bytes();
        stdout_write(s, writer)?;
        stdout_flush(writer)?;
        // Change the flag so the main loop stops
        self.running = false;
        Ok(())
    }
}

impl Default for VM {
    /// Creates a VM instance with all the registers and
    /// memory locations set to 0.
    /// 
    /// This is used for easier testing
    fn default() -> Self {
        Self { mem: Memory::new(), regs: Registers::new(), running: true }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

      

    #[test]
    /// Test if doing the bitwise 'AND' with register mode
    /// gets the correct result
    fn and_with_register_mode() {
        let sr1 = 0xFFFF;
        let sr2 = 0x0000;
        let result = 0x0000;
        // Create the registers and set the values on R1 and R2
        let mut vm = VM::new();
        vm.regs[Register::R1] = sr1;
        vm.regs[Register::R2] = sr2;
        // The instruction will have the following encoding:
        // 0 1 0 1 0 0 0 0 0 1 0 0 0 0 1 0
        let instr = 0x5042;
        let _ = vm.and(instr);

        // Check if in R0 we have the desired result
        assert_eq!(vm.regs[Register::R0], result);
    }

    #[test]
    /// Test if doing the bitwise 'AND' with immediate mode
    /// gets the correct result
    fn and_with_immediate_mode() {
        let sr1 = 0xFFFF;
        let result = 0x0000;
        // Create the registers and set the value on R1
        let mut vm = VM::new();
        vm.regs[Register::R1] = sr1;
        // The instruction will have the following encoding:
        // 0 1 0 1 0 0 0 0 0 1 0 0 0 0 0 0
        let instr = 0x5040;
        let _ = vm.and(instr);

        // Check if in R0 we have the desired result
        assert_eq!(vm.regs[Register::R0], result);
    }

    #[test]
    /// Test if bitwise 'NOT' actually negates all the bits
    /// in a register
    fn bitwise_not_negates_all_bits() {
        let sr = 0xFFFF;
        let result = 0x0000;
        // Create the registers and set the value on R1
        let mut vm = VM::new();
        vm.regs[Register::R1] = sr;
        // The instruction will have the following encoding:
        // 1 0 0 1 0 0 0 0 0 1 1 1 1 1 1 1
        let instr = 0x907F;
        let _ = vm.not(instr);

        // Check if in R0 we have the desired result
        assert_eq!(vm.regs[Register::R0], result);
    }

    #[test]
    /// Test if branch changes the PC for condition
    /// flag set to positive
    fn branch_changes_pc_with_pos_cond_flag() {
        // Create the registers and set the value on register Cond
        let mut vm = VM::default();
        vm.regs[Register::Cond] = CondFlag::Pos.value();
        // The instruction will have the following encoding:
        // 0 0 0 0 0 0 1 0 0 0 0 0 0 0 0 1
        let instr = 0x0201;
        let _ = vm.branch(instr);

        // Check if the PC register was set to 1
        assert_eq!(vm.regs[Register::PC], 0x0001);
    }

    #[test]
    /// Test if branch changes the PC for condition
    /// flag set to zero
    fn branch_changes_pc_with_zro_cond_flag() {
        // Create the registers and set the value on register Cond
        let mut vm = VM::default();
        vm.regs[Register::Cond] = CondFlag::Zro.value();
        // The instruction will have the following encoding:
        // 0 0 0 0 0 1 0 0 0 0 0 0 0 0 0 1
        let instr = 0x0401;
        let _ = vm.branch(instr);

        // Check if the PC register was set to 1
        assert_eq!(vm.regs[Register::PC], 0x0001);
    }

    #[test]
    /// Test if branch changes the PC for condition
    /// flag set to negative
    fn branch_changes_pc_with_neg_cond_flag() {
        // Create the registers and set the value on register Cond
        let mut vm = VM::default();
        vm.regs[Register::Cond] = CondFlag::Neg.value();
        // The instruction will have the following encoding:
        // 0 0 0 0 1 0 0 0 0 0 0 0 0 0 0 1
        let instr = 0x0801;
        let _ = vm.branch(instr);

        // Check if the PC register was set to 1
        assert_eq!(vm.regs[Register::PC], 0x0001);
    }

    #[test]
    /// Test if jump changes the value of the PC is set
    /// to the value of the register specified in the
    /// encoding
    fn jump_changes_pc() {
        // Create the registers and set the value of R1 register
        let mut vm = VM::new();
        let result = 0xFFFF;
        vm.regs[Register::R1] = result;
        // The instruction will have the following encoding:
        // 1 1 0 0  0 0 0 0  0 1 0 0  0 0 0 0
        let instr = 0xC040;
        let _ = vm.jump(instr);

        // Check if the PC was set with the value that R1 had
        assert_eq!(vm.regs[Register::PC], result);
    }

    #[test]
    /// Test if the PC register adds the desired value to the PC
    /// when using the jump register instruction with the long
    /// flag set. With this flag being set, means we have 11 bits
    /// to choose a value for our PC register.
    fn jump_register_adds_pc_with_long_flag() {
        let mut vm = VM::default();
        let result = 0x03FF;
        // The instruction will have the following encoding:
        // 0 1 0 0  1 0 1 1  1 1 1 1  1 1 1 1
        let instr = 0x4BFF;
        let _ = vm.jump_register(instr);

        // Check if the PC register was set to the value
        // embedded on the encoding
        assert_eq!(vm.regs[Register::PC], result);
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
        let mut vm = VM::new();
        vm.regs[Register::PC] = 0x000A;
        let result = 0x0007;
        // The instruction will have the following encoding:
        // 0 1 0 0  1 1 1 1  1 1 1 1  1 1 0 1
        let instr = 0x4FFD;
        let _ = vm.jump_register(instr);

        // Check if the PC register was set to the value
        // embedded on the encoding
        assert_eq!(vm.regs[Register::PC], result);
    }

    #[test]
    /// Test if the PC register changes to the desired value
    /// when using the jum register instruction without the long
    /// flag set. This means that the value for the PC register
    /// will be take from one of the registers.
    fn jump_register_changes_pc_without_long_flag() {
        // Create the registers and set the for the PC on R1
        let mut vm = VM::new();
        let result = 0x07FF;
        vm.regs[Register::R1] = result;
        // The instruction will have the following encoding:
        // 0 1 0 0  0 0 0 0  0 1 0 0  0 0 0 0
        let instr = 0x4040;
        let _ = vm.jump_register(instr);

        // Check if the PC register was set to the value
        // embedded on the encoding
        assert_eq!(vm.regs[Register::PC], result);
    }

    #[test]
    /// Test if the jump register instruction stores the PC
    /// value into the R7 register before it is changed to a
    /// new value.
    fn jump_register_stores_pc_on_r7() {
        // Create the registers and set the value of the PC
        let result = 0x000A;
        let mut vm = VM::new();
        vm.regs[Register::PC] = result;

        // Check if R7 has a different value before calling the instruction
        assert_ne!(vm.regs[Register::R7], result);
        // Run the instruction
        let _ = vm.jump_register(0x4040);
        // Check if R7 changed its value to the one the PC had
        assert_eq!(vm.regs[Register::R7], result);
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
        let mut vm = VM::new();
        let first_address: u16 = 0x000F;
        let result_address = 0x0014;
        let result = 0x0001;
        let _ = vm.mem.write(first_address, result_address);
        let _ = vm.mem.write(result_address, result);
        // Set the value of pc to 10.
        vm.regs[Register::PC] = 0x000A;
        // The instruction will have the following encoding:
        // 1 0 1 0  0 0 1 0  0 0 0 0  0 1 0 1
        let instr = 0xA205;
        let _ = vm.load_indirect(instr);

        // Check if R1 has the value that was on memory in 'result_address'
        assert_eq!(vm.regs[Register::R1], result);
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
        let mut vm = VM::new();
        let result = 0x0001;
        let address: u16 = 0x000F;
        let _ = vm.mem.write(address, result);
        // Set the value of PC to 10.
        vm.regs[Register::PC] = 0x000A;
        // The instruction will have the following encoding:
        // 0 0 1 0  0 0 1 0  0 0 0 0  0 1 0 1
        let instr = 0x2205;
        let _ = vm.load(instr);

        // Check if R1 has the value that was on memory in 'result_address'
        assert_eq!(vm.regs[Register::R1], result);
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
        let mut vm = VM::new();
        let result = 0x0001;
        let address: u16 = 0x000F;
        let _ = vm.mem.write(address, result);
        // Set the value of pc to 10.
        vm.regs[Register::R0] = 0x000A;
        // The instruction will have the following encoding:
        // 0 1 1 0  0 0 1 0  0 0 0 0  0 1 0 1
        let instr = 0x6205;
        let _ = vm.load_register(instr);

        // Check if R1 has the value that was on memory in 'result_address'
        assert_eq!(vm.regs[Register::R1], result);
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
        let mut vm = VM::new();
        vm.regs[Register::PC] = 0x000A;
        let result: u16 = 0x000A + 0x0005;
        // The instruction will have the following encoding:
        // 1 1 1 0  0 0 1 0  0 0 0 0  0 1 0 1
        let instr = 0x6205;
        let _ = vm.load_effective_address(instr);

        // Check if R1 has the value of PC + PCoffset9
        assert_eq!(vm.regs[Register::R1], result);
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
        // Create the registers and set the values for R1 and PC
        let mut vm = VM::new();
        vm.regs[Register::PC] = 0x0005;
        vm.regs[Register::R1] = 0x0001;
        // The instruction will have the following encoding:
        // 0 0 1 1  0 0 1 0  0 0 0 0  0 1 0 1
        let instr = 0x3205;
        let _ = vm.store(instr);

        // Check if memory[PC + PCoffset9] = vm.regs[R1]
        assert_eq!(
            vm.mem.read(affected_address).unwrap(),
            vm.regs[Register::R1]
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
        // Set the value of the address 0x000A to 0x000F
        let mut vm = VM::new();
        let first_address: u16 = 0x000A;
        let final_address: u16 = 0x000F;
        let _ = vm.mem.write(first_address, final_address);
        // Set the values of PC and R1
        vm.regs[Register::R1] = 0x0001;
        vm.regs[Register::PC] = 0x0005;
        // The instruction will have the following encoding:
        // 1 0 1 1  0 0 1 0  0 0 0 0  0 1 0 1
        let instr = 0xB205;
        let _ = vm.store_indirect(instr);

        // Check if 0x000F has the value of register R1
        assert_eq!(vm.mem.read(final_address).unwrap(), vm.regs[Register::R1]);
    }

    #[test]
    /// Test if store register instruction changes the value in memory.
    ///
    /// Register 1 will have value 1, register 0 and offset6 section will both
    /// have value 5. The address = offset6 + register 0 = 0x000A will be written
    /// with the value of register 1.
    fn store_register_changes_value_in_memory() {
        let mut vm = VM::new();
        // Set the values of R1 and R0
        vm.regs[Register::R0] = 0x0005;
        vm.regs[Register::R1] = 0x0001;
        // The instruction will have the following encoding:
        // 0 1 1 1  0 0 1  0 0 0  0 0 0 1 0 1
        let instr = 0x7205;
        let _ = vm.store_register(instr);

        // Check if address 0x000A = R0 + offset6 was written with R1's value
        let affected_address: u16 = 0x000A;
        assert_eq!(
            vm.mem.read(affected_address).unwrap(),
            vm.regs[Register::R1]
        );
    }

    #[test]
    /// Test if by using the trap instruction, the register R7 gets the value
    /// of the PC
    fn trap_sets_register_7_with_pc_value() {
        let mut vm = VM::new();
        let pc_val = 0x0A0A;
        vm.regs[Register::PC] = pc_val;
        // The instruction will have the following encoding:
        // 1 1 1 1  0 0 0 0  0 0 1 0  0 1 0 1
        let instr = 0xF025;

        let _ = vm.trap(instr);

        assert_eq!(vm.regs[Register::R7], pc_val);
    }

    #[test]
    /// Test trap instruction calls the halt trap routine
    fn trap_calls_correct_subroutine() {
        let mut vm = VM::new();
        let pc_val = 0x0A0A;
        vm.regs[Register::PC] = pc_val;
        // The instruction will have the following encoding:
        // 1 1 1 1  0 0 0 0  0 0 1 0  0 1 0 1
        let instr = 0xF025;

        let _ = vm.trap(instr);

        // The running flag should change to false
        assert!(!vm.running);
    }

    #[test]
    /// Test if the data is written in the memory, starting from
    /// the indicated address and with the data in the correct
    /// endianess
    fn read_image_file_writes_memory_correctly() {
        let mut vm = VM::new();
        let mut data: Vec<u8> = vec![0xFA, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06];
        let _ = vm.read_image_file(&mut data).unwrap();

        let origin = 0xFA00;
        assert_eq!(vm.mem.read(origin).unwrap(), 0x0102);
        assert_eq!(vm.mem.read(origin + 1).unwrap(), 0x0304);
        assert_eq!(vm.mem.read(origin + 2).unwrap(), 0x0506);
    }

    #[test]
    /// Test if the data is written in the memory, starting from
    /// the indicated address and with the data in the correct
    /// endianess when opening a file
    fn read_image_reads_file_correctly_into_memory() {
        let mut vm = VM::new();

        let _ = vm.read_image("test_files/bytes.bin".to_string());

        let origin = 0xFA00;
        assert_eq!(vm.mem.read(origin).unwrap(), 0x0102);
        assert_eq!(vm.mem.read(origin + 1).unwrap(), 0x0304);
        assert_eq!(vm.mem.read(origin + 2).unwrap(), 0x0506);
    }

    #[test]
    /// Test result when adding two values in register mode.
    /// One value will be in R1 and the other in R2, while
    /// the destination register will be R0.
    fn add_with_register_mode() {
        let sr1 = 0x0001;
        let sr2 = 0x0002;
        let result = 0x0003;
        // Create the registers and set the values on R1 and R2
        let mut vm = VM::new();
        vm.regs[Register::R1] = sr1;
        vm.regs[Register::R2] = sr2;
        // The instruction will have the following encoding:
        // 0 0 0 1 0 0 0 0 0 1 0 0 0 0 1 0
        let instr = 0x1042;
        let _ = vm.add(instr);

        // Check if in R0 we have the desired result
        assert_eq!(vm.regs[Register::R0], result);
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
        let mut vm = VM::new();
        vm.regs[Register::R1] = sr1;
        // The instruction will have the following encoding:
        // 0 0 0 1 0 0 0 0 0 1 1 0 0 0 1 0
        let instr = 0x1062;
        let _ = vm.add(instr);

        // Check if in R0 we have the desired result
        assert_eq!(vm.regs[Register::R0], result);
    }

    #[test]
    /// Test result when adding one positve value with a
    /// negative one. To do this adding we need to use immediate mode.
    fn add_with_negative_value() {
        let sr1 = 0x0001;
        let result = 0x000;
        // Create the registers and set the value on R1
        let mut vm = VM::new();
        vm.regs[Register::R1] = sr1;
        // The instruction will have the following encoding:
        // 0 0 0 1 0 0 0 0 0 1 1 1 1 1 1 1
        let instr = 0x107F;
        let _ = vm.add(instr);

        // Check if in R0 we have the desired result
        assert_eq!(vm.regs[Register::R0], result);
    }

    #[test]
    /// Test if the condition flag was set to POS when the
    /// result of an addition is a positive number.
    fn add_updates_cond_flag_to_pos() {
        let sr1 = 0x0001;
        let sr2 = 0x0002;
        // Create the registers and set the values on R1 and R2
        let mut vm = VM::new();
        vm.regs[Register::R1] = sr1;
        vm.regs[Register::R2] = sr2;
        // The instruction will have the following encoding:
        // 0 0 0 1 0 0 0 0 0 1 0 0 0 0 1 0
        let instr = 0x1042;
        let _ = vm.add(instr);

        assert_eq!(vm.regs[Register::Cond], CondFlag::Pos.value());
    }

    #[test]
    /// Test if the condition flag was set to ZRO when the
    /// result of an addition is a 0.
    fn add_updates_cond_flag_to_zro() {
        let sr1 = 0x0001;
        // Create the registers and set the value on R1
        let mut vm = VM::new();
        vm.regs[Register::R1] = sr1;
        // The instruction will have the following encoding:
        // 0 0 0 1 0 0 0 0 0 1 1 1 1 1 1 1
        let instr = 0x107F;
        let _ = vm.add(instr);

        assert_eq!(vm.regs[Register::Cond], CondFlag::Zro.value());
    }

    #[test]
    fn get_c_modifies_register_0_with_input_char() {
        let char = "c";
        let char_bytes: u16 = char.as_bytes()[0].into();
        let mut reader = Cursor::new(char);
        let mut vm = VM::new();
        let _ = vm.get_c(&mut reader);

        assert_eq!(vm.regs[Register::R0], char_bytes);
    }

    #[test]
    fn trap_out_writes_register_0_value() {
        let char = "c";
        let char_bytes: u16 = char.as_bytes()[0].into();
        let mut writer: Vec<u8> = Vec::new();
        let mut vm = VM::new();
        vm.regs[Register::R0] = char_bytes;
        let _ = vm.out(&mut writer);

        let written_val: u16 = writer[0].into();
        assert_eq!(written_val, char_bytes);
    }

    #[test]
    fn trap_in_writes_register_0_with_reader_value() {
        let char = "c";
        let char_bytes: u16 = char.as_bytes()[0].into();
        let mut reader = Cursor::new(char);
        let mut writer: Vec<u8> = Vec::new();
        let mut vm = VM::new();
        let _ = vm.trap_in(&mut writer, &mut reader);

        assert_eq!(vm.regs[Register::R0], char_bytes);
    }

    #[test]
    fn trap_in_writes_writer_with_value() {
        let char = "c";
        let char_bytes: u16 = char.as_bytes()[0].into();
        let mut reader = Cursor::new(char);
        let mut writer: Vec<u8> = Vec::new();
        let mut vm = VM::new();
        let _ = vm.trap_in(&mut writer, &mut reader);

        let written_val: u16 = writer[0].into();
        assert_eq!(written_val, char_bytes);
    }

    #[test]
    fn puts_writes_whole_string_on_writer() {
        let mut writer: Vec<u8> = Vec::new();
        let (char1, char2, char3) = ("1", "2", "3");
        let char1_bytes: u16 = char1.as_bytes()[0].into();
        let char2_bytes: u16 = char2.as_bytes()[0].into();
        let char3_bytes: u16 = char3.as_bytes()[0].into();
        let starting_address: u16 = 0x0005;

        let mut vm = VM::new();
        vm.regs[Register::R0] = starting_address;
        let _ = vm.mem.write(starting_address, char1_bytes);
        let _ = vm.mem.write(starting_address + 1, char2_bytes);
        let _ = vm.mem.write(starting_address + 2, char3_bytes);

        let _ = vm.puts(&mut writer);

        let written_val_1: u16 = writer[0].into();
        let written_val_2: u16 = writer[1].into();
        let written_val_3: u16 = writer[2].into();
        assert_eq!(written_val_1, char1_bytes);
        assert_eq!(written_val_2, char2_bytes);
        assert_eq!(written_val_3, char3_bytes);
    }

    #[test]
    fn halt_changes_bool() {
        let mut vm = VM::new();
        let mut writer: Vec<u8> = Vec::new();

        let _ = vm.halt(&mut writer);

        assert!(!vm.running);
    }

    #[test]
    fn halt_writes_on_writer() {
        let mut vm = VM::new();
        let mut writer: Vec<u8> = Vec::new();
        let expected_bytes = "HALT\n".as_bytes();
        let _ = vm.halt(&mut writer);

        assert_eq!(writer, expected_bytes);
    }

    #[test]
    fn puts_p_run_writes_on_writer() {
        let mut writer: Vec<u8> = Vec::new();
        let (char1, char2, char3, char4) = ("1", "2", "3", "4");
        let char1_bytes: u16 = char1.as_bytes()[0].into();
        let char2_bytes: u16 = char2.as_bytes()[0].into();
        let char3_bytes: u16 = char3.as_bytes()[0].into();
        let char4_bytes: u16 = char4.as_bytes()[0].into();

        let memory_location1: u16 = (char2_bytes << 8) | char1_bytes;
        let memory_location2: u16 = (char4_bytes << 8) | char3_bytes;

        let starting_address: u16 = 0x0005;
        let mut vm = VM::new();
        vm.regs[Register::R0] = starting_address;
        let _ = vm.mem.write(starting_address, memory_location1);
        let _ = vm.mem.write(starting_address + 1, memory_location2);

        let _ = vm.puts_p(&mut writer);

        let written_val_1: u16 = writer[0].into();
        let written_val_2: u16 = writer[1].into();
        let written_val_3: u16 = writer[2].into();
        let written_val_4: u16 = writer[3].into();
        assert_eq!(written_val_1, char1_bytes);
        assert_eq!(written_val_2, char2_bytes);
        assert_eq!(written_val_3, char3_bytes);
        assert_eq!(written_val_4, char4_bytes);
    }
}
