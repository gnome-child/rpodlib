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
    mhit(Unimplemented),

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
        let stored_hash: [u8; HASH_LEN] = on_disk[HASH_OFFSET..HASH_OFFSET + HASH_LEN]
            .try_into()
            .unwrap();

        let master = super::read_from_buffer(on_disk);
        let mut written = super::write_to_buffer(&master).to_vec();

        written[HASH_OFFSET..HASH_OFFSET + HASH_LEN].fill(0);
        written[0x18..0x20].fill(0);

        let new_hash = hash58::generate_hash58(FWID, &written).expect("failed to hash database");

        assert_eq!(stored_hash, new_hash);
    }

    #[test]
    fn parse_itdb() {
        let bytes = include_bytes!("./sample/iTunesDB");
        let mut cursor = Cursor::new(&bytes[..]);
        let root: Record = Record::read(&mut cursor).expect("failed");

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
