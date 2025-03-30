use crate::{error::VMError, hardware::{Memory, MemoryRegister}};


pub fn mem_read(address: u16, memory: &mut Memory) -> Result<&u16, VMError> {
    if address == MemoryRegister::KeyboardStatus {
        if check_key() {
            memory.set(MemoryRegister::KeyboardStatus, 1 << 15);
            memory.set(MemoryRegister::KeyboardData, getchar());

            // memory[MemoryRegister::KeyboardStatus] = 1 << 15;
            // memory[MemoryRegister::KeyboardData] = getchar();
        } else {
            // memory[MemoryRegister::KeyboardStatus] = 0;
            memory.set(MemoryRegister::KeyboardStatus, 0);

        }
    }
    memory.get(address)
}
