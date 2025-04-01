#[derive(Debug)]
pub enum VMError {
    Arithmetic,
    Conversion,
    InvalidIndex,
    STDINRead,
    STDOUTWrite,
    STDOUTFlush,
    ShellMisuse,
    OpenFile,
    ReadFile,
    SetHandler,
    TermiosCreation,
    TermiosAttrSet,
}
