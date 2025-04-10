use crate::error::VMError;

pub enum TrapCode {
    GetC,
    Out,
    Puts,
    In,
    PutsP,
    Halt,
}

impl TryFrom<u16> for TrapCode {
    type Error = VMError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0x20 => Ok(TrapCode::GetC),
            0x21 => Ok(TrapCode::Out),
            0x22 => Ok(TrapCode::Puts),
            0x23 => Ok(TrapCode::In),
            0x24 => Ok(TrapCode::PutsP),
            0x25 => Ok(TrapCode::Halt),
            _ => Err(VMError::Conversion(String::from(
                "Invalid u16 for TrapCode conversion",
            ))),
        }
    }
}
