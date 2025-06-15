#![allow(non_snake_case)]
#![allow(unused)]

use binrw::binrw;

use super::data_obj::DataObject;

#[binrw]
#[brw(little, magic = b"mhyp")]
#[derive(Debug)]
pub(crate) struct Playlist {
    pub header_len: u32,
    pub len: u32,

    #[bw(calc = data_objects.len() as u32)]
    pub data_obj_count: u32,

    #[bw(calc = entries.len() as u32)]
    pub entry_count: u32,

    pub is_master_flag: u8, // if true, this is the master playlist, containing all the tracks
    pub flag_0x15: u8,
    pub flag_0x16: u8,
    pub flag_0x17: u8,
    pub hfs_timestamp_0x18: u32,
    pub persistent_id: u64,
    pub unk_0x24: u32, // always 0?
    pub string_obj_count: u16,
    pub is_podcast_playlist_flag: u16,
    pub sort_order: u32,
    pub padding_0x30: [u8; 40],
    pub hfs_timestamp_0x58: u32,

    #[brw(pad_before = 92)]
    #[br(count = data_obj_count)]
    pub data_objects: Vec<DataObject>,

    #[br(count = entry_count)]
    pub entries: Vec<PlaylistItem>,
}

#[binrw]
#[brw(little, magic = b"mhip")]
#[derive(Debug)]
pub(crate) struct PlaylistItem {
    pub header_len: u32,
    pub len: u32,

    #[bw(calc = data_objects.len() as u32)]
    pub data_obj_count: u32,

    pub podcast_group_flag: u16,
    pub unk_0x18: u16,
    pub group_id: u32, // doesn't seem to actually be useful
    pub track_id: u32, // corresponds to an actual track in the track list
    pub hfs_timestamp_0x28: u32,
    pub padding_0x32: [u8; 12],
    pub podcast_group_id: u32, // parent group that the podcast should be under, 0 for all other cases
    pub unk_0x48: u32,
    pub padding_0x52: [u8; 8],
    pub unk_0x60: u64,

    #[brw(pad_before = 8)]
    #[br(count = data_obj_count)]
    pub data_objects: Vec<DataObject>,
}
