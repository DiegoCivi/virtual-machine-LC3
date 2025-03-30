use crate::{error::VMError, hardware::{Memory, MemoryRegister}, utils::{check_key, getchar}};

/// Reads a memory address. Also checks whether a key was pressed and it
/// that case it stores which key was pressed.
/// 
/// ### Arguments
/// 
/// - `address`: An u16 representing the memory address to read.
/// - `Memory`: A Mmeory struct that handles the memory in the system
pub fn mem_read(address: u16, memory: &mut Memory) -> Result<u16, VMError> {
    if address == MemoryRegister::KeyboardStatus {
        if check_key() {
            memory.set(MemoryRegister::KeyboardStatus, 1 << 15)?;
            let buffer = getchar()?;
            let char: u16 = buffer[0].into();
            memory.set(MemoryRegister::KeyboardData, char)?;
        } else {
            memory.set(MemoryRegister::KeyboardStatus, 0)?;
        }
    }
    memory.get(address).copied()
}
