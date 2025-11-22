#![allow(non_snake_case, unused)]

use binrw::binrw;

use crate::{
    LIST_CONTAINER_MAX_SIZE, LIST_CONTAINER_MIN_SIZE, LIST_PAYLOAD_MAX_SIZE, LIST_PAYLOAD_MIN_SIZE,
    ListType, SizeRange, album_item::AlbumItem, playlist::PlaylistItem, track_item::TrackItem,
};

#[binrw]
#[brw(magic = b"mhsd", little)]
#[derive(Debug, Clone, PartialEq)]
pub struct ListContainer {
    #[bw(calc = self.header_bytes_len())]
    header_len: u32,

    #[bw(calc = self.total_bytes_len())]
    len: u32,

    pub list_type: ListType,

    #[br(count = header_len - Self::MIN_SIZE)]
    padding: Vec<u8>,

    #[br(args { list_type: list_type } )]
    pub list: List,
}

impl SizeRange for ListContainer {
    const MIN_SIZE: u32 = LIST_CONTAINER_MIN_SIZE;
    const MAX_SIZE: u32 = LIST_CONTAINER_MAX_SIZE;

    fn header_bytes_len(&self) -> u32 {
        Self::MIN_SIZE + self.padding.len() as u32
    }

    fn total_bytes_len(&self) -> u32 {
        self.header_bytes_len() + self.list.total_bytes_len()
    }
}

impl ListContainer {
    pub fn new(list_type: ListType) -> Self {
        let list = match list_type {
            ListType::TrackItems => List::TrackItems(TrackItemList::default()),
            ListType::LibraryPlaylists => List::LibraryPlaylists(PlaylistItemList::default()),
            ListType::PodcastFmtLibPlaylists => {
                List::PodcastFmtLibPlaylists(PlaylistItemList::default())
            }
            ListType::AlbumItems => List::AlbumItems(AlbumItemList::default()),
            ListType::SpecialPlaylists => List::LibraryPlaylists(PlaylistItemList::default()),
        };

        Self {
            list_type,
            padding: vec![0u8; (Self::MAX_SIZE - Self::MIN_SIZE) as usize],
            list,
        }
    }
}

#[binrw]
#[brw(little)]
#[br (import { list_type: ListType })]
#[derive(Debug, Clone, PartialEq)]
pub enum List {
    #[br(pre_assert(list_type == ListType::TrackItems))]
    TrackItems(TrackItemList),

    #[br(pre_assert(list_type == ListType::LibraryPlaylists))]
    LibraryPlaylists(PlaylistItemList),

    #[br(pre_assert(list_type == ListType::PodcastFmtLibPlaylists))]
    PodcastFmtLibPlaylists(PlaylistItemList),

    #[br(pre_assert(list_type == ListType::AlbumItems))]
    AlbumItems(AlbumItemList),

    #[br(pre_assert(list_type == ListType::SpecialPlaylists))]
    SpecialPlaylists(PlaylistItemList),
}

impl SizeRange for List {
    const MIN_SIZE: u32 = 0;
    const MAX_SIZE: u32 = Self::MIN_SIZE;

    fn header_bytes_len(&self) -> u32 {
        match self {
            Self::TrackItems(list) => list.header_bytes_len(),
            Self::LibraryPlaylists(list) => list.header_bytes_len(),
            Self::PodcastFmtLibPlaylists(list) => list.header_bytes_len(),
            Self::AlbumItems(list) => list.header_bytes_len(),
            Self::SpecialPlaylists(list) => list.header_bytes_len(),
        }
    }

    fn total_bytes_len(&self) -> u32 {
        match self {
            Self::TrackItems(list) => list.total_bytes_len(),
            Self::LibraryPlaylists(list) => list.total_bytes_len(),
            Self::PodcastFmtLibPlaylists(list) => list.total_bytes_len(),
            Self::AlbumItems(list) => list.total_bytes_len(),
            Self::SpecialPlaylists(list) => list.total_bytes_len(),
        }
    }
}

#[binrw]
#[brw(little, magic = b"mhlt")]
#[derive(Debug, Clone, PartialEq)]
pub struct TrackItemList {
    #[bw(calc = self.header_bytes_len())]
    header_len: u32,

    #[bw(calc = items.len() as u32)]
    item_count: u32,

    #[br(count = header_len - Self::MIN_SIZE)]
    padding: Vec<u8>,

    #[br(count = item_count)]
    pub items: Vec<TrackItem>,
}

impl Default for TrackItemList {
    fn default() -> Self {
        Self {
            padding: vec![0u8; (Self::MAX_SIZE - Self::MIN_SIZE) as usize],
            items: Vec::new(),
        }
    }
}

impl SizeRange for TrackItemList {
    const MIN_SIZE: u32 = LIST_PAYLOAD_MIN_SIZE;
    const MAX_SIZE: u32 = LIST_PAYLOAD_MAX_SIZE;

    fn header_bytes_len(&self) -> u32 {
        Self::MIN_SIZE + self.padding.len() as u32
    }

    fn total_bytes_len(&self) -> u32 {
        self.header_bytes_len()
            + self
                .items
                .iter()
                .map(SizeRange::total_bytes_len)
                .sum::<u32>()
    }
}

#[binrw]
#[brw(little, magic = b"mhla")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlbumItemList {
    #[bw(calc = self.header_bytes_len())]
    header_len: u32,

    #[bw(calc = items.len() as u32)]
    item_count: u32,

    #[br(count = header_len - Self::MIN_SIZE)]
    padding: Vec<u8>,

    #[br(count = item_count)]
    pub items: Vec<AlbumItem>,
}

impl Default for AlbumItemList {
    fn default() -> Self {
        Self {
            padding: vec![0u8; (Self::MAX_SIZE - Self::MIN_SIZE) as usize],
            items: Vec::new(),
        }
    }
}

impl SizeRange for AlbumItemList {
    const MIN_SIZE: u32 = LIST_PAYLOAD_MIN_SIZE;
    const MAX_SIZE: u32 = LIST_PAYLOAD_MAX_SIZE;

    fn header_bytes_len(&self) -> u32 {
        Self::MIN_SIZE + self.padding.len() as u32
    }

    fn total_bytes_len(&self) -> u32 {
        self.header_bytes_len()
            + self
                .items
                .iter()
                .map(SizeRange::total_bytes_len)
                .sum::<u32>()
    }
}

#[binrw]
#[brw(little, magic = b"mhlp")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaylistItemList {
    #[bw(calc = self.header_bytes_len())]
    header_len: u32,

    #[bw(calc = items.len() as u32)]
    item_count: u32,

    #[br(count = header_len - Self::MIN_SIZE)]
    padding: Vec<u8>,

    #[br(count = item_count)]
    pub items: Vec<PlaylistItem>,
}

impl Default for PlaylistItemList {
    fn default() -> Self {
        Self {
            padding: vec![0u8; (Self::MAX_SIZE - Self::MIN_SIZE) as usize],
            items: Vec::new(),
        }
    }
}

impl SizeRange for PlaylistItemList {
    const MIN_SIZE: u32 = LIST_PAYLOAD_MIN_SIZE;
    const MAX_SIZE: u32 = LIST_PAYLOAD_MAX_SIZE;

    fn header_bytes_len(&self) -> u32 {
        Self::MIN_SIZE + self.padding.len() as u32
    }

    fn total_bytes_len(&self) -> u32 {
        self.header_bytes_len()
            + self
                .items
                .iter()
                .map(SizeRange::total_bytes_len)
                .sum::<u32>()
    }
}
