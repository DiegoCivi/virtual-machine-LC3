#[derive(Debug)]
pub enum VMError {
    Arithmetic,
    Conversion,
    InvalidIndex,
    STDINRead
}
