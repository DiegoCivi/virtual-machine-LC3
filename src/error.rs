#[derive(Debug)]
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
