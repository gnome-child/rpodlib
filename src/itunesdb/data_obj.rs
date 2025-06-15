//! rpodlib – iTunesDB parser & writer
//!
//! SPDX-License-Identifier: GPL-3.0-or-later
//! Copyright © 2025 gnome-child

#![allow(non_snake_case)]
#![allow(unused)]

use binrw::binrw;
use bytemuck::try_cast_slice;

#[binrw]
#[br(import { id: u32, bytes_left: u32 })]
#[derive(Debug)]
pub(crate) enum ObjectType {
    // 1‒9 ---------------------------------------------------------------
    #[br(pre_assert(id ==  1))]
    Title(#[br(args { bytes_left })] Utf16StringObj),
    #[br(pre_assert(id ==  2))]
    Location(#[br(args { bytes_left })] Utf16StringObj),
    #[br(pre_assert(id ==  3))]
    Album(#[br(args { bytes_left })] Utf16StringObj),
    #[br(pre_assert(id ==  4))]
    Artist(#[br(args { bytes_left })] Utf16StringObj),
    #[br(pre_assert(id ==  5))]
    Genre(#[br(args { bytes_left })] Utf16StringObj),
    #[br(pre_assert(id ==  6))]
    Filetype(#[br(args { bytes_left })] Utf16StringObj),
    #[br(pre_assert(id ==  7))]
    EqSetting(#[br(args { bytes_left })] Utf16StringObj),
    #[br(pre_assert(id ==  8))]
    Comment(#[br(args { bytes_left })] Utf16StringObj),
    #[br(pre_assert(id ==  9))]
    Category(#[br(args { bytes_left })] Utf16StringObj),

    // 12‒25 -------------------------------------------------------------
    #[br(pre_assert(id == 12))]
    Composer(#[br(args { bytes_left })] Utf16StringObj),
    #[br(pre_assert(id == 13))]
    Grouping(#[br(args { bytes_left })] Utf16StringObj),
    #[br(pre_assert(id == 14))]
    Description(#[br(args { bytes_left })] Utf16StringObj),
    #[br(pre_assert(id == 15))]
    PodcastEnclosureUrl(#[br(args { bytes_left })] PodcastUrlObj),
    #[br(pre_assert(id == 16))]
    PodcastRssUrl(#[br(args { bytes_left })] PodcastUrlObj),
    #[br(pre_assert(id == 17))]
    ChapterData(#[br(args { bytes_left })] Unimplemented),
    #[br(pre_assert(id == 18))]
    Subtitle(#[br(args { bytes_left })] Unimplemented),
    #[br(pre_assert(id == 19))]
    Show(#[br(args { bytes_left })] Unimplemented),
    #[br(pre_assert(id == 20))]
    EpisodeNumber(#[br(args { bytes_left })] Unimplemented),
    #[br(pre_assert(id == 21))]
    TvNetwork(#[br(args { bytes_left })] Unimplemented),
    #[br(pre_assert(id == 22))]
    AlbumArtist(#[br(args { bytes_left })] Unimplemented),
    #[br(pre_assert(id == 23))]
    ArtistSort(#[br(args { bytes_left })] Unimplemented),
    #[br(pre_assert(id == 24))]
    Keywords(#[br(args { bytes_left })] Unimplemented),
    #[br(pre_assert(id == 25))]
    TvShowLocale(#[br(args { bytes_left })] Unimplemented),

    // 27‒32 -------------------------------------------------------------
    #[br(pre_assert(id == 27))]
    TitleSort(#[br(args { bytes_left })] Unimplemented),
    #[br(pre_assert(id == 28))]
    AlbumSort(#[br(args { bytes_left })] Unimplemented),
    #[br(pre_assert(id == 29))]
    AlbumArtistSort(#[br(args { bytes_left })] Unimplemented),
    #[br(pre_assert(id == 30))]
    ComposerSort(#[br(args { bytes_left })] Unimplemented),
    #[br(pre_assert(id == 31))]
    TvShowSort(#[br(args { bytes_left })] Unimplemented),
    #[br(pre_assert(id == 32))]
    UnknownVideoBinary(#[br(args { bytes_left })] Unimplemented),

    // 39 ---------------------------------------------------------------
    #[br(pre_assert(id == 39))]
    Copyright(#[br(args { bytes_left })] Unimplemented),

    // 50‒53 -------------------------------------------------------------
    #[br(pre_assert(id == 50))]
    SmartPlaylistData(#[br(args { bytes_left })] Unimplemented),
    #[br(pre_assert(id == 51))]
    SmartPlaylistRules(#[br(args { bytes_left })] Unimplemented),
    #[br(pre_assert(id == 52))]
    LibraryPlaylistIndex(#[br(args { bytes_left })] Unimplemented),
    #[br(pre_assert(id == 53))]
    JumpTable(#[br(args { bytes_left })] Unimplemented),

    // 100 ---------------------------------------------------------------
    #[br(pre_assert(id == 100))]
    ColumnSizingAndOrder(#[br(args { bytes_left })] Unimplemented),
    #[br(pre_assert(id == 102))]
    UnknownObject(#[br(args { bytes_left })] Unimplemented),

    // 200‒204 -----------------------------------------------------------
    #[br(pre_assert(id == 200))]
    AlbumInAlbumList(#[br(args { bytes_left })] Unimplemented),
    #[br(pre_assert(id == 201))]
    ArtistInAlbumList(#[br(args { bytes_left })] Unimplemented),
    #[br(pre_assert(id == 202))]
    ArtistSortInAlbumList(#[br(args { bytes_left })] Unimplemented),
    #[br(pre_assert(id == 203))]
    PodcastUrlInAlbumList(#[br(args { bytes_left })] Unimplemented),
    #[br(pre_assert(id == 204))]
    TvShowInAlbumList(#[br(args { bytes_left })] Unimplemented),
}

impl ObjectType {
    /// Return the numeric `obj_type` tag that this variant represents.
    pub fn as_id(&self) -> u32 {
        match self {
            // 1‒9
            ObjectType::Title(_) => 1,
            ObjectType::Location(_) => 2,
            ObjectType::Album(_) => 3,
            ObjectType::Artist(_) => 4,
            ObjectType::Genre(_) => 5,
            ObjectType::Filetype(_) => 6,
            ObjectType::EqSetting(_) => 7,
            ObjectType::Comment(_) => 8,
            ObjectType::Category(_) => 9,

            // 12‒25
            ObjectType::Composer(_) => 12,
            ObjectType::Grouping(_) => 13,
            ObjectType::Description(_) => 14,
            ObjectType::PodcastEnclosureUrl(_) => 15,
            ObjectType::PodcastRssUrl(_) => 16,
            ObjectType::ChapterData(_) => 17,
            ObjectType::Subtitle(_) => 18,
            ObjectType::Show(_) => 19,
            ObjectType::EpisodeNumber(_) => 20,
            ObjectType::TvNetwork(_) => 21,
            ObjectType::AlbumArtist(_) => 22,
            ObjectType::ArtistSort(_) => 23,
            ObjectType::Keywords(_) => 24,
            ObjectType::TvShowLocale(_) => 25,

            // 27‒32
            ObjectType::TitleSort(_) => 27,
            ObjectType::AlbumSort(_) => 28,
            ObjectType::AlbumArtistSort(_) => 29,
            ObjectType::ComposerSort(_) => 30,
            ObjectType::TvShowSort(_) => 31,
            ObjectType::UnknownVideoBinary(_) => 32,

            // 39
            ObjectType::Copyright(_) => 39,

            // 50‒53
            ObjectType::SmartPlaylistData(_) => 50,
            ObjectType::SmartPlaylistRules(_) => 51,
            ObjectType::LibraryPlaylistIndex(_) => 52,
            ObjectType::JumpTable(_) => 53,

            // 100
            ObjectType::ColumnSizingAndOrder(_) => 100,
            ObjectType::UnknownObject(_) => 102,

            // 200‒204
            ObjectType::AlbumInAlbumList(_) => 200,
            ObjectType::ArtistInAlbumList(_) => 201,
            ObjectType::ArtistSortInAlbumList(_) => 202,
            ObjectType::PodcastUrlInAlbumList(_) => 203,
            ObjectType::TvShowInAlbumList(_) => 204,
        }
    }
}

#[binrw]
#[brw(little, magic = b"mhod")]
#[derive(Debug)]
pub(crate) struct DataObject {
    header_len: u32,
    len: u32,

    #[bw(calc = object.as_id())]
    obj_type: u32,

    #[br(args { id: obj_type, bytes_left: len - 16 })]
    pub object: ObjectType,
}

#[binrw]
#[br(import { bytes_left: u32 = 0 })]
#[derive(Debug)]
pub(crate) struct Utf16StringObj {
    unk_0x00: u32,
    unk_0x04: u32,
    position: u32,

    #[bw(calc = string_data.len() as u32)]
    length: u32,

    unk_0x0C: u32,
    unk_0x10: u32,

    #[br(count = length)]
    string_data: Vec<u8>,
}

impl Utf16StringObj {
    pub fn to_string(&self) -> Result<String, std::string::FromUtf16Error> {
        assert!(self.string_data.len() % 2 == 0, "Malformed utf16!");

        let words: &[u16] =
            try_cast_slice(&self.string_data).expect("Failed to cast byte array to u16 array");

        String::from_utf16(&words)
    }
}

#[binrw]
#[br(import { bytes_left: u32 = 0 })]
#[derive(Debug)]
pub(crate) struct PodcastUrlObj {
    unk_0x00: u32,
    unk_0x04: u32,

    #[br(count = bytes_left - 8)]
    string_data: Vec<u8>,
}

impl PodcastUrlObj {
    pub fn to_string(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.string_data)
    }
}

#[binrw]
#[br(import { bytes_left: u32 = 0 })]
#[derive(Debug)]
pub(crate) struct Unimplemented {
    #[br(count = bytes_left)]
    data: Vec<u8>,
}

