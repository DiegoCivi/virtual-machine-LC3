use std::fmt::Debug;

pub enum VMError {
    Arithmetic(String),
    Conversion(String),
    InvalidIndex(String),
    STDINRead(String),
    STDOUTWrite(String),
    STDOUTFlush(String),
    TermiosCreation(String),
    TermiosSetup(String),
    OpenFile(String),
    NoMoreBytes(String),
}

impl Debug for VMError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Arithmetic(arg0) => f.debug_tuple("Arithmetic").field(arg0).finish(),
            Self::Conversion(arg0) => f.debug_tuple("Conversion").field(arg0).finish(),
            Self::InvalidIndex(arg0) => f.debug_tuple("InvalidIndex").field(arg0).finish(),
            Self::STDINRead(arg0) => f.debug_tuple("STDINRead").field(arg0).finish(),
            Self::STDOUTWrite(arg0) => f.debug_tuple("STDOUTWrite").field(arg0).finish(),
            Self::STDOUTFlush(arg0) => f.debug_tuple("STDOUTFlush").field(arg0).finish(),
            Self::TermiosCreation(arg0) => f.debug_tuple("TermiosCreation").field(arg0).finish(),
            Self::TermiosSetup(arg0) => f.debug_tuple("TermiosSetup").field(arg0).finish(),
            Self::OpenFile(arg0) => f.debug_tuple("OpenFile").field(arg0).finish(),
            Self::NoMoreBytes(arg0) => f.debug_tuple("NoMoreBytes").field(arg0).finish(),
        }
    }
}
