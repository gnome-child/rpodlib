#![allow(non_snake_case, unused)]

use std::borrow::Cow;

use binrw::binrw;

use crate::{
    DATA_OBJECT_MAX_SIZE, DATA_OBJECT_MIN_SIZE, DataType, LIB_PLAYLIST_INDEX_MIN_SIZE,
    LibraryIndexType, SizeRange, UTF_16_STR_MIN_SIZE,
};

const TYPE_100_BLOB: [u8; 624] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x01, 0x00, 0x05, 0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00,
    0x01, 0x00, 0x38, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x3E, 0x00, 0x13, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x02, 0x00, 0xFA, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x0D, 0x00, 0x4A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x04, 0x00, 0x9C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x03, 0x00, 0x9C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x08, 0x00, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x47, 0x00, 0x22, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x14, 0x00, 0x38, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xAF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

#[binrw]
#[brw(little, magic = b"mhod")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataObject {
    #[bw(calc = self.header_bytes_len())]
    header_len: u32,

    #[bw(calc = self.total_bytes_len())]
    len: u32,

    pub data_type: DataType,

    #[br(count = header_len - Self::MIN_SIZE)]
    padding: Vec<u8>,

    #[br(args { data_type: data_type, payload_len: len - header_len })]
    pub payload: Payload,
}

impl SizeRange for DataObject {
    const MIN_SIZE: u32 = DATA_OBJECT_MIN_SIZE;
    const MAX_SIZE: u32 = DATA_OBJECT_MAX_SIZE;

    fn header_bytes_len(&self) -> u32 {
        Self::MIN_SIZE + self.padding.len() as u32
    }

    fn total_bytes_len(&self) -> u32 {
        self.header_bytes_len() + self.payload.total_bytes_len() as u32
    }
}

impl DataObject {
    pub fn new_utf16(data_type: DataType, s: impl AsRef<str>) -> Self {
        let padding = vec![0u8; (Self::MAX_SIZE - Self::MIN_SIZE) as usize];
        let payload = Payload::Utf16String(Utf16String {
            chars: s.as_ref().encode_utf16().collect(),
            ..Utf16String::default()
        });

        Self {
            data_type,
            padding,
            payload,
        }
    }

    pub fn new_utf8(data_type: DataType, s: impl AsRef<str>) -> Self {
        let padding = vec![0u8; (Self::MAX_SIZE - Self::MIN_SIZE) as usize];
        let payload = Payload::Utf8Chars(Utf8Chars {
            chars: s.as_ref().as_bytes().to_vec(),
        });

        Self {
            data_type,
            padding,
            payload,
        }
    }

    pub fn new_playlist_pos(position: u32) -> Self {
        let data_type = DataType::Type100;
        let padding = vec![0u8; (Self::MAX_SIZE - Self::MIN_SIZE) as usize];
        let payload = Payload::PlaylistPosition(PlaylistPosition { position });

        Self {
            data_type,
            padding,
            payload,
        }
    }

    pub fn new_lib_playlist_index(index_type: LibraryIndexType, indices: Vec<u32>) -> Self {
        let data_type = DataType::LibraryPlaylistIndex;
        let padding = vec![0u8; (Self::MAX_SIZE - Self::MIN_SIZE) as usize];
        let payload = Payload::LibraryPlaylistIndex(LibraryPlaylistIndex {
            index_type,
            indices,
        });

        Self {
            data_type,
            padding,
            payload,
        }
    }

    pub fn new_jump_table(index_type: LibraryIndexType, entries: Vec<JumpTableEntry>) -> Self {
        let data_type = DataType::JumpTable;
        let padding = vec![0u8; (Self::MAX_SIZE - Self::MIN_SIZE) as usize];
        let payload = Payload::JumpTable(JumpTable {
            index_type,
            entries,
        });

        Self {
            data_type,
            padding,
            payload,
        }
    }

    pub fn new_type_100_blob() -> Self {
        let data_type = DataType::Type100;
        let padding = vec![0u8; (Self::MAX_SIZE - Self::MIN_SIZE) as usize];
        let payload = Payload::Raw(Raw {
            bytes: TYPE_100_BLOB.to_vec(),
        });

        Self {
            data_type,
            padding,
            payload,
        }
    }
}

#[binrw]
#[brw(little)]
#[br(import { data_type: DataType, payload_len: u32 })]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Payload {
    #[br(pre_assert(data_type.is_utf16()))]
    Utf16String(Utf16String),

    #[br(pre_assert(data_type.is_utf8()))]
    Utf8Chars(#[br(args { payload_len })] Utf8Chars),

    #[br(pre_assert(data_type.is_playlist_index()))]
    LibraryPlaylistIndex(LibraryPlaylistIndex),

    #[br(pre_assert(data_type.is_jump_table()))]
    JumpTable(JumpTable),

    #[br(pre_assert(data_type.is_type_100() && payload_len == 20))]
    PlaylistPosition(PlaylistPosition),

    #[br(pre_assert(data_type.is_type_100() && payload_len != 20 || data_type.is_unimplemented()))]
    Raw(#[br(args { payload_len })] Raw),
}

impl SizeRange for Payload {
    const MIN_SIZE: u32 = 0;
    const MAX_SIZE: u32 = Self::MIN_SIZE;

    fn header_bytes_len(&self) -> u32 {
        match self {
            Payload::Utf16String(payload) => payload.header_bytes_len(),
            Payload::Utf8Chars(payload) => payload.header_bytes_len(),
            Payload::LibraryPlaylistIndex(payload) => payload.header_bytes_len(),
            Payload::JumpTable(payload) => payload.header_bytes_len(),
            Payload::PlaylistPosition(payload) => payload.header_bytes_len(),
            Payload::Raw(payload) => payload.header_bytes_len(),
        }
    }

    fn total_bytes_len(&self) -> u32 {
        match self {
            Payload::Utf16String(payload) => payload.total_bytes_len(),
            Payload::Utf8Chars(payload) => payload.total_bytes_len(),
            Payload::LibraryPlaylistIndex(payload) => payload.total_bytes_len(),
            Payload::JumpTable(payload) => payload.total_bytes_len(),
            Payload::PlaylistPosition(payload) => payload.total_bytes_len(),
            Payload::Raw(payload) => payload.total_bytes_len(),
        }
    }
}

impl Payload {
    pub fn as_str_lossy(&self) -> Option<Cow<'_, str>> {
        match self {
            Payload::Utf16String(payload) => {
                Some(Cow::Owned(String::from_utf16_lossy(&payload.chars)))
            }
            Payload::Utf8Chars(payload) => Some(String::from_utf8_lossy(&payload.chars)),
            _ => None,
        }
    }

    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Payload::Raw(raw) => Some(&raw.bytes),
            _ => None,
        }
    }

    pub fn as_playlist_pos(&self) -> Option<u32> {
        match self {
            Payload::PlaylistPosition(pos) => Some(pos.position),
            _ => None,
        }
    }
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Utf16String {
    pub position: u32,

    #[bw(calc = chars.len() as u32 * 2)]
    len: u32,

    pub unk_0x08: u32,
    pub unk_0x0C: u32,

    #[br(count = len / 2)]
    pub chars: Vec<u16>,
}

impl Default for Utf16String {
    fn default() -> Self {
        Self {
            position: 1,
            unk_0x08: 1,
            unk_0x0C: 0,
            chars: Vec::new(),
        }
    }
}

impl SizeRange for Utf16String {
    const MIN_SIZE: u32 = UTF_16_STR_MIN_SIZE;
    const MAX_SIZE: u32 = Self::MIN_SIZE;

    fn header_bytes_len(&self) -> u32 {
        Self::MIN_SIZE
    }

    fn total_bytes_len(&self) -> u32 {
        self.header_bytes_len() + (self.chars.len() as u32 * 2)
    }
}

#[binrw]
#[brw(little)]
#[br(import { payload_len: u32 })]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Utf8Chars {
    #[br(count = payload_len)]
    pub chars: Vec<u8>,
}

impl Default for Utf8Chars {
    fn default() -> Self {
        Self { chars: Vec::new() }
    }
}

impl SizeRange for Utf8Chars {
    const MIN_SIZE: u32 = 0;
    const MAX_SIZE: u32 = Self::MIN_SIZE;

    fn header_bytes_len(&self) -> u32 {
        Self::MIN_SIZE
    }

    fn total_bytes_len(&self) -> u32 {
        self.chars.len() as u32
    }
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryPlaylistIndex {
    pub index_type: LibraryIndexType,

    #[bw(calc = indices.len() as u32)]
    count: u32,

    #[brw(pad_before = 0x28)]
    #[br(count = count)]
    pub indices: Vec<u32>,
}

impl SizeRange for LibraryPlaylistIndex {
    const MIN_SIZE: u32 = LIB_PLAYLIST_INDEX_MIN_SIZE;
    const MAX_SIZE: u32 = Self::MIN_SIZE;

    fn header_bytes_len(&self) -> u32 {
        Self::MIN_SIZE
    }

    fn total_bytes_len(&self) -> u32 {
        self.header_bytes_len() + (self.indices.len() as u32) * 4
    }
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JumpTable {
    pub index_type: LibraryIndexType,

    #[bw(calc = self.entries.len() as u32)]
    entry_count: u32,

    #[brw(pad_before = 0x08)]
    #[br(count = entry_count)]
    pub entries: Vec<JumpTableEntry>,
}

impl SizeRange for JumpTable {
    const MIN_SIZE: u32 = 16;
    const MAX_SIZE: u32 = Self::MIN_SIZE;

    fn header_bytes_len(&self) -> u32 {
        Self::MIN_SIZE
    }

    fn total_bytes_len(&self) -> u32 {
        self.header_bytes_len() + (self.entries.len() as u32) * 12
    }
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JumpTableEntry {
    pub letter: u16,

    #[bw(calc = 0)]
    null: u16,

    pub start_index: u32,
    pub span: u32,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaylistPosition {
    #[brw(pad_after = 16)]
    pub position: u32,
}

impl SizeRange for PlaylistPosition {
    const MIN_SIZE: u32 = 20;
    const MAX_SIZE: u32 = Self::MIN_SIZE;

    fn header_bytes_len(&self) -> u32 {
        Self::MIN_SIZE
    }

    fn total_bytes_len(&self) -> u32 {
        self.header_bytes_len()
    }
}

#[binrw]
#[brw(little)]
#[br(import { payload_len: u32 })]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Raw {
    #[br(count = payload_len)]
    pub bytes: Vec<u8>,
}

impl SizeRange for Raw {
    const MIN_SIZE: u32 = 0;
    const MAX_SIZE: u32 = Self::MIN_SIZE;

    fn header_bytes_len(&self) -> u32 {
        Self::MIN_SIZE
    }

    fn total_bytes_len(&self) -> u32 {
        self.bytes.len() as u32
    }
}
