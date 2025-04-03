#[derive(Debug)]
pub enum VMError {
    Arithmetic,
    Conversion,
    InvalidIndex,
    STDINRead,
    STDOUTWrite,
    STDOUTFlush,
    TermiosCreation,
    TermiosSetup,
    OpenFile,
    NoMoreBytes,
}
