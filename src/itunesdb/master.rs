//! rpodlib – iTunesDB parser & writer
//!
//! SPDX-License-Identifier: GPL-3.0-or-later
//! Copyright © 2025 gnome-child

#![allow(non_snake_case)]
#![allow(unused)]

use binrw::binrw;

use super::dataset::DataSet;

#[binrw]
#[brw(little, magic = b"mhbd")]
#[derive(Debug)]
pub(crate) struct Master {
    pub header_len: u32,
    pub len: u32,
    pub unk_0x0C: u32,
    pub version: u32,

    #[bw(calc = data_sets.len() as u32)]
    pub data_set_count: u32,

    pub database_id: u64,
    pub unk_0x20: u16,
    pub hashing_scheme: u16,
    pub unk_0x24: u64,
    pub unk_0x2C: u32,
    pub unk_0x30: u16,
    pub padding_0x32: [u8; 20],
    pub lang: u16,
    pub persistent_id: u64,
    pub unk_0x50: u32,
    pub unk_0x54: u32,
    pub hash_0x58: [u8; 20],
    pub timezone_offset: i32,
    pub unk_0x70: u16,
    pub hash_0x72: [u8; 46],
    pub unk_0xA0: u32,
    pub audio_lang: u16,
    pub subtitle_lang: u16,

    #[brw(pad_before = 76)]
    #[br(count = data_set_count)]
    pub data_sets: Vec<DataSet>,
}

