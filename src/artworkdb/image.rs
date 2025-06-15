//! rpodlib – iTunesDB parser & writer
//!
//! SPDX-License-Identifier: GPL-3.0-or-later
//! Copyright © 2025 gnome-child

#![allow(non_snake_case)]
#![allow(unused)]

use binrw::binrw;

use super::data_obj::DataObject;

#[binrw]
#[brw(little, magic = b"mhii")]
#[derive(Debug)]
pub(crate) struct ImageItem {
    pub header_len: u32,
    pub len: u32,

    #[bw(calc = data_objects.len() as u32)]
    pub data_obj_count: u32,

    pub id: u32,
    pub track_persistent_id: u64,
    unk_0x1C: u32,
    pub rating: u32,
    unk_0x24: u32,
    pub image_creation_hfs: u32,
    pub image_exif_creation_hfs: u32,
    pub image_size: u32,
    unk_0x34: u32,
    unk_0x38: u32,
    unk_0x3C: u32,
    unk_0x40: u32,
    unk_0x44: u32,

    #[bw(calc = self.image_size)]
    image_size_dup: u32,

    #[brw(pad_before = 76)]
    #[br(count = data_obj_count)]
    pub data_objects: Vec<DataObject>,
}

