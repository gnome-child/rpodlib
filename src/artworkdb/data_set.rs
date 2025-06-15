//! rpodlib – iTunesDB parser & writer
//!
//! SPDX-License-Identifier: GPL-3.0-or-later
//! Copyright © 2025 gnome-child

#![allow(non_snake_case)]
#![allow(unused)]

use binrw::binrw;

use super::image::ImageItem;

#[binrw]
#[br(import { id: u32 })]
#[derive(Debug)]
pub enum SetType {
    #[br(pre_assert(id == 0x01))] // ← pick this variant when id == 1
    Images(ImageList),

    #[br(pre_assert(id == 0x02))]
    Albums(AlbumList),

    #[br(pre_assert(id == 0x03))]
    Files(FileList),
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
            SetType::Images(_) => 0x01,
            SetType::Albums(_) => 0x02,
            SetType::Files(_) => 0x03,
        }
    }
}

#[binrw]
#[brw(little, magic = b"mhli")]
#[derive(Debug)]
pub(crate) struct ImageList {
    pub header_len: u32,

    #[bw(calc = entries.len() as u32)]
    pub entry_count: u32,

    #[brw(pad_before = 80)]
    #[br(count = entry_count)]
    pub entries: Vec<ImageItem>,
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
    pub entries: Vec<u8>,
}

#[binrw]
#[brw(little, magic = b"mhlp")]
#[derive(Debug)]
pub(crate) struct FileList {
    pub header_len: u32,

    #[bw(calc = entries.len() as u32)]
    pub entry_count: u32,

    #[brw(pad_before = 80)]
    #[br(count = entry_count)]
    pub entries: Vec<u8>,
}
