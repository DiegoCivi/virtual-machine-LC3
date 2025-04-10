use std::{env::Args, fs, io::Error, process::exit};

use crate::{
    error::VMError,
    hardware::{CondFlag, Memory, OpCode, Register, Registers},
    instructions::*,
};

const PC_START: u16 = 0x3000;

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
                OpCode::Br => branch(instr, &mut self.regs)?,
                OpCode::Add => add(instr, &mut self.regs)?,
                OpCode::Ld => load(instr, &mut self.regs, &mut self.mem)?,
                OpCode::St => store(instr, &mut self.regs, &mut self.mem)?,
                OpCode::Jsr => jump_register(instr, &mut self.regs)?,
                OpCode::And => and(instr, &mut self.regs)?,
                OpCode::Ldr => load_register(instr, &mut self.regs, &mut self.mem)?,
                OpCode::Str => store_register(instr, &mut self.regs, &mut self.mem)?,
                OpCode::Not => not(instr, &mut self.regs)?,
                OpCode::Ldi => load_indirect(instr, &mut self.regs, &mut self.mem)?,
                OpCode::Sti => store_indirect(instr, &mut self.regs, &mut self.mem)?,
                OpCode::Jmp => jump(instr, &mut self.regs)?,
                OpCode::Lea => load_effective_address(instr, &mut self.regs)?,
                OpCode::Trap => trap(instr, &mut self.regs, &mut self.mem, &mut self.running)?,
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
