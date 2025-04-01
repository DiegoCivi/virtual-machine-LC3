use std::{
    env::Args,
    fs::File,
    io::{self, Read, Write, stdin},
    mem,
    os::fd::{AsFd, AsRawFd},
};

use termios::ffi::tcsetattr;

use crate::{
    error::VMError,
    hardware::{CondFlag, MEMORY_MAX, Memory, Register, Registers},
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

pub fn load_arguments(args: Args, mem: &mut Memory) -> Result<(), VMError> {
    let args_quantiy = args.len();
    if args_quantiy < 2 {
        println!("lc3 [image-file1] ...");
        return Err(VMError::ShellMisuse);
    }

    for image in args {
        if let Err(e) = read_image(image.clone(), mem) {
            println!("failed to load image: {:?}", image);
            return Err(e);
        }
    }

    Ok(())
}

/// Reads a image and loads it into memory.
///
/// The first 16 bits of the program file specify the address in memory
/// where the program should start. This address is called the origin.
/// After that we find the data that is in bif endian, that is why
/// after reading it we need to pass it to little endian.
pub fn read_image(image: String, mem: &mut Memory) -> Result<(), VMError> {
    let mut file = File::open(image).map_err(|_| VMError::OpenFile)?;
    // Get the origin from the first 2 bytes and swap it
    let mut origin_buffer: [u8; 2] = [0; 2];
    file.read_exact(&mut origin_buffer)
        .map_err(|_| VMError::ReadFile)?;
    let origin: usize = join_bytes(origin_buffer[1].into(), origin_buffer[0].into()).into();

    // Define the maximum read we are capable of
    let max_read: usize = MEMORY_MAX.saturating_sub(origin);

    // Read the whole file into a buffer
    let mut file_vec_buffer = Vec::with_capacity(max_read);
    let mut file_buffer = file_vec_buffer.as_mut_slice();
    file.read_exact(&mut file_buffer)
        .map_err(|_| VMError::ReadFile)?;

    // Iter the file content, get the u16 memory locations by adding the
    // u8 and the swap them so we get the little endian format
    let mut mem_addr = origin;
    for chunk in file_buffer.chunks(2) {
        let mem_data = join_bytes(chunk[1].into(), chunk[0].into());
        mem.write(mem_addr, mem_data)?;
        mem_addr = mem_addr.wrapping_add(1);
    }

    Ok(())
}

/// Makes a 
fn swap16(num: u16) -> u16 {
    (num << 8) | (num >> 8)
}

fn join_bytes(byte1: u16, byte2: u16) -> u16 {
    let leftmost_byte: u16 = byte1 << 8;
    let rightmost_byte: u16 = byte2.into();
    let joined: u16 = leftmost_byte | rightmost_byte;
    joined
}
