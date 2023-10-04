#[derive(Debug)]
pub struct EmptyChunkError {
    pub message: String,
}

impl std::fmt::Display for EmptyChunkError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "EmptyChunkError: {}", self.message)
    }
}

impl std::error::Error for EmptyChunkError {}