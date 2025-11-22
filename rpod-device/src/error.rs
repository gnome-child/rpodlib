use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("iTunesLock exists, try deleting it")]
    LockFileExists,

    #[error("Couldn't find a device profile")]
    ProfileNotFound,

    #[error("Query was ambiguous; matches {matches} device profiles")]
    AmbiguousProfile { matches: usize },

    #[error("The serial number was not found")]
    MissingSerial,

    #[error("The FireWire GUID was not found")]
    MissingFireWireGUID,

    #[error("Invalid serial number: {serial_num}")]
    InvalidSerialNum { serial_num: String },

    #[error(transparent)]
    Plist(#[from] plist::Error),

    #[error(transparent)]
    Binrw(#[from] binrw::Error),

    #[error(transparent)]
    Platform(#[from] crate::platform::Error),

    #[error(transparent)]
    Database(#[from] rpod_itdb::error::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
