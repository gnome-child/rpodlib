#![allow(non_snake_case, unused)]

use binrw::binrw;
use rand::random;

use crate::{
    ALBUM_ITEM_MAX_SIZE, ALBUM_ITEM_MIN_SIZE, DataObjectContainer, DataType, SizeRange,
    data_object::DataObject,
    error::{Error, Result},
    track_item::TrackItem,
};

#[binrw]
#[brw(magic = b"mhia", little)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlbumItem {
    #[bw(calc = self.header_bytes_len())]
    header_len: u32,

    #[bw(calc = self.total_bytes_len())]
    len: u32,

    #[bw(calc = self.data_objects.len() as u32)]
    data_object_count: u32,

    pub album_id: u32, // for podcasts, an album exists, but on the track items, no track/disc count
    pub sql_id: u64,
    pub unk_0x1C: u32,        // always 2?
    pub first_track_uid: u64, // found in playlist item and track item corresponding

    #[br(count = header_len - Self::MIN_SIZE)]
    padding: Vec<u8>,

    #[br(count = data_object_count)]
    pub data_objects: Vec<DataObject>,
}

impl Default for AlbumItem {
    fn default() -> Self {
        Self {
            album_id: 0,
            sql_id: 0,
            unk_0x1C: 2,
            first_track_uid: 0,
            padding: vec![0u8; (Self::MAX_SIZE - Self::MIN_SIZE) as usize],
            data_objects: Vec::new(),
        }
    }
}

impl DataObjectContainer for AlbumItem {
    fn data_objects(&self) -> &[DataObject] {
        &self.data_objects
    }

    fn data_objects_mut(&mut self) -> &mut Vec<DataObject> {
        &mut self.data_objects
    }
}

impl SizeRange for AlbumItem {
    const MIN_SIZE: u32 = ALBUM_ITEM_MIN_SIZE;
    const MAX_SIZE: u32 = ALBUM_ITEM_MAX_SIZE;

    fn header_bytes_len(&self) -> u32 {
        Self::MIN_SIZE + self.padding.len() as u32
    }

    fn total_bytes_len(&self) -> u32 {
        self.header_bytes_len()
            + self
                .data_objects
                .iter()
                .map(SizeRange::total_bytes_len)
                .sum::<u32>()
    }
}

impl AlbumItem {
    pub fn from_track_item(track_item: &TrackItem) -> Result<Self> {
        let mut album_item = Self {
            album_id: track_item.album_id,
            sql_id: random::<u64>(),
            first_track_uid: track_item.uid,
            ..Self::default()
        };

        if let Some(string) = track_item.get_string(DataType::Album) {
            album_item.upsert_string(DataType::AlbumInAlbumList, string)?;
            if let Some(string) = track_item
                .get_string(DataType::AlbumArtist)
                .or(track_item.get_string(DataType::Artist))
            {
                album_item.upsert_string(DataType::ArtistInAlbumList, string)?;
            }

            if let Some(string) = track_item.get_string(DataType::ArtistSort) {
                album_item.upsert_string(DataType::ArtistSortInAlbumList, string)?;
            }

            if let Some(string) = track_item.get_string(DataType::PodcastRssUrl) {
                album_item.upsert_string(DataType::PodcastUrlInAlbumList, string)?;
            }

            if let Some(string) = track_item.get_string(DataType::TVShowSort) {
                album_item.upsert_string(DataType::TVShowInAlbumList, string)?;
            }
            Ok(album_item)
        } else {
            Err(Error::MissingAlbumTitle)
        }
    }
}
