use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Symphonia(#[from] symphonia::core::errors::Error),

    #[error(transparent)]
    Lofty(#[from] lofty::error::LoftyError),

    #[error("No audio tracks found: {path}")]
    NoAudioTracks { path: std::path::PathBuf },

    #[error("Time base unknown: {path}")]
    TimeBaseUnknown { path: std::path::PathBuf },

    #[error("Missing frame count: {path}")]
    MissingFrameCount { path: std::path::PathBuf },

    #[error("Missing channel layout: {path}")]
    MissingChannelLayout { path: std::path::PathBuf },

    #[error("Missing sample rate: {path}")]
    MissingSampleRate { path: std::path::PathBuf },

    #[error("Missing bits per sample: {path}")]
    MissingBitsPerSample { path: std::path::PathBuf },

    #[error("Unsupported codec ({codec:?}): {path}")]
    UnsupportedCodec {
        codec: symphonia::core::codecs::CodecType,
        path: std::path::PathBuf,
    },
}
