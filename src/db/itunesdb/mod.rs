#![allow(unused, non_camel_case_types, non_snake_case)]

use std::io::Cursor;

use binrw::binrw;

pub(crate) mod io;

#[binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub(crate) enum Record {
    /// Upper level records
    #[brw(magic = b"mhbd")]
    mhbd(Master),

    #[brw(magic = b"mhsd")]
    mhsd(ListContainer),

    /// Lower level item records
    #[brw(magic = b"mhit")]
    mhit(Track),

    #[brw(magic = b"mhia")]
    mhia(Album),

    /// Odd item record, playlist with playlist items
    #[brw(magic = b"mhyp")]
    mhyp(Playlist),

    #[brw(magic = b"mhip")]
    mhip(PlaylistEntry),

    /// Leaf record, variable len
    #[brw(magic = b"mhod")]
    mhod(DataContainer),
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub(crate) struct Master {
    #[bw(calc = 244)]
    header_len: u32,

    len: u32,
    unk_0x0C: u32,
    version: u32,

    #[bw(calc = children.len() as u32)]
    child_count: u32,

    database_id: u64,
    unk_0x20: u16,
    hashing_scheme: u16,
    unk_0x24: u64,
    unk_0x2C: u32,
    unk_0x30: u16,
    padding_0x32: [u8; 20],
    lang: u16,
    persistent_id: u64,
    unk_0x50: u32,
    unk_0x54: u32,
    hash_0x58: [u8; 20],
    timezone_offset: i32,
    unk_0x70: u16,
    hash_0x72: [u8; 46],
    unk_0xA0: u32,
    audio_lang: u16,
    subtitle_lang: u16,

    #[brw(pad_before = 76)]
    #[br(count = child_count)]
    children: Vec<Record>,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub(crate) struct ListContainer {
    #[bw(calc = 96)]
    header_len: u32,

    len: u32,

    #[bw(calc = list.as_u32())]
    list_type: u32,

    #[brw(pad_before = 80)]
    #[br(args { list_type: list_type })]
    list: List,
}

#[binrw]
#[brw(little)]
#[br(import { list_type: u32 })]
#[derive(Debug, Clone)]
pub enum List {
    #[br(pre_assert(list_type == 0x01))]
    #[brw(magic = b"mhlt")]
    Tracks(RecordList),

    #[br(pre_assert(list_type == 0x02))]
    #[brw(magic = b"mhlp")]
    Playlists(RecordList),

    #[br(pre_assert(list_type == 0x03))]
    #[brw(magic = b"mhlp")]
    Podcasts(RecordList),

    #[br(pre_assert(list_type == 0x04))]
    #[brw(magic = b"mhla")]
    Albums(RecordList),

    #[br(pre_assert(list_type == 0x05))]
    #[brw(magic = b"mhlp")]
    InclSmartPlaylists(RecordList),
}

impl List {
    pub fn as_u32(&self) -> u32 {
        match self {
            List::Tracks(_) => 0x01,
            List::Playlists(_) => 0x02,
            List::Podcasts(_) => 0x03,
            List::Albums(_) => 0x04,
            List::InclSmartPlaylists(_) => 0x05,
        }
    }
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub(crate) struct RecordList {
    #[bw(calc = 92)]
    header_len: u32,

    #[bw(calc = children.len() as u32)]
    child_count: u32,

    #[brw(pad_before = 80)]
    #[br(count = child_count)]
    children: Vec<Record>,
}

// TODO: need to double check these fields
#[binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub(crate) struct Track {
    #[bw(calc = 624)]
    header_len: u32,

    len: u32,

    #[bw(calc = children.len() as u32)]
    child_count: u32,

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

    #[brw(pad_before = 28)]
    unk_0x22C: u32, // rogue 0x0000_0001, big endian flag?? seems to be set to 1 on podcasts

    #[brw(pad_before = 64)]
    #[br(count = child_count)]
    children: Vec<Record>,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub(crate) struct Album {
    #[bw(calc = 88)]
    header_len: u32,

    len: u32,

    #[bw(calc = children.len() as u32)]
    child_count: u32,

    unk_0x10: u32, // looks like the track ids and such that start with 5
    unk_0x14: u64, // looks like an id
    unk_0x1C: u32, // always 2?
    unk_0x20: u64, // looks like another id

    #[brw(pad_before = 48)]
    #[br(count = child_count)]
    children: Vec<Record>,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub(crate) struct Playlist {
    #[bw(calc = 184)]
    header_len: u32,

    len: u32,

    #[bw(calc = children.len() as u32)]
    child_count: u32,

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
    #[br(count = child_count)]
    children: Vec<Record>,

    #[br(count = entry_count)]
    entries: Vec<Record>,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub(crate) struct PlaylistEntry {
    #[bw(calc = 76)]
    header_len: u32,

    len: u32,

    #[bw(calc = children.len() as u32)]
    child_count: u32,

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
    #[br(count = child_count)]
    children: Vec<Record>,
}

#[binrw]
#[brw(little)]
#[br(import { data_type: u32, bytes_left: u32 })]
#[derive(Debug, Clone)]
pub enum Data {
    #[br(pre_assert(data_type ==  1))]
    Title(Utf16String),

    #[br(pre_assert(data_type ==  2))]
    Location(Utf16String),

    #[br(pre_assert(data_type ==  3))]
    Album(Utf16String),

    #[br(pre_assert(data_type ==  4))]
    Artist(Utf16String),

    #[br(pre_assert(data_type ==  5))]
    Genre(Utf16String),

    #[br(pre_assert(data_type ==  6))]
    Filetype(Utf16String),

    #[br(pre_assert(data_type ==  7))]
    EqSetting(Utf16String),

    #[br(pre_assert(data_type ==  8))]
    Comment(Utf16String),

    #[br(pre_assert(data_type ==  9))]
    Category(Utf16String),

    #[br(pre_assert(data_type == 12))]
    Composer(Utf16String),

    #[br(pre_assert(data_type == 13))]
    Grouping(Utf16String),

    #[br(pre_assert(data_type == 14))]
    Description(Utf16String),

    #[br(pre_assert(data_type == 15))]
    PodcastEnclosureUrl(#[br(args { bytes_left })] Utf8PodcastUrl),

    #[br(pre_assert(data_type == 16))]
    PodcastRssUrl(#[br(args { bytes_left })] Utf8PodcastUrl),

    #[br(pre_assert(data_type == 17))]
    ChapterData(#[br(args { bytes_left })] Blob),

    #[br(pre_assert(data_type == 18))]
    Subtitle(Utf16String),

    #[br(pre_assert(data_type == 19))]
    Show(#[br(args { bytes_left })] Blob),

    #[br(pre_assert(data_type == 20))]
    EpisodeNumber(#[br(args { bytes_left })] Blob),

    #[br(pre_assert(data_type == 21))]
    TvNetwork(#[br(args { bytes_left })] Blob),

    #[br(pre_assert(data_type == 22))]
    AlbumArtist(Utf16String),

    #[br(pre_assert(data_type == 23))]
    ArtistSort(#[br(args { bytes_left })] Blob),

    #[br(pre_assert(data_type == 24))]
    Keywords(#[br(args { bytes_left })] Blob),

    #[br(pre_assert(data_type == 25))]
    TvShowLocale(#[br(args { bytes_left })] Blob),

    #[br(pre_assert(data_type == 27))]
    TitleSort(#[br(args { bytes_left })] Blob),

    #[br(pre_assert(data_type == 28))]
    AlbumSort(#[br(args { bytes_left })] Blob),

    #[br(pre_assert(data_type == 29))]
    AlbumArtistSort(#[br(args { bytes_left })] Blob),

    #[br(pre_assert(data_type == 30))]
    ComposerSort(#[br(args { bytes_left })] Blob),

    #[br(pre_assert(data_type == 31))]
    TvShowSort(#[br(args { bytes_left })] Blob),

    #[br(pre_assert(data_type == 32))]
    UnknownVideoBinary(#[br(args { bytes_left })] Blob),

    #[br(pre_assert(data_type == 39))]
    Copyright(Utf16String),

    #[br(pre_assert(data_type == 50))]
    SmartPlaylistData(#[br(args { bytes_left })] Blob),

    #[br(pre_assert(data_type == 51))]
    SmartPlaylistRules(#[br(args { bytes_left })] Blob),

    #[br(pre_assert(data_type == 52))]
    LibraryPlaylistIndex(#[br(args { bytes_left })] Blob),

    #[br(pre_assert(data_type == 53))]
    JumpTable(#[br(args { bytes_left })] Blob),

    #[br(pre_assert(data_type == 100))]
    ColumnSizingAndOrder(#[br(args { bytes_left })] Blob),

    #[br(pre_assert(data_type == 102))]
    UnknownObject(#[br(args { bytes_left })] Blob),

    #[br(pre_assert(data_type == 200))]
    AlbumInAlbumList(#[br(args { bytes_left })] Blob),

    #[br(pre_assert(data_type == 201))]
    ArtistInAlbumList(#[br(args { bytes_left })] Blob),

    #[br(pre_assert(data_type == 202))]
    ArtistSortInAlbumList(#[br(args { bytes_left })] Blob),

    #[br(pre_assert(data_type == 203))]
    PodcastUrlInAlbumList(#[br(args { bytes_left })] Blob),

    #[br(pre_assert(data_type == 204))]
    TvShowInAlbumList(#[br(args { bytes_left })] Blob),
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub(crate) struct DataContainer {
    #[bw(calc = 24)]
    header_len: u32,

    len: u32,
    data_type: u32,

    #[brw(pad_before = 8)]
    #[br(args { data_type: data_type, bytes_left: len - header_len })]
    data: Data,
}

#[binrw]
#[brw(little)]
#[br(import { bytes_left: u32 })]
#[derive(Debug, Clone)]
pub(crate) struct Blob {
    #[br(count = bytes_left)]
    bytes: Vec<u8>,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub(crate) struct Utf16String {
    position: u32,

    #[bw(calc = string.len() as u32 * 2)]
    len: u32,

    unk_0x08: u32,
    unk_0x0C: u32,

    #[br(count = len / 2)]
    string: Vec<u16>,
}

#[binrw]
#[brw(little)]
#[br(import { bytes_left: u32 })]
#[derive(Debug, Clone)]
pub(crate) struct Utf8PodcastUrl {
    #[br(count = bytes_left)]
    url: Vec<u8>,
}
