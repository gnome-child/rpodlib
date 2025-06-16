#![allow(non_camel_case_types, non_snake_case)]
#![allow(unused)]

use std::io::Cursor;

use binrw::{binrw, BinRead, BinWrite};

use crate::util::ByteCounter;

fn record_size(record: &Record) -> u32 {
    let mut counter = ByteCounter::new();

    record
        .write(&mut counter)
        .expect("writing to temporary buffer failed");
    counter.bytes()
}

fn update_len(record: &mut Record) {
    match record {
        Record::mhbd(master) => {
            for child in &mut master.children {
                update_len(child);
            }
            master.len = record_size(&Record::mhbd(master.clone()));
        }
        Record::mhsd(list_container) => {
            let list = match &mut list_container.list {
                List::Tracks(list)
                | List::Playlists(list)
                | List::Podcasts(list)
                | List::Albums(list)
                | List::InclSmartPlaylists(list) => list,
            };

            for child in &mut list.children {
                update_len(child);
            }
            list_container.len = record_size(&Record::mhsd(list_container.clone()));
        }
        Record::mhit(track) => {
            for child in &mut track.children {
                update_len(child);
            }
            track.len = record_size(&Record::mhit(track.clone()));
        }
        _ => {}
    }
}

pub(crate) fn write_to_buffer(record: &Record) -> Vec<u8> {
    let mut record = record.clone();
    let mut buf = Cursor::new(Vec::new());

    update_len(&mut record);

    record
        .write(&mut buf)
        .expect("binrw failed to write to buffer");

    buf.into_inner()
}

pub(crate) fn read_from_buffer(buf: &[u8]) -> Record {
    let mut cursor = Cursor::new(buf);

    Record::read(&mut cursor).expect("binrw failed to read from buffer")
}

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
    mhia(Unimplemented),

    /// Odd item record, playlist with playlist items
    #[brw(magic = b"mhyp")]
    mhyp(Unimplemented),

    #[brw(magic = b"mhip")]
    mhip(Unimplemented),

    /// Leaf record, variable len
    #[brw(magic = b"mhod")]
    mhod(Unimplemented),
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub(crate) struct Unimplemented {
    header_len: u32,
    len: u32,

    #[br(count = len - 12)]
    data: Vec<u8>,
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

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use binrw::BinRead;

    use crate::db::hash58;

    use super::{List, Record};

    #[test]
    fn test_hash58() {
        const FWID: &str = "000A270013E10993";
        const HASH_OFFSET: usize = 0x58;
        const HASH_LEN: usize = 20;

        let mut buf = include_bytes!("./sample/iTunesDB").to_vec();

        let stored_hash: [u8; HASH_LEN] =
            buf[HASH_OFFSET..HASH_OFFSET + HASH_LEN].try_into().unwrap();

        buf[HASH_OFFSET..HASH_OFFSET + HASH_LEN].fill(0);
        buf[0x18..0x20].fill(0);

        let new_hash = hash58::generate_hash58(FWID, &buf).expect("failed to hash database");

        assert_eq!(stored_hash, new_hash);
    }

    #[test]
    fn read_write_hash() {
        const FWID: &str = "000A270013E10993";
        const HASH_OFFSET: usize = 0x58;
        const HASH_LEN: usize = 20;

        let on_disk = include_bytes!("./sample/iTunesDB");
        let on_disk_copy = on_disk.to_vec().clone();

        let stored_hash: [u8; HASH_LEN] = on_disk[HASH_OFFSET..HASH_OFFSET + HASH_LEN]
            .try_into()
            .unwrap();

        let master = super::read_from_buffer(on_disk);
        let mut written = super::write_to_buffer(&master).to_vec();
        let written_copy = written.clone();

        written[HASH_OFFSET..HASH_OFFSET + HASH_LEN].fill(0);
        written[0x18..0x20].fill(0);

        let new_hash = hash58::generate_hash58(FWID, &written).expect("failed to hash database");

        crate::util::print_byte_diffs(&on_disk_copy, &written_copy);

        assert_eq!(stored_hash, new_hash);
    }

    #[test]
    fn parse_itdb() {
        let bytes = include_bytes!("./sample/iTunesDB");
        let mut cursor = Cursor::new(&bytes[..]);
        let mut root: Record = Record::read(&mut cursor).expect("failed");

        match &root {
            Record::mhbd(mhbd) => {
                println!("found master record");

                for mhsd in &mhbd.children {
                    match mhsd {
                        Record::mhsd(mhsd) => {
                            println!("found list container");

                            match &mhsd.list {
                                List::Tracks(mhlt) => {
                                    println!("found track list");
                                }
                                List::Playlists(mhlp) => {
                                    println!("found playlist list");
                                }
                                List::Podcasts(mhlp) => {
                                    println!("found podcast playlists");
                                }
                                List::Albums(mhla) => {
                                    println!("found album list");
                                }
                                List::InclSmartPlaylists(mhlp) => {
                                    println!("found playlist list incl smart playlists");
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}
