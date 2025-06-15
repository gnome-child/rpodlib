#![allow(non_snake_case)]
#![allow(unused)]

use binrw::binrw;

use super::{album::AlbumItem, playlist::Playlist, track::TrackItem};

#[binrw]
#[br(import { id: u32 })]
#[derive(Debug)]
pub enum SetType {
    #[br(pre_assert(id == 0x01))] // ← pick this variant when id == 1
    Tracks(TrackList),

    #[br(pre_assert(id == 0x02))]
    Playlists(PlaylistList),

    #[br(pre_assert(id == 0x03))]
    Podcasts(PlaylistList),

    #[br(pre_assert(id == 0x04))]
    Albums(AlbumList),

    #[br(pre_assert(id == 0x05))]
    InclSmartPlaylists(PlaylistList),
}

#[binrw]
#[brw(little, magic = b"mhsd")]
#[derive(Debug)]
pub(crate) struct DataSet {
    pub header_len: u32,
    pub len: u32,

    #[bw(calc = set.as_id())]
    pub set_type: u32,

    #[br(args { id: set_type })]
    #[brw(pad_before = 80)]
    pub set: SetType,
}

impl SetType {
    pub fn as_id(&self) -> u32 {
        match self {
            SetType::Tracks(_) => 0x01,
            SetType::Playlists(_) => 0x02,
            SetType::Podcasts(_) => 0x03,
            SetType::Albums(_) => 0x04,
            SetType::InclSmartPlaylists(_) => 0x05,
        }
    }
}

#[binrw]
#[brw(little, magic = b"mhlt")]
#[derive(Debug)]
pub(crate) struct TrackList {
    pub header_len: u32,

    #[bw(calc = entries.len() as u32)]
    pub entry_count: u32,

    #[brw(pad_before = 80)]
    #[br(count = entry_count)]
    pub entries: Vec<TrackItem>,
}

#[binrw]
#[brw(little, magic = b"mhla")]
#[derive(Debug)]
pub(crate) struct AlbumList {
    pub header_len: u32,

    #[bw(calc = entries.len() as u32)]
    pub entry_count: u32,

    #[brw(pad_before = 80)]
    #[br(count = entry_count)]
    pub entries: Vec<AlbumItem>,
}

#[binrw]
#[brw(little, magic = b"mhlp")]
#[derive(Debug)]
pub(crate) struct PlaylistList {
    pub header_len: u32,

    #[bw(calc = entries.len() as u32)]
    pub entry_count: u32,

    #[brw(pad_before = 80)]
    #[br(count = entry_count)]
    pub entries: Vec<Playlist>,
}
