#[derive(Debug)]
pub enum AllocError {
    OutOfMemory,
    NullPointer,
    InvalidPointer,
    InvalidChunk,
    CorruptedChunk,
    DoubleFree,
}