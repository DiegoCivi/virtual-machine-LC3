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

/// Gets one character from the stdin
fn get_c(regs: &mut Registers, reader: &mut impl Read) -> Result<(), VMError> {
    let buffer = getchar(reader)?;
    let char: u16 = buffer[0].into();
    regs[Register::R0] = char;
    update_flags(Register::R0, regs);
    Ok(())
}

fn trap_out(regs: &mut Registers, writer: &mut impl Write) -> Result<(), VMError> {
    let c: u8 = regs[Register::R0].try_into().map_err(|_| VMError::Conversion)?;
    stdout_write(&[c], writer)?;
    Ok(())
}

fn trap_in(regs: &mut Registers, writer: &mut impl Write, reader: &mut impl Read) -> Result<(), VMError> {
    print!("Enter a character: ");
    let buffer = getchar(reader)?;
    stdout_write(&buffer, writer)?;
    stdout_flush(writer)?;
    regs[Register::R0] = buffer[0].try_into().map_err(|_| VMError::Conversion)?;
    update_flags(Register::R0, regs);
    Ok(())
}

fn puts(regs: &mut Registers, mem: &mut Memory, writer: &mut impl Write) -> Result<(), VMError> {
    let mut c_addr = regs[Register::R0];
    let mut c = mem.read(c_addr)?;
    while c != NULL {
        let char: u8 = c.try_into().map_err(|_| VMError::Conversion)?;
        stdout_write(&[char], writer)?;
        c_addr += 1;
        c = mem.read(c_addr)?;
    }
    stdout_flush(writer)?;
    Ok(())
}

fn puts_p(regs: &mut Registers, mem: &mut Memory, writer: &mut impl Write) -> Result<(), VMError> {
    let mut c_addr = regs[Register::R0];
    let mut c = mem.read(c_addr)?;
    while c != NULL {
        let char1 = (c & 0xFF).try_into().map_err(|_| VMError::Conversion)?;
        stdout_write(&[char1], writer)?;
        let char2 = (c >> 8).try_into().map_err(|_| VMError::Conversion)?;
        if char2 != 0x00 {
            stdout_write(&[char2], writer)?;
        }
        c_addr += 1;
        c = mem.read(c_addr)?;
    }
    stdout_flush(writer)?;
    Ok(())
}

fn halt(mut running: bool, writer: &mut impl Write) -> Result<(), VMError> {
    let s = "HALT\n".as_bytes();
    stdout_write(s, writer)?;
    stdout_flush(writer)?;
    running = false;
    Ok(())
}
