//! rpodlib – iTunesDB parser & writer
//!
//! SPDX-License-Identifier: GPL-3.0-or-later
//! Copyright © 2025 gnome-child

#![allow(unused)]

pub(crate) mod data_obj;
pub(crate) mod data_set;
pub(crate) mod image;
pub(crate) mod image_meta;
pub(crate) mod master;

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use binrw::BinRead;

    use super::master::Master;

    #[test]
    fn test_artworkdb() {
        let bytes = include_bytes!("./sample/ArtworkDB");
        let mut cursor = Cursor::new(&bytes[..]);
        let master: Master = Master::read(&mut cursor).expect("failed");
    }
}

