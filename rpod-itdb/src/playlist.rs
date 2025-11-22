#![allow(non_snake_case, unused)]

use std::{cmp, collections::HashMap};

use binrw::binrw;
use rand::random;

use crate::{
    DataObjectContainer, DataType, LibraryIndexType, PLAYLIST_ENTRY_MAX_SIZE,
    PLAYLIST_ENTRY_MIN_SIZE, PLAYLIST_ITEM_MAX_SIZE, PLAYLIST_ITEM_MIN_SIZE, SizeRange,
    data_object::{DataObject, JumpTableEntry},
    error::Result,
    hfs,
    list::TrackItemList,
    track_item::TrackItem,
};

struct JumpListSeed {
    key: String,
    char: u16,
    track_index: u32,
    track_uid: u64,
}

impl JumpListSeed {
    fn build_seeds(track_items: &[TrackItem], index_type: LibraryIndexType) -> Vec<Self> {
        let mut indices: Vec<Self> = track_items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let key = index_type.pick_string(item).unwrap_or_default();
                let char = key.encode_utf16().next().unwrap_or(0);
                let track_index = index as u32;
                let track_uid = item.uid;

                Self {
                    key,
                    char,
                    track_index,
                    track_uid,
                }
            })
            .collect();

        indices.sort_by(|a, b| {
            a.key
                .cmp(&b.key)
                .then_with(|| a.track_uid.cmp(&b.track_uid))
        });
        indices
    }
}

#[binrw]
#[brw(little, magic = b"mhyp")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaylistItem {
    #[bw(calc = self.header_bytes_len())]
    header_len: u32,

    #[bw(calc = self.total_bytes_len())]
    len: u32,

    #[bw(calc = self.data_objects.len() as u32)]
    data_object_count: u32,

    #[bw(calc = self.entries.len() as u32)]
    entry_count: u32,

    pub is_master_flag: u8, // if true, this is the master playlist, containing all the tracks
    pub flag_0x15: u8,
    pub flag_0x16: u8,
    pub flag_0x17: u8,
    pub hfs_timestamp_created: u32,
    pub uid: u64,
    pub unk_0x24: u32, // always 0?

    #[bw(calc = {
        self.data_objects
            .iter()
            .filter(|obj| obj.data_type.is_utf16() || obj.data_type.is_utf8())
            .count() as u16
    })]
    string_obj_count: u16,

    pub is_podcast_playlist_flag: u16,
    pub sort_order: u32,
    pub unk_0x30: u32,
    pub unk_0x34: u32,
    pub unk_0x38: u32,
    pub database_id: u64, // this is the same as database_id in track items
    pub persistent_id_copy: u64,
    pub unk_0x4C: u32,
    pub unk_0x50: u32,
    pub unk_0x54: u32,
    pub hfs_timestamp_modified: u32,

    #[br(count = header_len - Self::MIN_SIZE)]
    padding: Vec<u8>,

    #[br(count = data_object_count)]
    pub data_objects: Vec<DataObject>,

    #[br(count = entry_count)]
    pub entries: Vec<PlaylistEntry>,
}

impl Default for PlaylistItem {
    fn default() -> Self {
        Self {
            is_master_flag: 0,
            flag_0x15: 0,
            flag_0x16: 0,
            flag_0x17: 0,
            hfs_timestamp_created: 0,
            uid: 0,
            unk_0x24: 0,
            is_podcast_playlist_flag: 0,
            sort_order: 0,
            unk_0x30: 0,
            unk_0x34: 0,
            unk_0x38: 0,
            database_id: 0,
            persistent_id_copy: 0,
            unk_0x4C: 0,
            unk_0x50: 0,
            unk_0x54: 0,
            hfs_timestamp_modified: u32::from(hfs::Timestamp::now()),
            padding: vec![0u8; (Self::MAX_SIZE - Self::MIN_SIZE) as usize],
            data_objects: Vec::new(),
            entries: Vec::new(),
        }
    }
}

impl SizeRange for PlaylistItem {
    const MIN_SIZE: u32 = PLAYLIST_ITEM_MIN_SIZE;
    const MAX_SIZE: u32 = PLAYLIST_ITEM_MAX_SIZE;

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
            + self
                .entries
                .iter()
                .map(SizeRange::total_bytes_len)
                .sum::<u32>()
    }
}

impl DataObjectContainer for PlaylistItem {
    fn data_objects(&self) -> &[DataObject] {
        &self.data_objects
    }

    fn data_objects_mut(&mut self) -> &mut Vec<DataObject> {
        &mut self.data_objects
    }
}

impl PlaylistItem {
    pub fn build_master_playlist(title: &str, track_items: &[TrackItem]) -> Result<Self> {
        let mut playlist = PlaylistItem {
            uid: random::<u64>(),
            is_master_flag: 1,
            hfs_timestamp_created: u32::from(hfs::Timestamp::now()),
            sort_order: 5,
            entries: Vec::with_capacity(track_items.len()),
            ..Self::default()
        };
        track_items.iter().for_each(|item| playlist.push(item));

        playlist.upsert_string(DataType::Title, title)?;
        playlist.upsert_data_object(DataType::Type100, DataObject::new_type_100_blob());
        playlist.rebuild_jumplists(track_items);
        Ok(playlist)
    }

    pub fn push(&mut self, track_item: &TrackItem) {
        let pos = self
            .entries
            .iter()
            .filter_map(|entry| entry.entry_num())
            .max()
            .unwrap_or(0)
            + 1;

        self.entries
            .push(PlaylistEntry::from_track_item(track_item, pos));
    }

    pub fn insert(&mut self, track_item: &TrackItem, pos: usize) {
        let entry_num = self
            .entries
            .iter()
            .filter_map(|entry| entry.entry_num())
            .max()
            .unwrap_or(0)
            + 1;

        self.entries
            .insert(pos, PlaylistEntry::from_track_item(track_item, entry_num));
    }

    pub fn remove(&mut self, track_uid: u64) {
        self.entries.retain(|entry| entry.track_uid != track_uid);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn rebuild_jumplists(&mut self, track_items: &[TrackItem]) {
        self.data_objects
            .retain(|obj| !obj.data_type.is_playlist_index() && !obj.data_type.is_jump_table());

        self.build_jump_list(track_items, LibraryIndexType::Title);
        self.build_jump_list(track_items, LibraryIndexType::Artist);
        self.build_jump_list(track_items, LibraryIndexType::Album);
        self.build_jump_list(track_items, LibraryIndexType::Genre);
        self.build_jump_list(track_items, LibraryIndexType::Composer);
    }

    // FIXME
    fn rebuild_podcast_entries(&mut self, track_items: &[TrackItem], base_group_id: u32) {
        let by_id: HashMap<u32, &TrackItem> =
            track_items.iter().map(|item| (item.id, item)).collect();

        let mut next_group_id = base_group_id;
        let mut next_pos = 100_000;
        let mut album_table = HashMap::new();
        let mut new_entries = Vec::new();

        for mut entry in self.entries.drain(..) {
            entry.data_objects.clear();

            if let Some(item) = by_id.get(&entry.track_id) {
                let album = item
                    .get_string(DataType::Album)
                    .unwrap_or_default()
                    .into_owned();

                let group_id = *album_table.entry(album.clone()).or_insert({
                    let group_id = next_group_id;
                    next_group_id = next_group_id.wrapping_add(1);

                    let mut header = PlaylistEntry {
                        podcast_group_flag: 1,
                        podcast_group_id: group_id,
                        ..PlaylistEntry::default()
                    };
                    _ = header.upsert_string(DataType::Title, album.clone());

                    if let Some(location) = album.strip_prefix("The ") {
                        _ = header.upsert_string(DataType::Location, location);
                    }
                    new_entries.push(header);
                    group_id
                });

                entry.podcast_group_ref = group_id;
                entry.upsert_data_object(DataType::Type100, DataObject::new_playlist_pos(next_pos));

                next_pos = next_pos.wrapping_add(1);
                new_entries.push(entry);
            } else {
                continue;
            }
        }
        self.entries = new_entries
    }

    fn build_jump_list(&mut self, track_items: &[TrackItem], index_type: LibraryIndexType) {
        let seeds = JumpListSeed::build_seeds(track_items, index_type);

        self.build_playlist_index(&seeds, index_type);
        self.build_jump_table(&seeds, index_type);
    }

    fn build_jump_table(&mut self, seeds: &[JumpListSeed], index_type: LibraryIndexType) {
        let mut entries = Vec::new();

        if !seeds.is_empty() {
            let mut current = seeds[0].char;
            let mut start = 0;

            for (index, seed) in seeds.iter().enumerate().skip(1) {
                if seed.char != current {
                    entries.push(JumpTableEntry {
                        letter: current,
                        start_index: start,
                        span: index as u32 - start,
                    });
                    current = seed.char;
                    start = index as u32;
                }
            }

            entries.push(JumpTableEntry {
                letter: current,
                start_index: start,
                span: seeds.len() as u32 - start,
            });
        }
        self.data_objects
            .push(DataObject::new_jump_table(index_type, entries));
    }

    fn build_playlist_index(&mut self, seeds: &[JumpListSeed], index_type: LibraryIndexType) {
        let indices = seeds.iter().map(|row| row.track_index).collect();

        self.data_objects
            .push(DataObject::new_lib_playlist_index(index_type, indices));
    }
}

#[binrw]
#[brw(little, magic = b"mhip")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaylistEntry {
    #[bw(calc = self.header_bytes_len())]
    header_len: u32,

    #[bw(calc = self.total_bytes_len())]
    len: u32,

    #[bw(calc = self.data_objects.len() as u32)]
    data_object_count: u32,

    pub unk_0x16: u8,           // always 0??
    pub podcast_group_flag: u8, // 0x01 in podcast headers
    pub unk_0x18: u8,           // 0x00 or 0x01 in podcast headers
    pub unk_0x19: u8,           // 0x81 or 0x80 in podcast headers
    pub podcast_group_id: u32, // cannot overlap track ids, used in podcast group header as group id, referenced below
    pub track_id: u32,         // from track item
    pub timestamp: u32,        // time added?
    pub podcast_group_ref: u32, // podcast grouping reference
    pub podcast_uid: u64,      // one occurence in the db??
    pub track_uid: u64,
    pub unk_0x34: u32,
    pub unk_0x38: u32,
    pub persistent_id: u64,

    #[br(count = header_len - Self::MIN_SIZE)]
    padding: Vec<u8>,

    #[br(count = data_object_count)]
    pub data_objects: Vec<DataObject>,
}

impl Default for PlaylistEntry {
    fn default() -> Self {
        Self {
            unk_0x16: 0,
            podcast_group_flag: 0,
            unk_0x18: 0,
            unk_0x19: 0,
            podcast_group_id: 0,
            track_id: 0,
            timestamp: 0,
            podcast_group_ref: 0,
            podcast_uid: 0,
            track_uid: 0,
            unk_0x34: 0,
            unk_0x38: 0,
            persistent_id: 0,
            padding: vec![0u8; (Self::MAX_SIZE - Self::MIN_SIZE) as usize],
            data_objects: Vec::new(),
        }
    }
}

impl SizeRange for PlaylistEntry {
    const MIN_SIZE: u32 = PLAYLIST_ENTRY_MIN_SIZE;
    const MAX_SIZE: u32 = PLAYLIST_ENTRY_MAX_SIZE;

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

impl DataObjectContainer for PlaylistEntry {
    fn data_objects(&self) -> &[DataObject] {
        &self.data_objects
    }

    fn data_objects_mut(&mut self) -> &mut Vec<DataObject> {
        &mut self.data_objects
    }
}

impl PlaylistEntry {
    pub fn from_track_item(track_item: &TrackItem, entry_num: u32) -> Self {
        let mut entry = PlaylistEntry {
            timestamp: u32::from(hfs::Timestamp::now()),
            track_id: track_item.id,
            track_uid: track_item.uid,
            ..Self::default()
        };
        entry.upsert_data_object(DataType::Type100, DataObject::new_playlist_pos(entry_num));
        entry
    }

    pub fn entry_num(&self) -> Option<u32> {
        self.data_objects
            .iter()
            .find(|obj| obj.data_type == DataType::Type100)
            .and_then(|obj| obj.payload.as_playlist_pos())
    }
}
