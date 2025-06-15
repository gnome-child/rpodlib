//! rpodlib – iTunesDB parser & writer
//!
//! SPDX-License-Identifier: GPL-3.0-or-later
//! Copyright © 2025 gnome-child

#![allow(non_snake_case)]
#![allow(unused)]

use binrw::binrw;
use bytemuck::try_cast_slice;

use super::image::{ImageInfo, MhafItem};

#[binrw]
#[br(import { id: u32, bytes_left: u32 })]
#[derive(Debug)]
pub(crate) enum ObjectType {
    #[br(pre_assert(id ==  2))]
    ImageMeta(#[br(args { bytes_left })] ImageMetaObj),
    #[br(pre_assert(id ==  3))]
    IthmbFileName(#[br(args { bytes_left })] Utf16IthmbName),
    #[br(pre_assert(id ==  6))]
    MhafHolder(#[br(args { bytes_left })] MhafObj),
}
impl ObjectType {
    pub fn as_id(&self) -> u32 {
        match self {
            ObjectType::ImageMeta(_) => 2,
            ObjectType::IthmbFileName(_) => 3,
            ObjectType::MhafHolder(_) => 6,
        }
    }
}

#[binrw]
#[brw(little, magic = b"mhod")]
#[derive(Debug)]
pub(crate) struct DataObject {
    header_len: u32,
    len: u32,

    #[bw(calc = object.as_id())]
    obj_type: u32,

    #[br(args { id: obj_type, bytes_left: len - 16 })]
    pub object: ObjectType,
}

#[binrw]
#[br(import { bytes_left: u32 = 0 })]
#[derive(Debug)]
pub(crate) struct ImageMetaObj {
    #[brw(pad_before = 8)]
    image_info: ImageInfo,
}

#[binrw]
#[br(import { bytes_left: u32 = 0 })]
#[derive(Debug)]
pub(crate) struct Utf16IthmbName {
    unk_0x10: u32,
    unk_0x14: u32,

    #[bw(calc = string_data.len() as u32)]
    length: u32,

    unk_0x1C: u32,
    unk_0x20: u32,

    #[br(count = length)]
    string_data: Vec<u8>,
}

impl Utf16IthmbName {
    pub fn to_string(&self) -> Result<String, std::string::FromUtf16Error> {
        assert!(self.string_data.len() % 2 == 0, "Malformed UTF-16");

        let words: &[u16] =
            try_cast_slice(&self.string_data).expect("Failed to cast byte array to u16 array");

        String::from_utf16(&words)
    }
}

#[binrw]
#[br(import { bytes_left: u32 = 0 })]
#[derive(Debug)]
pub(crate) struct MhafObj {
    unk_0x00: u32,
    unk_0x04: u32,
    pub mhaf: MhafItem,
}

#[binrw]
#[br(import { bytes_left: u32 = 0 })]
#[derive(Debug)]
pub(crate) struct Unimplemented {
    #[br(count = bytes_left)]
    data: Vec<u8>,
}
