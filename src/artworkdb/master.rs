//! rpodlib – iTunesDB parser & writer
//!
//! SPDX-License-Identifier: GPL-3.0-or-later
//! Copyright © 2025 gnome-child

#![allow(non_snake_case)]
#![allow(unused)]

use binrw::binrw;

use super::data_set::DataSet;

#[binrw]
#[brw(little, magic = b"mhfd")]
#[derive(Debug)]
pub(crate) struct Master {
    pub header_len: u32,
    pub len: u32,
    unk_0x0C: u32,
    unk_0x10: u32,

    #[bw(calc = data_sets.len() as u32)]
    pub data_set_count: u32,

    unk_0x18: u32,
    pub next_mhii_id: u32,
    unk_0x20: u64,
    unk_0x28: u64,
    unk_0x30: u32,
    unk_0x34: u32,
    unk_0x38: u32,
    unk_0x3C: u32,
    unk_0x40: u32,

    #[brw(pad_before = 64)]
    #[br(count = data_set_count)]
    pub data_sets: Vec<DataSet>,
}

