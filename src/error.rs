use std::fmt::Debug;

pub enum VMError {
    Arithmetic { minuend: usize, subtrahend: usize },
    Conversion(String),
    InvalidIndex(usize),
    STDINRead(String),
    STDOUTWrite(String),
    STDOUTFlush(String),
    TermiosCreation(String),
    TermiosSetup(String),
    OpenFile(String, String),
    NoMoreBytes(String),
}

impl Debug for VMError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Arithmetic {
                minuend,
                subtrahend,
            } => {
                write!(
                    f,
                    "Arithmetic: cannot subtract {} from {}",
                    subtrahend, minuend
                )
            }
            Self::Conversion(arg0) => f.debug_tuple("Conversion").field(arg0).finish(),
            Self::InvalidIndex(index) => write!(f, "InvalidIndex: index [{}] is invalid", index),
            Self::STDINRead(arg0) => f.debug_tuple("STDINRead").field(arg0).finish(),
            Self::STDOUTWrite(arg0) => f.debug_tuple("STDOUTWrite").field(arg0).finish(),
            Self::STDOUTFlush(arg0) => f.debug_tuple("STDOUTFlush").field(arg0).finish(),
            Self::TermiosCreation(arg0) => f.debug_tuple("TermiosCreation").field(arg0).finish(),
            Self::TermiosSetup(arg0) => f.debug_tuple("TermiosSetup").field(arg0).finish(),
            Self::OpenFile(path, error) => write!(
                f,
                "OpenFile: cannot open file [{}] because of error [{}]",
                path, error
            ),
            Self::NoMoreBytes(arg0) => f.debug_tuple("NoMoreBytes").field(arg0).finish(),
        }
    }
}
