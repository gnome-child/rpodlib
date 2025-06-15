#![allow(non_snake_case)]
#![allow(unused)]

use super::{util, ByteSerializable};

#[repr(packed(1))]
#[derive(Debug, Clone, Copy)]
pub(crate) struct DatabaseHeader {
    tag: [u8; 4],
    header_len: u32,
    len: u32,
    unk_0x0C: u32, // usually 1?
    version: u32,  // version of the database
    data_set_count: u32,
    database_id: u64,
    unk_0x20: u16,          // usually 2?
    hashing_scheme: u16,    // identified as hashing scheme
    unk_0x24: u64,          // some kind of id looking field
    padding_0x2C: [u8; 26], // 26 empty bytes
    lang: u16,              // usually en
    persistent_id: u64,
    unk_0x50: u32,          // varies per database
    unk_0x54: u32,          // varies per database
    padding_0x58: [u8; 20], // 20 empty bytes
    timezone_offset: i32,
    padding_0x70: [u8; 48],
    unk_0xA0: u32,          // usually 0xFFFFFFF?
    audio_lang: u16,        // usually 25?
    subtitle_lang: u16,     // usually 10?
    padding_0xA8: [u8; 76], // padded to end of header
}

impl Default for DatabaseHeader {
    fn default() -> Self {
        Self {
            tag: *b"mhbd",
            header_len: std::mem::size_of::<Self>() as u32,
            len: std::mem::size_of::<Self>() as u32,
            unk_0x0C: 1,
            version: 0x70,
            data_set_count: 0,
            database_id: 0, // TODO: generate id
            unk_0x20: 2,
            hashing_scheme: 1,
            unk_0x24: 0,
            padding_0x2C: [0; 26],
            lang: 0x656E,
            persistent_id: 0, // TODO: generate a persistent id
            unk_0x50: 0,
            unk_0x54: 0,
            padding_0x58: [0; 20],
            timezone_offset: 0,
            padding_0x70: [0; 48],
            unk_0xA0: 0xFFFFFFFF,
            audio_lang: 25,
            subtitle_lang: 10,
            padding_0xA8: [0; 76],
        }
    }
}

#[repr(packed(1))]
#[derive(Debug, Clone, Copy)]
pub(crate) struct ListContainerHeader {
    tag: [u8; 4],
    header_len: u32,
    len: u32,
    data_set_type: u32,
    padding: [u8; 80],
}

impl Default for ListContainerHeader {
    fn default() -> Self {
        Self {
            tag: *b"mhsd",
            header_len: std::mem::size_of::<Self>() as u32,
            len: std::mem::size_of::<Self>() as u32,
            data_set_type: 1,
            padding: [0; 80],
        }
    }
}

#[repr(packed(1))]
#[derive(Debug, Clone, Copy)]
pub(crate) struct TrackListHeader {
    tag: [u8; 4],
    header_len: u32,
    track_item_count: u32,
    padding: [u8; 80],
}

impl Default for TrackListHeader {
    fn default() -> Self {
        Self {
            tag: *b"mhlt",
            header_len: std::mem::size_of::<Self>() as u32,
            track_item_count: 0,
            padding: [0; 80],
        }
    }
}

#[repr(packed(1))]
#[derive(Debug, Clone, Copy)]
pub(crate) struct AlbumListHeader {
    tag: [u8; 4],
    header_len: u32,
    album_item_count: u32,
    padding: [u8; 80],
}

impl Default for AlbumListHeader {
    fn default() -> Self {
        Self {
            tag: *b"mhla",
            header_len: std::mem::size_of::<Self>() as u32,
            album_item_count: 0,
            padding: [0; 80],
        }
    }
}

#[repr(packed(1))]
#[derive(Debug, Clone, Copy)]
pub(crate) struct PlaylistListHeader {
    tag: [u8; 4],
    header_len: u32,
    playlist_count: u32,
    padding: [u8; 80],
}

impl Default for PlaylistListHeader {
    fn default() -> Self {
        Self {
            tag: *b"mhlp",
            header_len: std::mem::size_of::<Self>() as u32,
            playlist_count: 0,
            padding: [0; 80],
        }
    }
}

#[repr(packed(1))]
#[derive(Debug, Clone, Copy)]
pub(crate) struct TrackItemHeader {
    tag: [u8; 4],
    header_len: u32,
    len: u32,
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
    padding_0x0210: [u8; 96],
}

impl Default for TrackItemHeader {
    fn default() -> Self {
        Self {
            tag: *b"mhit",
            header_len: std::mem::size_of::<Self>() as u32,
            len: std::mem::size_of::<Self>() as u32,
            data_obj_count: 0,
            visible: 1,
            unique_id: 0, // TODO: generate this
            file_type: *b"    ",
            vbr_flag: 0,
            mp3_flag: 0,
            compilation_flag: 0,
            rating: 0,
            hfs_time_last_modified: util::get_current_time_hfs(),
            file_size_bytes_u32: 0,
            duration_ms: 0,
            album_index: 1,
            album_track_count: 1,
            release_year: 1517,
            bitrate: 0,
            sample_rate: 0,
            playback_volume_adj: 0,
            start_offset_ms: 0,
            stop_offset_ms: 0,
            soundcheck: 0,
            play_count_1: 0,
            play_count_2: 0,
            hfs_time_last_played: util::get_current_time_hfs(),
            album_disc_index: 1,
            album_disc_count: 1,
            drm_user_id: 0,
            hfs_time_date_added: util::get_current_time_hfs(),
            bookmark_ms: 0,
            persistent_id: 0, // TODO: generate this
            unchecked_flag: 0,
            last_rating: 0,
            bpm: 0,
            artwork_count: 0,
            audio_format_tag: 0,
            artwork_size_bytes: 0,
            unk_0x84: 0,
            IEEE_f32_sample_rate: 0,
            hfs_time_release_date: util::get_current_time_hfs(),
            unk_0x90: 0,
            unk_0x92: 0,
            unk_0x94: 0,
            unk_0x98: 0,
            skip_count: 0,
            hfs_time_last_skipped: util::get_current_time_hfs(),
            has_artwork: 0,
            skip_on_shuffle_flag: 0,
            remember_playback_position_flag: 0,
            podcast_flag: 0,
            unk_0xA8: 0,
            has_lyrics_flag: 0,
            is_movie_flag: 0,
            podcast_unplayed: 1,
            unk_0xB3: 0,
            unk_0xB4: 0,
            samples_before_start_gapless: 0,
            samples_count_gapless: 0,
            unk_0xC4: 0,
            samples_before_end_gapless: 0,
            mp3_encoded: 0,
            media_type: 1,
            season_number: 0,
            episode_number: 0,
            unk_0xDC: 0,
            padding_0xE0: [0; 24],
            gapless_data: 0,
            unk_0xFC: 0,
            is_gapless_track_flag: 0,
            is_gapless_album_flag: 0,
            padding_0x0104: [0; 28],
            unk_0x0120: 0x8DDA0000,
            unk_0x0124: 0,
            file_size_bytes_u64: 0,
            unk_0x0134: [0x80; 6],
            album_id: 0,
            padding_0x013A: [0; 36],
            mhii_link: 0,
            unk_0x0168: 0,
            padding_0x0170: [0; 112],
            unk_0x01E0: 0,
            padding_0x01E4: [0; 16],
            unk_0x01F4: 0,
            padding_0x01F8: [0; 20],
            unk_0x020C: 0,
            padding_0x0210: [0; 96],
        }
    }
}

#[repr(packed(1))]
#[derive(Debug, Clone, Copy)]
pub(crate) struct AlbumItemHeader {
    tag: [u8; 4],
    header_len: u32,
    len: u32,
    data_obj_count: u32,
    unk_0x10: u32, // looks like the track ids and such that start with 5
    unk_0x14: u64, // looks like an id
    unk_0x1C: u32, // always 2?
    unk_0x20: u64, // looks like another id
    padding_0x28: [u8; 48],
}

impl Default for AlbumItemHeader {
    fn default() -> Self {
        Self {
            tag: *b"mhia",
            header_len: std::mem::size_of::<Self>() as u32,
            len: std::mem::size_of::<Self>() as u32,
            data_obj_count: 0,
            unk_0x10: 0,
            unk_0x14: 0,
            unk_0x1C: 2,
            unk_0x20: 0,
            padding_0x28: [0; 48],
        }
    }
}

#[repr(packed(1))]
#[derive(Debug, Clone, Copy)]
pub(crate) struct PlaylistHeader {
    tag: [u8; 4],
    header_len: u32,
    len: u32,
    data_obj_count: u32,
    playlist_item_count: u32,
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
    padding_0x5C: [u8; 92],
}

impl Default for PlaylistHeader {
    fn default() -> Self {
        Self {
            tag: *b"mhyp",
            header_len: std::mem::size_of::<Self>() as u32,
            len: std::mem::size_of::<Self>() as u32,
            data_obj_count: 0,
            playlist_item_count: 0,
            is_master_flag: 0,
            flag_0x15: 0,
            flag_0x16: 0,
            flag_0x17: 0,
            hfs_timestamp_0x18: util::get_current_time_hfs(),
            persistent_id: 0, // TODO: generate this
            string_obj_count: 0,
            unk_0x24: 0,
            is_podcast_playlist_flag: 0,
            sort_order: 0,
            padding_0x30: [0; 40],
            hfs_timestamp_0x58: util::get_current_time_hfs(),
            padding_0x5C: [0; 92],
        }
    }
}

#[repr(packed(1))]
#[derive(Debug, Clone, Copy)]
pub(crate) struct PlaylistItemHeader {
    tag: [u8; 4],
    header_len: u32,
    len: u32,
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
    padding_0x68: [u8; 8],
}

impl Default for PlaylistItemHeader {
    fn default() -> Self {
        Self {
            tag: *b"mhip",
            header_len: std::mem::size_of::<Self>() as u32,
            len: std::mem::size_of::<Self>() as u32,
            data_obj_count: 0,
            podcast_group_flag: 0,
            unk_0x18: 0,
            group_id: 0,
            track_id: 0,
            hfs_timestamp_0x28: util::get_current_time_hfs(),
            padding_0x32: [0; 12],
            podcast_group_id: 0,
            unk_0x48: 0,
            padding_0x52: [0; 8],
            unk_0x60: 0,
            padding_0x68: [0; 8],
        }
    }
}

pub enum DataObjectHeader {
    String(StringObjectHeader),
    PodcastUrl(PodcastUrlObjectHeader),
    Unimplemented(UnimplementedObjectHeader),
}

#[repr(packed(1))]
#[derive(Debug, Clone, Copy)]
pub(crate) struct StringObjectHeader {
    tag: [u8; 4],
    header_len: u32,
    len: u32,
    data_type: u32,
    padding_0x10: [u8; 8],
    position: u32,   // setting this to zero on type 2 mhod will cause track not to play
    string_len: u32, // length of the string in utf-16. keep shorter than 512!
    unk_0x20: u32,
    unk_0x24: u32,
}

impl ByteSerializable for StringObjectHeader {}

impl Default for StringObjectHeader {
    fn default() -> Self {
        Self {
            tag: *b"mhod",
            header_len: std::mem::size_of::<Self>() as u32,
            len: std::mem::size_of::<Self>() as u32,
            data_type: 0,
            padding_0x10: [0; 8],
            position: 1,
            string_len: 0,
            unk_0x20: 0,
            unk_0x24: 0,
        }
    }
}

#[repr(packed(1))]
#[derive(Debug, Clone, Copy)]
pub(crate) struct PodcastUrlObjectHeader {
    tag: [u8; 4],
    header_len: u32,
    len: u32,
    data_type: u32,
    padding_0x10: [u8; 8],
}

impl ByteSerializable for PodcastUrlObjectHeader {}

impl Default for PodcastUrlObjectHeader {
    fn default() -> Self {
        Self {
            tag: *b"mhod",
            header_len: std::mem::size_of::<Self>() as u32,
            len: std::mem::size_of::<Self>() as u32,
            data_type: 0,
            padding_0x10: [0; 8],
        }
    }
}

#[repr(packed(1))]
#[derive(Debug, Clone, Copy)]
pub(crate) struct UnimplementedObjectHeader {
    tag: [u8; 4],
    header_len: u32,
    len: u32,
    data_type: u32,
    padding_0x10: [u8; 8],
}

impl ByteSerializable for UnimplementedObjectHeader {}

impl Default for UnimplementedObjectHeader {
    fn default() -> Self {
        Self {
            tag: *b"mhod",
            header_len: std::mem::size_of::<Self>() as u32,
            len: std::mem::size_of::<Self>() as u32,
            data_type: 0,
            padding_0x10: [0; 8],
        }
    }
}
