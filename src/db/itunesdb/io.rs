#![allow(unused)]

use std::io::Cursor;

use anyhow::ensure;
use binrw::{binrw, BinRead, BinWrite};

use super::{List, Record};
use crate::util::ByteCounter;

fn get_record_size(record: &Record) -> u32 {
    let mut counter = ByteCounter::new();

    record
        .write(&mut counter)
        .expect("writing to temporary buffer failed");
    counter.bytes()
}

//TODO: this is very inefficient, consider other methods
fn update_len(record: &mut Record) {
    match record {
        Record::mhbd(master) => {
            for child in &mut master.children {
                update_len(child);
            }
            master.len = get_record_size(&Record::mhbd(master.clone()));
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
            list_container.len = get_record_size(&Record::mhsd(list_container.clone()));
        }
        Record::mhit(track) => {
            for child in &mut track.children {
                update_len(child);
            }
            track.len = get_record_size(&Record::mhit(track.clone()));
        }
        Record::mhia(album) => {
            for child in &mut album.children {
                update_len(child);
            }
            album.len = get_record_size(&Record::mhia(album.clone()));
        }
        Record::mhyp(playlist) => {
            for child in &mut playlist.children {
                update_len(child);
            }

            for entry in &mut playlist.entries {
                update_len(entry);
            }
            playlist.len = get_record_size(&Record::mhyp(playlist.clone()));
        }
        Record::mhip(playlist_entry) => {
            for child in &mut playlist_entry.children {
                update_len(child);
            }
            playlist_entry.len = get_record_size(&Record::mhip(playlist_entry.clone()));
        }
        Record::mhod(data_container) => {
            data_container.len = get_record_size(&Record::mhod(data_container.clone()));
        }
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
