//! rpodlib – iTunesDB parser & writer
//!
//! SPDX-License-Identifier: GPL-3.0-or-later
//! Copyright © 2025 gnome-child

#![allow(non_snake_case)]
#![allow(unused)]

use binrw::binrw;

use super::data_obj::DataObject;

#[binrw]
#[brw(little, magic = b"mhia")]
#[derive(Debug)]
pub(crate) struct AlbumItem {
    header_len: u32,
    len: u32,

    #[bw(calc = data_objects.len() as u32)]
    data_obj_count: u32,

    unk_0x10: u32, // looks like the track ids and such that start with 5
    unk_0x14: u64, // looks like an id
    unk_0x1C: u32, // always 2?
    unk_0x20: u64, // looks like another id

    #[brw(pad_before = 48)]
    #[br(count = data_obj_count)]
    data_objects: Vec<DataObject>,
}

