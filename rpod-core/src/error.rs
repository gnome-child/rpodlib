use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Device(#[from] rpod_device::error::Error),

    #[error(transparent)]
    Metadata(#[from] rpod_meta::error::Error),

    #[error(transparent)]
    Database(#[from] rpod_itdb::error::Error),

    #[error(transparent)]
    Transcode(#[from] rpod_transcode::error::Error),
}
