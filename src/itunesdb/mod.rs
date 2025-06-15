//! rpodlib – iTunesDB parser & writer
//!
//! SPDX-License-Identifier: GPL-3.0-or-later
//! Copyright © 2025 gnome-child

#![allow(unused)]

use chrono::Local;
use rand::{rngs::OsRng, TryRngCore};

pub(crate) mod album;
pub(crate) mod data_obj;
pub(crate) mod dataset;
pub(crate) mod hash58;
pub(crate) mod master;
pub(crate) mod playlist;
pub(crate) mod track;

pub(crate) fn generate_persistent_id() -> u64 {
    let mut rng = OsRng;
    return rng
        .try_next_u64()
        .expect("Couldn't generate a persistent id!");
}

pub(crate) fn generate_db_id() -> u64 {
    return rand::random::<u64>();
}

pub(crate) fn get_sys_tz_offset() -> i32 {
    return Local::now().offset().local_minus_utc() as i32;
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, io::Cursor};

    use binrw::BinReaderExt;

    use crate::itunesdb::data_obj::ObjectType;

    use super::master::Master;

    fn load_sample_file() -> &'static [u8] {
        return include_bytes!("./sample/iTunesDB");
    }

    #[test]
    fn get_sample_file() {
        let sample_bytes = load_sample_file();

        assert!(sample_bytes.len() > 0)
    }

    #[test]
    fn generate_id() {
        println!(
            "random persistent id: {:?}",
            super::generate_persistent_id()
        );
    }

    #[test]
    fn get_tz_offset() {
        let offset_in_sec = super::get_sys_tz_offset();
        println!("host tz offset: {:?}", offset_in_sec)
    }

    #[test]
    fn test_hash58() -> Result<(), Box<dyn std::error::Error>> {
        use super::hash58;

        const FWID: &str = "000A270013E10993";
        const HASH_OFFSET: usize = 0x58;
        const HASH_LEN: usize = 20; // 0x6C - 0x58

        // Load the file into a mutable Vec<u8>
        let mut itunesdb = include_bytes!("./sample/iTunesDB").to_vec();

        // Copy the on-disk hash *before* we blank it
        let stored_hash: [u8; HASH_LEN] = itunesdb[HASH_OFFSET..HASH_OFFSET + HASH_LEN]
            .try_into()
            .unwrap();

        // Wipe the hash field so it’s not part of the new digest
        itunesdb[HASH_OFFSET..HASH_OFFSET + HASH_LEN].fill(0);
        itunesdb[0x18..0x20].fill(0);

        // Recalculate
        let computed_hash = hash58::generate_hash58(FWID, &itunesdb)?;

        println!("stored   = {:02X?}", stored_hash);
        println!("computed = {:02X?}", computed_hash);

        assert_eq!(stored_hash, computed_hash);
        Ok(())
    }

    use super::dataset::SetType;
    use binrw::{BinRead, BinResult};

    #[test]
    fn test_itdb() {
        let bytes = include_bytes!("./sample/iTunesDB");
        let mut cursor = Cursor::new(&bytes[..]);
        let master: Master = Master::read(&mut cursor).expect("failed");

        println!("database id: {:#?}", master.database_id);
        println!("db tz offset: {:#?}", master.timezone_offset);
        println!("data sets: {:#?}", master.data_sets.len());

        for data_set in &master.data_sets {
            match &data_set.set {
                SetType::Tracks(tl) => {
                    for entry in &tl.entries {
                        for obj in &entry.data_objects {
                            match &obj.object {
                                ObjectType::Title(title) => {
                                    println!(
                                        "title: {}",
                                        title
                                            .to_string()
                                            .expect("Failed to convert bytes to string!")
                                    );
                                }
                                ObjectType::PodcastRssUrl(rss) => {
                                    println!(
                                        "rss url: {}",
                                        rss.to_string()
                                            .expect("Failed to convert bytes to string!")
                                    )
                                }
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

