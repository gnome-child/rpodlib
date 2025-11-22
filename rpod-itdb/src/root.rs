#![allow(non_snake_case, unused)]

use std::{collections::HashMap, io::Cursor};

use binrw::{BinRead, BinWrite, binrw};
use rand::random;

use crate::{
    DataObjectContainer, DataType, ROOT_MAX_SIZE, ROOT_MIN_SIZE, SizeRange,
    album_item::AlbumItem,
    error::{Error, Result},
    list::{AlbumItemList, List, ListContainer, PlaylistItemList, TrackItemList},
    playlist::PlaylistItem,
};

#[binrw]
#[brw(magic = b"mhbd", little)]
#[derive(Debug, Clone, PartialEq)]
pub struct Root {
    #[bw(calc = self.header_bytes_len())]
    header_len: u32,

    #[bw(calc = self.total_bytes_len())]
    len: u32,

    pub unk_0x0C: u32,
    pub version: u32,

    #[bw(calc = list_containers.len() as u32)]
    list_container_count: u32,

    pub generation_uid: u64,
    pub unk_0x20: u16, // often 2
    pub unk_0x22: u16,
    pub database_uid: u64,
    pub unk_0x2C: u32,
    pub unk_0x30: u16,
    pub unk_0x32: u16,
    pub unk_0x34: u32,
    pub unk_0x38: u32,
    pub unk_0x3C: u32,
    pub unk_0x40: u32,
    pub unk_0x44: u16,
    pub lang: [u8; 2],
    pub persistent_id: u64, // seen in iTunes
    pub unk_0x50: u32,
    pub unk_0x54: u32,
    pub hash_0x58: [u8; 20],
    pub timezone_offset: i32,
    pub unk_0x70: u16,
    pub hash_0x72: [u8; 46],
    pub unk_0xA0: i32, // often -1
    pub audio_lang: [u8; 2],
    pub subtitle_lang: [u8; 2],

    #[br(count = header_len - Self::MIN_SIZE)]
    padding: Vec<u8>,

    #[br(count = list_container_count)]
    pub list_containers: Vec<ListContainer>,
}

impl Default for Root {
    fn default() -> Self {
        Self {
            unk_0x0C: 0,
            version: 0,
            generation_uid: 0,
            unk_0x20: 2,
            unk_0x22: 0,
            database_uid: 0,
            unk_0x2C: 0,
            unk_0x30: 0,
            unk_0x32: 0,
            unk_0x34: 0,
            unk_0x38: 0,
            unk_0x3C: 0,
            unk_0x40: 0,
            unk_0x44: 0,
            lang: [0; 2],
            persistent_id: 0,
            unk_0x50: 0,
            unk_0x54: 0,
            hash_0x58: [0; 20],
            timezone_offset: 0,
            unk_0x70: 0,
            hash_0x72: [0; 46],
            unk_0xA0: -1,
            audio_lang: [0; 2],
            subtitle_lang: [0; 2],
            padding: vec![0u8; (Self::MAX_SIZE - Self::MIN_SIZE) as usize],
            list_containers: Vec::new(),
        }
    }
}

impl SizeRange for Root {
    const MIN_SIZE: u32 = ROOT_MIN_SIZE;
    const MAX_SIZE: u32 = ROOT_MAX_SIZE;

    fn header_bytes_len(&self) -> u32 {
        Self::MIN_SIZE + self.padding.len() as u32
    }

    fn total_bytes_len(&self) -> u32 {
        self.header_bytes_len()
            + self
                .list_containers
                .iter()
                .map(SizeRange::total_bytes_len)
                .sum::<u32>()
    }
}

impl Root {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(bytes);

        Ok(Root::read(&mut cursor)?)
    }

    pub fn to_bytes(&mut self, buf: &mut [u8]) -> Result<()> {
        let mut cursor = Cursor::new(buf);

        self.write(&mut cursor)?;
        Ok(())
    }

    pub fn track_item_list(&self) -> Option<&TrackItemList> {
        self.list_containers
            .iter()
            .find_map(|container| match &container.list {
                List::TrackItems(list) => Some(list),
                _ => None,
            })
    }

    pub fn album_item_list(&self) -> Option<&AlbumItemList> {
        self.list_containers
            .iter()
            .find_map(|container| match &container.list {
                List::AlbumItems(list) => Some(list),
                _ => None,
            })
    }

    pub fn playlist_item_list(&self) -> Option<&PlaylistItemList> {
        self.list_containers
            .iter()
            .find_map(|container| match &container.list {
                List::LibraryPlaylists(list) => Some(list),
                _ => None,
            })
    }

    pub fn podcast_fmt_playlist_list(&self) -> Option<&PlaylistItemList> {
        self.list_containers
            .iter()
            .find_map(|container| match &container.list {
                List::PodcastFmtLibPlaylists(list) => Some(list),
                _ => None,
            })
    }

    pub fn special_playlist_list(&self) -> Option<&PlaylistItemList> {
        self.list_containers
            .iter()
            .find_map(|container| match &container.list {
                List::SpecialPlaylists(list) => Some(list),
                _ => None,
            })
    }

    pub fn track_item_list_mut(&mut self) -> Option<&mut TrackItemList> {
        self.list_containers
            .iter_mut()
            .find_map(|container| match &mut container.list {
                List::TrackItems(list) => Some(list),
                _ => None,
            })
    }

    pub fn album_item_list_mut(&mut self) -> Option<&mut AlbumItemList> {
        self.list_containers
            .iter_mut()
            .find_map(|container| match &mut container.list {
                List::AlbumItems(list) => Some(list),
                _ => None,
            })
    }

    pub fn playlist_item_list_mut(&mut self) -> Option<&mut PlaylistItemList> {
        self.list_containers
            .iter_mut()
            .find_map(|container| match &mut container.list {
                List::LibraryPlaylists(list) => Some(list),
                _ => None,
            })
    }

    pub fn podcast_fmt_playlist_list_mut(&mut self) -> Option<&mut PlaylistItemList> {
        self.list_containers
            .iter_mut()
            .find_map(|container| match &mut container.list {
                List::PodcastFmtLibPlaylists(list) => Some(list),
                _ => None,
            })
    }

    pub fn special_playlist_list_mut(&mut self) -> Option<&mut PlaylistItemList> {
        self.list_containers
            .iter_mut()
            .find_map(|container| match &mut container.list {
                List::SpecialPlaylists(list) => Some(list),
                _ => None,
            })
    }

    pub fn master_playlist(&self) -> Option<&PlaylistItem> {
        self.playlist_item_list()?
            .items
            .iter()
            .find(|playlist| playlist.is_master_flag != 0)
    }

    pub fn master_playlist_mut(&mut self) -> Option<&mut PlaylistItem> {
        self.playlist_item_list_mut()?
            .items
            .iter_mut()
            .find(|playlist| playlist.is_master_flag != 0)
    }
}
