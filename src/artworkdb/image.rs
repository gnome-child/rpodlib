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

#[binrw]
#[brw(little, magic = b"mhni")]
#[derive(Debug)]
pub(crate) struct ImageInfo {
    pub header_len: u32,
    pub len: u32,

    #[bw(calc = data_objects.len() as u32)]
    pub child_count: u32,

    pub corr_id: u32,
    pub ithmb_offset: u32,
    pub image_size: u32,
    pub v_pad: i16,
    pub h_pad: i16,
    pub img_height: u16,
    pub img_width: u16,
    unk_0x24: u32,

    #[bw(calc = self.image_size)]
    pub image_size_dup: u32,

    #[brw(pad_before = 32)]
    #[br(count = child_count)]
    pub data_objects: Vec<DataObject>,
}

#[binrw]
#[brw(little, magic = b"mhif")]
#[derive(Debug)]
pub(crate) struct ImageFile {
    pub header_len: u32,
    pub len: u32,
    unk_0x0C: u32,
    pub correlation_id: u32,

    #[brw(pad_after = 100)]
    pub image_size: u32,
}

#[binrw]
#[brw(little, magic = b"mhaf")]
#[derive(Debug)]
pub(crate) struct MhafItem {
    pub len: u32,

    #[brw(pad_after = 84)]
    unk_0x04: u32, // always 0x3C?? (60)
}
