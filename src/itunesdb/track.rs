//! rpodlib – iTunesDB parser & writer
//!
//! SPDX-License-Identifier: GPL-3.0-or-later
//! Copyright © 2025 gnome-child

#![allow(non_snake_case)]
#![allow(unused)]

use binrw::binrw;

use super::data_obj::DataObject;

#[binrw]
#[brw(little, magic = b"mhit")]
#[derive(Debug)]
pub(crate) struct TrackItem {
    pub header_len: u32,
    pub len: u32,

    #[bw(calc = data_objects.len() as u32)]
    pub data_obj_count: u32,

    pub unique_id: u32, // unique id for the track, used by playlists
    pub visible: u32,
    pub file_type: [u8; 4], // looks big endian. file extension padded with spaces (ie: ' 3PM')
    pub vbr_flag: u8,
    pub mp3_flag: u8,
    pub compilation_flag: u8,
    pub rating: u8,
    pub hfs_time_last_modified: u32, // all timestamps are likely in apples hfs+ format
    pub file_size_bytes_u32: u32,
    pub duration_ms: u32,
    pub album_index: u32,
    pub album_track_count: u32,
    pub release_year: u32,
    pub bitrate: u32,
    pub sample_rate: u32, // number stored here is sample rate of file mult by 0x10000
    pub playback_volume_adj: u32, // can be anything from -255 to 255, adjust volume on playback (replaygain??) set in itunes
    pub start_offset_ms: u32,
    pub stop_offset_ms: u32,
    pub soundcheck: u32, // works with replay gain, soundcheck = 1000 * 10^(-.1 * y) where y is adjustment in dB
    pub play_count_1: u32,
    pub play_count_2: u32, // need to see if this is ever different from above play count
    pub hfs_time_last_played: u32,
    pub album_disc_index: u32,
    pub album_disc_count: u32,
    pub drm_user_id: u32, // most likely just needs to be zero unless the user has drm protected files
    pub hfs_time_date_added: u32,
    pub bookmark_ms: u32, // used for .aa and .m4b files, ipod might actually set this in play counts file instead
    pub persistent_id: u64, // id used to link across database files (eg mhit -> mhii in artworkdb)
    pub unchecked_flag: u8, // unchecked in itunes true/false
    pub last_rating: u8,  // rating from itunes before sync goes here for some reason
    pub bpm: u16,
    pub artwork_count: u16,
    pub audio_format_tag: u16, // 0xFFFF for mp3/aac, 0x0 for uncompressed (wav), 0x1 for audible
    pub artwork_size_bytes: u32, // size of artwork (likely in metadata of audio file?)
    pub unk_0x84: u32,         // always seems to be 0
    pub IEEE_f32_sample_rate: u32, // sample rate as IEEE f32?
    pub hfs_time_release_date: u32,
    pub unk_0x90: u16, // encoding-related info? see wikipodlinux
    pub unk_0x92: u16, // some kind of played flag?
    pub unk_0x94: u32, // 0x01010100 if has apple drm?
    pub unk_0x98: u32,
    pub skip_count: u32,
    pub hfs_time_last_skipped: u32,
    pub has_artwork: u8, // 0x02 for tracks without artwork, 0x01 for tracks with artwork
    pub skip_on_shuffle_flag: u8, // recommended set to true for intro tracks/podcasts
    pub remember_playback_position_flag: u8, // set to true for files that aren't audiobooks to enable bookmark field
    pub podcast_flag: u8, // 0x1 won't show artist name, if podcast must be set to 0x01 or 0x02
    pub unk_0xA8: u64,    // some kind of id
    pub has_lyrics_flag: u8,
    pub is_movie_flag: u8,
    pub podcast_unplayed: u8, // 0x01 for non podcasts, 0x02 marks podcasts with a bullet (not played)
    pub unk_0xB3: u8,         // seems to be always 0
    pub unk_0xB4: u32,        // seems to be always 0
    pub samples_before_start_gapless: u32,
    pub samples_count_gapless: u64,
    pub unk_0xC4: u32, // seems to be always 0
    pub samples_before_end_gapless: u32,
    pub mp3_encoded: u32,       // set to 1 for mp3 encoding?
    pub media_type: u32,        // VERY IMPORTANT, denotes media type
    pub season_number: u32,     // for tv shows only
    pub episode_number: u32,    // for tv shows only
    pub unk_0xDC: u32,          // seems to be 0x01 for protected files?
    pub padding_0xE0: [u8; 24], // might be fields, looks like all 0s
    pub gapless_data: u32,      // size in bytes from first synch frame, can be 0 for AAC
    pub unk_0xFC: u32,
    pub is_gapless_track_flag: u16,
    pub is_gapless_album_flag: u16,
    pub padding_0x0104: [u8; 28],
    pub unk_0x0120: u32,          // seems to be set to 0x8DDA0000 across samples
    pub unk_0x0124: u64, // seems to be set to across track_items in database, possibly an id
    pub file_size_bytes_u64: u64, // seems to be the size of the track in bytes again, possibly as u64
    pub unk_0x0134: [u8; 6],      // each byte seems to be set to 0x80
    pub album_id: u16,            // supposedly an album id, looks like its always 0 though
    pub padding_0x013A: [u8; 36], // looks like padding
    pub mhii_link: u64,           // need more research
    pub unk_0x0168: u64,          // seems to always be 0x20
    pub padding_0x0170: [u8; 112],
    pub unk_0x01E0: u32, // seems to always be 0x30DB
    pub padding_0x01E4: [u8; 16],
    pub unk_0x01F4: u32,
    pub padding_0x01F8: [u8; 20],
    pub unk_0x020C: u32,

    #[brw(pad_before = 96)]
    #[br(count = data_obj_count)]
    pub data_objects: Vec<DataObject>,
}

