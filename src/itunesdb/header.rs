#![allow(non_snake_case)]
#![allow(unused)]

use binrw::binrw;

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::{BinRead, BinResult};
    use std::io::Cursor;

    #[test]
    fn test_itdb() {
        let bytes = include_bytes!("./sample/iTunesDB");
        let mut cursor = Cursor::new(&bytes[..]);
        let master: Master = Master::read(&mut cursor).expect("failed");

        println!("{:#?}", master);
    }
}

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

#[binrw]
#[brw(little, magic = b"mhsd")]
#[derive(Debug)]
pub(crate) struct DataSet {
    pub header_len: u32,
    pub len: u32,
    pub data_type: u32,

    #[br(args { id: data_type })]
    #[brw(pad_before = 80)]
    pub data: Data,
}

#[binrw]
#[br(import { id: u32 })]
#[derive(Debug)]
pub enum Data {
    #[br(pre_assert(id == 0x01))] // ← pick this variant when id == 1
    TrackList(TrackList),

    #[br(pre_assert(id == 0x02))]
    PlaylistList(PlaylistList),

    #[br(pre_assert(id == 0x03))]
    PodcastList(PlaylistList),

    #[br(pre_assert(id == 0x04))]
    AlbumList(AlbumList),

    #[br(pre_assert(id == 0x05))]
    SmartPlaylistList(PlaylistList),
}

#[binrw]
#[brw(little, magic = b"mhlt")]
#[derive(Debug)]
pub(crate) struct TrackList {
    pub header_len: u32,

    #[bw(calc = entries.len() as u32)]
    pub entry_count: u32,

    #[brw(pad_before = 80)]
    #[br(count = entry_count)]
    pub entries: Vec<TrackItem>,
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
    pub entries: Vec<AlbumItem>,
}

#[binrw]
#[brw(little, magic = b"mhlp")]
#[derive(Debug)]
pub(crate) struct PlaylistList {
    pub header_len: u32,

    #[bw(calc = entries.len() as u32)]
    pub entry_count: u32,

    #[brw(pad_before = 80)]
    #[br(count = entry_count)]
    pub entries: Vec<Playlist>,
}

#[binrw]
#[brw(little, magic = b"mhit")]
#[derive(Debug)]
pub(crate) struct TrackItem {
    header_len: u32,
    len: u32,

    #[bw(calc = data_objects.len() as u32)]
    data_obj_count: u32,

    unique_id: u32, // unique id for the track, used by playlists
    visible: u32,
    file_type: [u8; 4], // looks big endian. file extension padded with spaces (ie: ' 3PM')
    vbr_flag: u8,
    mp3_flag: u8,
    compilation_flag: u8,
    rating: u8,
    hfs_time_last_modified: u32, // all timestamps are likely in apples hfs+ format
    file_size_bytes_u32: u32,
    duration_ms: u32,
    album_index: u32,
    album_track_count: u32,
    release_year: u32,
    bitrate: u32,
    sample_rate: u32, // number stored here is sample rate of file mult by 0x10000
    playback_volume_adj: u32, // can be anything from -255 to 255, adjust volume on playback (replaygain??) set in itunes
    start_offset_ms: u32,
    stop_offset_ms: u32,
    soundcheck: u32, // works with replay gain, soundcheck = 1000 * 10^(-.1 * y) where y is adjustment in dB
    play_count_1: u32,
    play_count_2: u32, // need to see if this is ever different from above play count
    hfs_time_last_played: u32,
    album_disc_index: u32,
    album_disc_count: u32,
    drm_user_id: u32, // most likely just needs to be zero unless the user has drm protected files
    hfs_time_date_added: u32,
    bookmark_ms: u32, // used for .aa and .m4b files, ipod might actually set this in play counts file instead
    persistent_id: u64, // id used to link across database files (eg mhit -> mhii in artworkdb)
    unchecked_flag: u8, // unchecked in itunes true/false
    last_rating: u8,  // rating from itunes before sync goes here for some reason
    bpm: u16,
    artwork_count: u16,
    audio_format_tag: u16, // 0xFFFF for mp3/aac, 0x0 for uncompressed (wav), 0x1 for audible
    artwork_size_bytes: u32, // size of artwork (likely in metadata of audio file?)
    unk_0x84: u32,         // always seems to be 0
    IEEE_f32_sample_rate: u32, // sample rate as IEEE f32?
    hfs_time_release_date: u32,
    unk_0x90: u16, // encoding-related info? see wikipodlinux
    unk_0x92: u16, // some kind of played flag?
    unk_0x94: u32, // 0x01010100 if has apple drm?
    unk_0x98: u32,
    skip_count: u32,
    hfs_time_last_skipped: u32,
    has_artwork: u8, // 0x02 for tracks without artwork, 0x01 for tracks with artwork
    skip_on_shuffle_flag: u8, // recommended set to true for intro tracks/podcasts
    remember_playback_position_flag: u8, // set to true for files that aren't audiobooks to enable bookmark field
    podcast_flag: u8, // 0x1 won't show artist name, if podcast must be set to 0x01 or 0x02
    unk_0xA8: u64,    // some kind of id
    has_lyrics_flag: u8,
    is_movie_flag: u8,
    podcast_unplayed: u8, // 0x01 for non podcasts, 0x02 marks podcasts with a bullet (not played)
    unk_0xB3: u8,         // seems to be always 0
    unk_0xB4: u32,        // seems to be always 0
    samples_before_start_gapless: u32,
    samples_count_gapless: u64,
    unk_0xC4: u32, // seems to be always 0
    samples_before_end_gapless: u32,
    mp3_encoded: u32,       // set to 1 for mp3 encoding?
    media_type: u32,        // VERY IMPORTANT, denotes media type
    season_number: u32,     // for tv shows only
    episode_number: u32,    // for tv shows only
    unk_0xDC: u32,          // seems to be 0x01 for protected files?
    padding_0xE0: [u8; 24], // might be fields, looks like all 0s
    gapless_data: u32,      // size in bytes from first synch frame, can be 0 for AAC
    unk_0xFC: u32,
    is_gapless_track_flag: u16,
    is_gapless_album_flag: u16,
    padding_0x0104: [u8; 28],
    unk_0x0120: u32,          // seems to be set to 0x8DDA0000 across samples
    unk_0x0124: u64,          // seems to be set to across track_items in database, possibly an id
    file_size_bytes_u64: u64, // seems to be the size of the track in bytes again, possibly as u64
    unk_0x0134: [u8; 6],      // each byte seems to be set to 0x80
    album_id: u16,            // supposedly an album id, looks like its always 0 though
    padding_0x013A: [u8; 36], // looks like padding
    mhii_link: u64,           // need more research
    unk_0x0168: u64,          // seems to always be 0x20
    padding_0x0170: [u8; 112],
    unk_0x01E0: u32, // seems to always be 0x30DB
    padding_0x01E4: [u8; 16],
    unk_0x01F4: u32,
    padding_0x01F8: [u8; 20],
    unk_0x020C: u32,

    #[brw(pad_before = 96)]
    #[br(count = data_obj_count)]
    data_objects: Vec<DataObject>,
}

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

#[binrw]
#[brw(little, magic = b"mhyp")]
#[derive(Debug)]
pub(crate) struct Playlist {
    header_len: u32,
    len: u32,

    #[bw(calc = data_objects.len() as u32)]
    data_obj_count: u32,

    #[bw(calc = entries.len() as u32)]
    entry_count: u32,

    is_master_flag: u8, // if true, this is the master playlist, containing all the tracks
    flag_0x15: u8,
    flag_0x16: u8,
    flag_0x17: u8,
    hfs_timestamp_0x18: u32,
    persistent_id: u64,
    unk_0x24: u32, // always 0?
    string_obj_count: u16,
    is_podcast_playlist_flag: u16,
    sort_order: u32,
    padding_0x30: [u8; 40],
    hfs_timestamp_0x58: u32,

    #[brw(pad_before = 92)]
    #[br(count = data_obj_count)]
    data_objects: Vec<DataObject>,

    #[br(count = entry_count)]
    entries: Vec<PlaylistItem>,
}

#[binrw]
#[brw(little, magic = b"mhip")]
#[derive(Debug)]
pub(crate) struct PlaylistItem {
    header_len: u32,
    len: u32,

    #[bw(calc = data_objects.len() as u32)]
    data_obj_count: u32,

    podcast_group_flag: u16,
    unk_0x18: u16,
    group_id: u32, // doesn't seem to actually be useful
    track_id: u32, // corresponds to an actual track in the track list
    hfs_timestamp_0x28: u32,
    padding_0x32: [u8; 12],
    podcast_group_id: u32, // parent group that the podcast should be under, 0 for all other cases
    unk_0x48: u32,
    padding_0x52: [u8; 8],
    unk_0x60: u64,

    #[brw(pad_before = 8)]
    #[br(count = data_obj_count)]
    data_objects: Vec<DataObject>,
}

#[binrw]
#[brw(little, magic = b"mhod")]
#[derive(Debug)]
pub(crate) struct DataObject {
    header_len: u32,
    len: u32,

    #[br(pad_before = len - 12)]
    _skipped_bytes: (),
}
