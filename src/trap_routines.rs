use std::{char, io::{self, Write}};
use crate::{error::VMError, hardware::{Memory, Register, Registers}, utils::{getchar, read_byte, stdout_flush, stdout_write, update_flags}};
use libc::*;

const NULL: u8 = 0x000;

enum TrapCode {
    GetC = 0x20,
    Out = 0x21,
    Puts = 0x22,
    In = 0x23,
    PutsP = 0x24,
    Halt = 0x25,
}

// fn puts(regs: &mut Registers) -> Result<(), VMError> {

// }

fn get_c(regs: &mut Registers) -> Result<(), VMError> {
    let buffer = getchar()?;
    let char: u16 = buffer[0].into();
    regs[Register::R0] = char;
    update_flags(Register::R0, regs);
    Ok(())
}

fn trap_out(regs: &mut Registers) -> Result<(), VMError> {
    let c: u8 = regs[Register::R0].try_into().map_err(|_| VMError::Conversion)?;
    stdout_write(&[c])?;
    Ok(())
}

fn trap_in(regs: &mut Registers) -> Result<(), VMError> {
    print!("Enter a character: ");
    let buffer = getchar()?;
    stdout_write(&buffer);
    stdout_flush()?;
    regs[Register::R0] = buffer[0].try_into().map_err(|_| VMError::Conversion)?;
    update_flags(Register::R0, regs);
    Ok(())
}

fn puts(regs: &mut Registers, mem: &mut Memory) -> Result<(), VMError> {
    let mut c_addr = regs[Register::R0];
    let mut c = read_byte(c_addr, mem)?;
    while c != NULL {
        stdout_write(&[c])?;
        c_addr += 1;
        c = read_byte(c_addr, mem)?;
    }
    stdout_flush()?;
    Ok(())
}

fn puts_p(regs: &mut Registers, mem: &mut Memory) -> Result<(), VMError> {
    let mut c_addr = regs[Register::R0];
    let mut c = read_byte(c_addr, mem)?;
    while c != NULL {
        let char1 = c & 0xFF;
        stdout_write(&[char1])?;
        let char2 = c >> 8;
        if char2 != NULL {
            stdout_write(&[char2])?;
        }
        c_addr += 1;
        c = read_byte(c_addr, mem)?;
    }
    stdout_flush()?;
    Ok(())
}

fn halt(mut running: bool) -> Result<(), VMError> {
    let s = "HALT\n".as_bytes();
    stdout_write(s)?;
    stdout_flush()?;
    running = false;
    Ok(())
}
