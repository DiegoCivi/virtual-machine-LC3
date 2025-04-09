use std::{
    env::Args,
    fs,
    io::{Error, Read, Write, stdin},
    os::fd::AsRawFd,
    process::exit,
};

use termios::{ECHO, ICANON, TCSANOW, Termios, tcsetattr};

use crate::{
    error::VMError,
    hardware::{CondFlag, Memory, Register, Registers},
};

/// Extends a number represented in 'bit_count' bits into
/// 16 bits, always taking into account the sign of
/// the original number
pub fn sign_extend(mut x: u16, bit_count: usize) -> Result<u16, VMError> {
    // Get MSB and check if it is a 1
    let bitcount_sub = bit_count
        .checked_sub(1)
        .ok_or(VMError::Arithmetic(String::from(
            "Underflow when substracting",
        )))?;
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
        .map_err(|e: Error| VMError::STDINRead(e.to_string()))?;
    Ok(buffer)
}

/// Flushes the writer
///
/// ### Returns
///
/// A Result indicating if the flushing succeded or not
pub fn stdout_flush(writer: &mut impl Write) -> Result<(), VMError> {
    writer
        .flush()
        .map_err(|_| VMError::STDOUTFlush(String::from("Cannot flush stdout")))?;
    Ok(())
}

/// Writes the buffer into the writer
///
/// ### Returns
///
/// A Result indicating if the writting succeded or not
pub fn stdout_write(buffer: &[u8], writer: &mut impl Write) -> Result<(), VMError> {
    writer
        .write_all(buffer)
        .map_err(|_| VMError::STDOUTWrite(String::from("Cannot write on stdout")))?;
    Ok(())
}

/// Disables the input buffering on the terminal.
/// This is done by getting  the initial termios
/// and disabling its input buffering.
pub fn setup() -> Result<Termios, VMError> {
    let stdin_fd = stdin().lock().as_raw_fd();
    let initial_termios = Termios::from_fd(stdin_fd)
        .map_err(|_| VMError::TermiosCreation(String::from("Cannot create termios")))?;
    let mut new_termios = initial_termios;
    new_termios.c_lflag &= !ICANON & !ECHO;
    tcsetattr(stdin_fd, TCSANOW, &new_termios).map_err(|_| {
        VMError::TermiosSetup(String::from("Cannot set termios with new attributes"))
    })?;
    Ok(initial_termios)
}

/// Restores the termios to the original one
pub fn shutdown(initial_termios: Termios) -> Result<(), VMError> {
    let stdin_fd = stdin().lock().as_raw_fd();
    tcsetattr(stdin_fd, TCSANOW, &initial_termios).map_err(|_| {
        VMError::TermiosSetup(String::from("Cannot set termios when shutting down"))
    })?;
    Ok(())
}

/// Load the file into the vm memory
pub fn load_arguments(args: &mut Args, mem: &mut Memory) -> Result<(), VMError> {
    if args.len() < 2 {
        println!("lc3 [image-file1] ...");
        exit(2);
    }
    // We skip the first element of the args since it is not an image
    args.next();
    for path in args {
        if read_image(path.clone(), mem).is_err() {
            println!("failed to load image: {path}");
            exit(1);
        }
    }
    Ok(())
}

/// Read bytes from file and send them to get into memory
fn read_image(path: String, mem: &mut Memory) -> Result<(), VMError> {
    let mut f = fs::read(path).map_err(|e: Error| VMError::OpenFile(e.to_string()))?;
    read_image_file(&mut f, mem)?;
    Ok(())
}

/// Gets the bytes from the file into memory
fn read_image_file(file_bytes: &mut Vec<u8>, mem: &mut Memory) -> Result<(), VMError> {
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

        mem.write(mem_addr, data)?;
        mem_addr = mem_addr.wrapping_add(1);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::hardware::Memory;

    use super::*;

    #[test]
    /// Test if the data is written in the memory, starting from
    /// the indicated address and with the data in the correct
    /// endianess
    fn read_image_file_writes_memory_correctly() {
        let mut data: Vec<u8> = vec![0xFA, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06];
        let mut mem = Memory::new();
        read_image_file(&mut data, &mut mem).unwrap();

        let origin = 0xFA00;
        assert_eq!(mem.read(origin).unwrap(), 0x0102);
        assert_eq!(mem.read(origin + 1).unwrap(), 0x0304);
        assert_eq!(mem.read(origin + 2).unwrap(), 0x0506);
    }

    #[test]
    /// Test if the data is written in the memory, starting from
    /// the indicated address and with the data in the correct
    /// endianess when opening a file
    fn read_image_reads_file_correctly_into_memory() {
        let mut mem = Memory::new();
        let _ = read_image("test_files/bytes.bin".to_string(), &mut mem);

        let origin = 0xFA00;
        assert_eq!(mem.read(origin).unwrap(), 0x0102);
        assert_eq!(mem.read(origin + 1).unwrap(), 0x0304);
        assert_eq!(mem.read(origin + 2).unwrap(), 0x0506);
    }
}
