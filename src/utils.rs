use std::{io::{self, stdin, Read, Write}, os::fd::AsRawFd};

use termios::{tcsetattr, Termios, ECHO, ICANON, TCSANOW};

use crate::{
    error::VMError,
    hardware::{CondFlag, Memory, Register, Registers},
};

/// Extends a number represented in 'bit_count' bits into
/// 16 bits, always taking into account the sign of
/// the original number
pub fn sign_extend(mut x: u16, bit_count: usize) -> Result<u16, VMError> {
    // Get MSB and check if it is a 1
    let bitcount_sub = bit_count.checked_sub(1).ok_or(VMError::Arithmetic)?;
    let msb = x >> bitcount_sub;
    if msb != 0 {
        // If the MSB is 1 it means it is negative, else its positive
        x |= 0xFFFF << bit_count;
    }
    Ok(x)
}

/// It updates the register COND where we have the condition flag
pub fn update_flags(r: Register, regs: &mut Registers) {
    if regs[r] == 0 {
        regs[Register::Cond] = CondFlag::Zro.value();
    } else if regs[r] >> 15 == 1 {
        regs[Register::Cond] = CondFlag::Neg.value();
    } else {
        regs[Register::Cond] = CondFlag::Pos.value();
    }
}

/// Reads one byte from the standard input
pub fn getchar(reader: &mut impl Read) -> Result<[u8; 1], VMError> {
    let mut buffer = [0u8; 1];
    reader
        .read_exact(&mut buffer)
        .map_err(|_| VMError::STDINRead)?;
    Ok(buffer)
}

/// Checks if there is something to read on the standard input
///
/// ### Returns
///
/// A bool indicating where there is something to read or not
pub fn check_key() -> bool {
    let mut stdin = stdin().bytes().peekable();
    match stdin.peek() {
        Some(Ok(n)) => {
            if *n > 0 {
                return true;
            }
            false
        }
        _ => false,
    }
}

/// Flushes the writer
///
/// ### Returns
///
/// A Result indicating if the flushing succeded or not
pub fn stdout_flush(writer: &mut impl Write) -> Result<(), VMError> {
    writer.flush().map_err(|_| VMError::STDOUTFlush)?;
    Ok(())
}

/// Writes the buffer into the writer
///
/// ### Returns
///
/// A Result indicating if the writting succeded or not
pub fn stdout_write(buffer: &[u8], writer: &mut impl Write) -> Result<(), VMError> {
    writer.write_all(buffer).map_err(|_| VMError::STDOUTWrite)?;
    Ok(())
}

pub fn setup() -> Result<Termios, VMError> {
    // Handle interrupt
    /* TODO! */
    disable_input_buffering()
}

pub fn shutdown(initial_termios: Termios) -> Result<(), VMError> {
    let stdin_fd = stdin().lock().as_raw_fd();
    tcsetattr(stdin_fd, TCSANOW, &initial_termios).map_err(|_| VMError::TermiosSetup)?;
    Ok(())
}

fn disable_input_buffering() -> Result<Termios, VMError> {
    let stdin_fd = stdin().lock().as_raw_fd();
    let mut initial_termios = Termios::from_fd(stdin_fd).map_err(|_| VMError::TermiosCreation)?;
    tcsetattr(stdin_fd, TCSANOW, &initial_termios).map_err(|_| VMError::TermiosSetup)?;
    initial_termios.c_lflag &= !ICANON & !ECHO;
    tcsetattr(stdin_fd, TCSANOW, &initial_termios).map_err(|_| VMError::TermiosSetup)?;
    Ok(initial_termios)
}
