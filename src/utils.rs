use crate::hardware::{CondFlag, Register, Registers};

pub fn sign_extend(mut x: u16, bit_count: usize) -> u16 {
    // Get MSB and check if it is a 1
    let msb = x >> (bit_count - 1) & 1; // TODO: Is the '& 1' necessary?
    if msb != 0 {
        // If the MSB is 1 it means it is negative, else its positive
        x |= 0xFFFF << bit_count;
    }
    x
}

pub fn update_flags(r: Register, regs: &mut Registers) {
    if regs[r] == 0 {
        regs[Register::COND] = CondFlag::ZRO as u16;
    } else if regs[r] >> 15 == 1 {
        regs[Register::COND] = CondFlag::NEG as u16;
    } else {
        regs[Register::COND] = CondFlag::POS as u16;
    }
}
