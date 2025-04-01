use std::{char, fs::read, io::{self, Read, Write}};
use crate::{error::VMError, hardware::{Memory, Register, Registers}, utils::{getchar, stdout_flush, stdout_write, update_flags}};

const NULL: u16 = 0x0000;

enum TrapCode {
    GetC = 0x20,
    Out = 0x21,
    Puts = 0x22,
    In = 0x23,
    PutsP = 0x24,
    Halt = 0x25,
}

/// Read one character from the stdin.
fn get_c(regs: &mut Registers, reader: &mut impl Read) -> Result<(), VMError> {
    let buffer = getchar(reader)?;
    let char: u16 = buffer[0].into();
    regs[Register::R0] = char;
    update_flags(Register::R0, regs);
    Ok(())
}

/// Output a single character.
fn out(regs: &mut Registers, writer: &mut impl Write) -> Result<(), VMError> {
    let c: u8 = regs[Register::R0].try_into().map_err(|_| VMError::Conversion)?;
    stdout_write(&[c], writer)?;
    Ok(())
}

/// Prompt for input character from the user.
fn trap_in(regs: &mut Registers, writer: &mut impl Write, reader: &mut impl Read) -> Result<(), VMError> {
    print!("Enter a character: ");
    let buffer = getchar(reader)?;
    stdout_write(&buffer, writer)?;
    stdout_flush(writer)?;
    regs[Register::R0] = buffer[0].try_into().map_err(|_| VMError::Conversion)?;
    update_flags(Register::R0, regs);
    Ok(())
}

/// Output a null-terminated string. The characters are contained in consecutive memory locations, 
/// one character per memory location, starting with the address specified in R0. Writing 
/// terminates with the occurrence of x0000 in a memory location.
fn puts(regs: &mut Registers, mem: &mut Memory, writer: &mut impl Write) -> Result<(), VMError> {
    // Get the address of the first character and read it
    let mut c_addr = regs[Register::R0];
    let mut c = mem.read(c_addr)?;
    while c != NULL {
        // Parse it into a u8, write it and pass to the next memory location
        let char: u8 = c.try_into().map_err(|_| VMError::Conversion)?;
        stdout_write(&[char], writer)?;
        c_addr = c_addr.wrapping_add(1);
        c = mem.read(c_addr)?;
    }
    stdout_flush(writer)?;
    Ok(())
}

/// Output a null-terminated string. The characters are contained in consecutive memory locations, 
/// but this time there are two characters per memory location, starting with the address specified in R0. Writing 
/// terminates with the occurrence of x0000 in a memory location.
fn puts_p(regs: &mut Registers, mem: &mut Memory, writer: &mut impl Write) -> Result<(), VMError> {
    // Get the address of the first characters and read them
    let mut c_addr = regs[Register::R0];
    let mut c = mem.read(c_addr)?;
    while c != NULL {
        // Get the first character in the memory location (the 8 leftmost bits)
        let char1 = (c & 0xFF).try_into().map_err(|_| VMError::Conversion)?;
        stdout_write(&[char1], writer)?;
        // Get the second character in the same memory location (the 8 rightmost bits)
        let char2 = (c >> 8).try_into().map_err(|_| VMError::Conversion)?;
        if char2 != 0x00 {
            stdout_write(&[char2], writer)?;
        }
        c_addr = c_addr.wrapping_add(1);
        // Get the next memory location
        c = mem.read(c_addr)?;
    }
    stdout_flush(writer)?;
    Ok(())
}

/// Halt program
fn halt(running: &mut bool, writer: &mut impl Write) -> Result<(), VMError> {
    let s = "HALT\n".as_bytes();
    stdout_write(s, writer)?;
    stdout_flush(writer)?;
    // Change the flag so the main loop stops
    *running = false;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::hardware::{Register, Registers};

    use super::*;

    #[test]
    fn get_c_modifies_R0_with_input_char() {
        let char = "c";
        let char_bytes: u16 = char.as_bytes()[0].into();
        let mut reader = Cursor::new(char);
        let mut regs = Registers::new();
        let _ = get_c(&mut regs, &mut reader);

        assert_eq!(regs[Register::R0], char_bytes);
    }

    #[test]
    fn trap_out_writes_R0_value() {
        let char = "c";
        let char_bytes: u16 = char.as_bytes()[0].into();
        let mut writer: Vec<u8> = Vec::new();
        let mut regs = Registers::new();
        regs[Register::R0] = char_bytes;
        let _ = out(&mut regs, &mut writer);

        let written_val: u16 = writer[0].into();
        assert_eq!(written_val, char_bytes);
    }

    #[test]
    fn trap_in_writes_R0_with_reader_value() {
        let char = "c";
        let char_bytes: u16 = char.as_bytes()[0].into();
        let mut reader = Cursor::new(char);
        let mut writer: Vec<u8> = Vec::new();
        let mut regs = Registers::new();
        let _ = trap_in(&mut regs, &mut writer, &mut reader);

        assert_eq!(regs[Register::R0], char_bytes);
    }

    #[test]
    fn trap_in_writes_writer_with__value() {
        let char = "c";
        let char_bytes: u16 = char.as_bytes()[0].into();
        let mut reader = Cursor::new(char);
        let mut writer: Vec<u8> = Vec::new();
        let mut regs = Registers::new();
        let _ = trap_in(&mut regs, &mut writer, &mut reader);

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

        let mut regs = Registers::new();
        regs[Register::R0] = starting_address;
        let mut mem = Memory::new();
        let _ = mem.write(starting_address, char1_bytes);
        let _ = mem.write(starting_address + 1, char2_bytes);
        let _ = mem.write(starting_address + 2, char3_bytes);

        let _ = puts(&mut regs, &mut mem, &mut writer);

        let written_val_1: u16 = writer[0].into();
        let written_val_2: u16 = writer[1].into();
        let written_val_3: u16 = writer[2].into();
        assert_eq!(written_val_1, char1_bytes);
        assert_eq!(written_val_2, char2_bytes);
        assert_eq!(written_val_3, char3_bytes);
    }

    #[test]
    fn halt_changes_bool() {
        let mut writer: Vec<u8> = Vec::new();
        let mut running = true;

        let _ = halt(&mut running, &mut writer);

        assert!(!running);
    }

    #[test]
    fn halt_writes_on_writer() {
        let mut writer: Vec<u8> = Vec::new();
        let mut running = true;
        let expected_bytes = "HALT\n".as_bytes();
        let _ = halt(&mut running, &mut writer);

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
        let mut regs = Registers::new();
        regs[Register::R0] = starting_address;
        let mut mem = Memory::new();
        let _ = mem.write(starting_address, memory_location1);
        let _ = mem.write(starting_address + 1, memory_location2);
    
        let _ = puts_p(&mut regs, &mut mem, &mut writer);


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
