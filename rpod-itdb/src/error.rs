use std::ops::Range;

use thiserror::Error;

use crate::DataType;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Missing master playlist!")]
    MasterPlaylistMissing,

    #[error("Track item list missing!")]
    TrackItemListMissing,

    #[error("Playlist item list missing!")]
    PlaylistItemListMissing,

    #[error("Podcast format playlist list missing!")]
    PodcastItemListMissing,

    #[error("Podcast playlist missing!")]
    PodcastPlaylistMissing,

    #[error("Album item list missing!")]
    AlbumItemListMissing,

    #[error("Found an orphaned playlist entry referencing uid {uid}")]
    OrphanedPlaylistEntry { uid: u64 },

    #[error("Can't make an album item without an album title!")]
    MissingAlbumTitle,

    #[error("Data type {data_type:?} is not a string type")]
    NotAString { data_type: DataType },

    #[error(transparent)]
    RandOs(#[from] rand::rand_core::OsError),

    #[error("Buffer too short, {range:?} unreachable")]
    OutOfBounds { range: Range<usize> },

    #[error("Bad hash72 signature: {sig:04X?}")]
    BadSignature { sig: [u8; 2] },

    #[error(
        "CBC check failed at byte {i}:\ndec(C1)[{i}] ^ C0[{i}] = {calc:02X}\nexpected P1[{i}] = {expect:02X}\nsha1_tail={sha1_tail:02X?}\nrnd12={rand12:02X?}"
    )]
    CbcCheckFailure {
        i: usize,
        calc: u8,
        expect: u8,
        sha1_tail: [u8; 4],
        rand12: [u8; 12],
    },

    #[error(transparent)]
    Binrw(#[from] binrw::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
