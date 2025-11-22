use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Symphonia(#[from] symphonia::core::errors::Error),

    #[error("Child process failed with {code:?}: {message}")]
    ProcessFailed { code: Option<i32>, message: String },

    #[error("Encoder overflowed! Wrote {bytes} bytes")]
    EncoderOverflow { bytes: u64 },

    #[error("Failed to copy metadata to output file: {path}")]
    MetadataCopy { path: std::path::PathBuf },

    #[error("Error: {0}")]
    Generic(&'static str),
}
