use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    fs,
    io::Cursor,
    ops::{Deref, DerefMut},
    path::Path,
};

use binrw::{BinWrite, binrw};
use indexmap::IndexMap;
use rand::random;

use crate::{
    album_item::AlbumItem,
    data_object::DataObject,
    error::{Error, Result},
    hash::{Hasher, Seeds},
    playlist::{PlaylistEntry, PlaylistItem},
    root::Root,
    track_item::TrackItem,
};

pub mod album_item;
pub mod data_object;
pub mod error;
pub mod hash;
pub mod hfs;
pub mod list;
pub mod playlist;
pub mod root;
pub mod scratch;
pub mod track_item;

const ROOT_MIN_SIZE: u32 = 168;
const LIST_CONTAINER_MIN_SIZE: u32 = 16;
const LIST_PAYLOAD_MIN_SIZE: u32 = 12;
const TRACK_ITEM_MIN_SIZE: u32 = 560;
const ALBUM_ITEM_MIN_SIZE: u32 = 40;
const PLAYLIST_ITEM_MIN_SIZE: u32 = 92;
const PLAYLIST_ENTRY_MIN_SIZE: u32 = 68;
const DATA_OBJECT_MIN_SIZE: u32 = 16;

const ROOT_MAX_SIZE: u32 = 244;
const LIST_CONTAINER_MAX_SIZE: u32 = 96;
const LIST_PAYLOAD_MAX_SIZE: u32 = 92;
const TRACK_ITEM_MAX_SIZE: u32 = 624;
const ALBUM_ITEM_MAX_SIZE: u32 = 88;
const PLAYLIST_ITEM_MAX_SIZE: u32 = 184;
const PLAYLIST_ENTRY_MAX_SIZE: u32 = 76;
const DATA_OBJECT_MAX_SIZE: u32 = 24;

const UTF_16_STR_MIN_SIZE: u32 = 16;
const LIB_PLAYLIST_INDEX_MIN_SIZE: u32 = 48;

enum PayloadType {
    String(TextEncoding),
    PlaylistIndex,
    JumpTable,
    Type100,
    Unimplemented,
}

enum TextEncoding {
    Utf8,
    Utf16LE,
}

#[binrw]
#[brw(little, repr = u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ListType {
    TrackItems = 1,
    LibraryPlaylists = 2,
    PodcastFmtLibPlaylists = 3,
    AlbumItems = 4,
    SpecialPlaylists = 5,
}

#[binrw]
#[brw(little, repr = u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataType {
    Title = 1,
    Location = 2,
    Album = 3,
    Artist = 4,
    Genre = 5,
    FileDescriptor = 6,
    EQSetting = 7,
    Comment = 8,
    Category = 9,
    Composer = 12,
    Grouping = 13,
    Description = 14,
    PodcastEnclosureUrl = 15,
    PodcastRssUrl = 16,
    ChapterData = 17,
    Subtitle = 18,
    Show = 19,
    EpisodeNumber = 20,
    TVNetwork = 21,
    AlbumArtist = 22,
    ArtistSort = 23,
    Keywords = 24,
    TVShowLocale = 25,
    PlistMetadata = 26,
    TitleSort = 27,
    AlbumSort = 28,
    AlbumArtistSort = 29,
    ComposerSort = 30,
    TVShowSort = 31,
    UnknownVideoBinary = 32,
    LabelISRC = 37,
    Copyright = 39,
    SmartPlaylistData = 50,
    SmartPlaylistRules = 51,
    LibraryPlaylistIndex = 52,
    JumpTable = 53,
    Type100 = 100,
    UnknownObject = 102,
    AlbumInAlbumList = 200,
    ArtistInAlbumList = 201,
    ArtistSortInAlbumList = 202,
    PodcastUrlInAlbumList = 203,
    TVShowInAlbumList = 204,
}

impl DataType {
    const fn payload_type(self) -> PayloadType {
        match self {
            DataType::Title
            | DataType::Location
            | DataType::Album
            | DataType::Artist
            | DataType::Genre
            | DataType::FileDescriptor
            | DataType::EQSetting
            | DataType::Comment
            | DataType::Category
            | DataType::Composer
            | DataType::Grouping
            | DataType::Description
            | DataType::Subtitle
            | DataType::Show
            | DataType::TVNetwork
            | DataType::AlbumArtist
            | DataType::ArtistSort
            | DataType::Keywords
            | DataType::TVShowLocale
            | DataType::TitleSort
            | DataType::AlbumSort
            | DataType::AlbumArtistSort
            | DataType::ComposerSort
            | DataType::TVShowSort
            | DataType::LabelISRC
            | DataType::Copyright
            | DataType::AlbumInAlbumList
            | DataType::ArtistInAlbumList
            | DataType::ArtistSortInAlbumList
            | DataType::PodcastUrlInAlbumList
            | DataType::TVShowInAlbumList => PayloadType::String(TextEncoding::Utf16LE),

            DataType::PodcastEnclosureUrl | DataType::PodcastRssUrl => {
                PayloadType::String(TextEncoding::Utf8)
            }

            DataType::JumpTable => PayloadType::JumpTable,
            DataType::LibraryPlaylistIndex => PayloadType::PlaylistIndex,
            DataType::Type100 => PayloadType::Type100,

            DataType::ChapterData
            | DataType::UnknownObject
            | DataType::SmartPlaylistData
            | DataType::SmartPlaylistRules
            | DataType::UnknownVideoBinary
            | DataType::PlistMetadata
            | DataType::EpisodeNumber => PayloadType::Unimplemented,
        }
    }

    pub const fn is_utf16(self) -> bool {
        matches!(
            self.payload_type(),
            PayloadType::String(TextEncoding::Utf16LE)
        )
    }

    pub const fn is_utf8(self) -> bool {
        matches!(self.payload_type(), PayloadType::String(TextEncoding::Utf8))
    }

    pub const fn is_playlist_index(self) -> bool {
        matches!(self.payload_type(), PayloadType::PlaylistIndex)
    }

    pub const fn is_jump_table(self) -> bool {
        matches!(self.payload_type(), PayloadType::JumpTable)
    }

    pub const fn is_type_100(self) -> bool {
        matches!(self.payload_type(), PayloadType::Type100)
    }

    pub const fn is_unimplemented(self) -> bool {
        matches!(self.payload_type(), PayloadType::Unimplemented)
    }
}

#[binrw]
#[brw(little, repr = u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MediaType {
    Unknown = 0,
    Audio = 1,
    Video = 2,
    Podcast = 4,
    VideoPodcast = 6,
    Audiobook = 8,
    MusicVideo = 20,
    TVShow = 40,
    MusicTV = 60,
}

impl MediaType {
    pub fn is_audio(self) -> bool {
        matches!(self, Self::Audio | Self::Podcast | Self::Audiobook)
    }

    pub fn is_music(self) -> bool {
        matches!(self, Self::Audio)
    }

    pub fn is_podcast(self) -> bool {
        matches!(self, Self::Podcast | Self::VideoPodcast)
    }

    pub fn is_audiobook(self) -> bool {
        matches!(self, Self::Audiobook)
    }

    pub fn is_video(self) -> bool {
        matches!(
            self,
            Self::Video | Self::VideoPodcast | Self::MusicVideo | Self::TVShow | Self::MusicTV
        )
    }
}

#[binrw]
#[brw(little, repr = u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LibraryIndexType {
    Title = 0x03,
    Album = 0x04,
    Artist = 0x05,
    Genre = 0x07,
    Composer = 0x12,
    Show = 0x1D,
    Season = 0x1E,
    Episode = 0x1F,
    Unknown35 = 0x23,
    Unknown36 = 0x24,
}

impl LibraryIndexType {
    pub fn pick_string(&self, track_item: &TrackItem) -> Option<String> {
        match self {
            LibraryIndexType::Title => track_item
                .get_string(DataType::Title)
                .map(|cow_str| cow_str.into_owned())
                .and_then(|owned_str| {
                    let trimmed = owned_str.trim().to_string();

                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed)
                    }
                }),

            LibraryIndexType::Album => track_item
                .get_string(DataType::Album)
                .map(|c| c.into_owned())
                .and_then(|s| {
                    let s = s.trim().to_string();
                    if s.is_empty() { None } else { Some(s) }
                }),

            LibraryIndexType::Artist => track_item
                .get_string(DataType::AlbumArtist)
                .or(track_item.get_string(DataType::Artist))
                .map(|c| c.into_owned())
                .and_then(|s| {
                    let s = s.trim().to_string();
                    if s.is_empty() { None } else { Some(s) }
                }),

            LibraryIndexType::Genre => track_item
                .get_string(DataType::Genre)
                .map(|c| c.into_owned())
                .and_then(|s| {
                    let s = s.trim().to_string();
                    if s.is_empty() { None } else { Some(s) }
                }),

            LibraryIndexType::Composer => track_item
                .get_string(DataType::Composer)
                .map(|c| c.into_owned())
                .and_then(|s| {
                    let s = s.trim().to_string();
                    if s.is_empty() { None } else { Some(s) }
                }),

            LibraryIndexType::Show => track_item
                .get_string(DataType::Show)
                .map(|c| c.into_owned())
                .and_then(|s| {
                    let s = s.trim().to_string();
                    if s.is_empty() { None } else { Some(s) }
                }),

            LibraryIndexType::Season => None,

            LibraryIndexType::Episode => track_item
                .get_string(DataType::Subtitle)
                .map(|c| c.into_owned())
                .and_then(|s| {
                    let s = s.trim().to_string();
                    if s.is_empty() { None } else { Some(s) }
                }),

            LibraryIndexType::Unknown35 | LibraryIndexType::Unknown36 => None,
        }
    }
}

pub trait SizeRange {
    const MIN_SIZE: u32;
    const MAX_SIZE: u32;

    fn header_bytes_len(&self) -> u32;

    fn total_bytes_len(&self) -> u32 {
        self.header_bytes_len()
    }
}

pub trait DataObjectContainer {
    fn data_objects(&self) -> &[DataObject];
    fn data_objects_mut(&mut self) -> &mut Vec<DataObject>;

    fn contains_string(&self, needle: &str) -> bool {
        self.data_objects().iter().any(|obj| {
            obj.payload
                .as_str_lossy()
                .map(|s| s.contains(needle))
                .unwrap_or(false)
        })
    }

    fn get_string(&self, data_type: DataType) -> Option<Cow<'_, str>> {
        self.data_objects()
            .iter()
            .find(|obj| obj.data_type == data_type)
            .and_then(|obj| obj.payload.as_str_lossy())
    }

    fn get_or_default_string(&mut self, data_type: DataType) -> Result<String> {
        if let Some(string) = self
            .get_string(data_type)
            .filter(|str| !str.trim().is_empty())
        {
            return Ok(string.into_owned());
        } else {
            let default = format!("Unknown {:?}", data_type);

            self.upsert_string(data_type, &default)?;
            Ok(default)
        }
    }

    fn get_or_insert_string(&mut self, data_type: DataType, insert: &str) -> Result<String> {
        if let Some(string) = self
            .get_string(data_type)
            .filter(|str| !str.trim().is_empty())
        {
            return Ok(string.into_owned());
        } else {
            self.upsert_string(data_type, insert)?;
            Ok(insert.to_string())
        }
    }

    fn upsert_data_object(&mut self, data_type: DataType, data_obj: DataObject) {
        let objs = self.data_objects_mut();

        if let Some(pos) = objs.iter().position(|obj| obj.data_type == data_type) {
            let mut seen_first = false;
            objs[pos] = data_obj;

            objs.retain(|obj| {
                if obj.data_type != data_type {
                    return true;
                }

                if !seen_first {
                    seen_first = true;
                    true
                } else {
                    false
                }
            });
        } else {
            objs.push(data_obj);
        }
    }

    /// Replaces/inserts a string into the database record.
    /// Fails if data_type is not a string type.
    fn upsert_string<'a, S: Into<Cow<'a, str>>>(
        &mut self,
        data_type: DataType,
        string: S,
    ) -> Result<&mut Self> {
        let string = string.into();

        let obj = if data_type.is_utf16() {
            DataObject::new_utf16(data_type, &string)
        } else if data_type.is_utf8() {
            DataObject::new_utf8(data_type, &string)
        } else {
            return Err(Error::NotAString { data_type });
        };

        self.upsert_data_object(data_type, obj);
        Ok(self)
    }
}

pub struct Track(TrackItem);

impl From<&TrackItem> for Track {
    fn from(value: &TrackItem) -> Self {
        Track(value.clone())
    }
}

impl From<TrackItem> for Track {
    fn from(value: TrackItem) -> Self {
        Track(value)
    }
}

impl Deref for Track {
    type Target = TrackItem;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl DerefMut for Track {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl AsRef<TrackItem> for Track {
    fn as_ref(&self) -> &TrackItem {
        &self.0
    }
}

impl AsMut<TrackItem> for Track {
    fn as_mut(&mut self) -> &mut TrackItem {
        &mut self.0
    }
}

impl Track {
    /// Consume and return the owned `TrackItem`.
    pub fn into_inner(self) -> TrackItem {
        self.0
    }
}

pub struct Playlist(PlaylistItem);

impl From<&PlaylistItem> for Playlist {
    fn from(value: &PlaylistItem) -> Self {
        Playlist(value.clone())
    }
}

impl From<PlaylistItem> for Playlist {
    fn from(value: PlaylistItem) -> Self {
        Playlist(value)
    }
}

impl Deref for Playlist {
    type Target = PlaylistItem;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl DerefMut for Playlist {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl AsRef<PlaylistItem> for Playlist {
    fn as_ref(&self) -> &PlaylistItem {
        &self.0
    }
}

impl AsMut<PlaylistItem> for Playlist {
    fn as_mut(&mut self) -> &mut PlaylistItem {
        &mut self.0
    }
}

impl Playlist {
    /// Consume and return the owned `PlaylistItem`.
    pub fn into_inner(self) -> PlaylistItem {
        self.0
    }
}

pub struct Podcast {
    header: PlaylistEntry,
    entries: Vec<PlaylistEntry>,
}

impl Podcast {
    pub fn from<'a, I: IntoIterator<Item = &'a PlaylistEntry>>(
        header: &'a PlaylistEntry,
        entries: I,
    ) -> Self {
        Self {
            header: header.clone(),
            entries: entries.into_iter().map(|entry| entry.clone()).collect(),
        }
    }

    pub fn header(&self) -> &PlaylistEntry {
        &self.header
    }

    pub fn header_mut(&mut self) -> &mut PlaylistEntry {
        &mut self.header
    }

    pub fn entries(&self) -> &[PlaylistEntry] {
        &self.entries
    }
}

pub struct Itdb {
    pub version: u32,
    pub id: u64,
    pub tracks: IndexMap<u64, Track>,
    pub playlists: IndexMap<u64, Playlist>,
    pub podcasts: IndexMap<u64, Podcast>,
    pub by_title: HashMap<String, Vec<u64>>,
    pub by_album: HashMap<String, Vec<u64>>,
    pub by_artist: HashMap<String, Vec<u64>>,
    pub by_album_artist: HashMap<String, Vec<u64>>,
    root: Root,
}

impl TryFrom<Root> for Itdb {
    type Error = Error;

    fn try_from(root: Root) -> Result<Self> {
        let version = root.version;
        let id = root.database_uid;
        let track_items = &root
            .track_item_list()
            .ok_or(Error::TrackItemListMissing)?
            .items;
        let playlist_items = &root
            .playlist_item_list()
            .ok_or(Error::PlaylistItemListMissing)?
            .items;
        let podcast_playlist = root
            .podcast_fmt_playlist_list()
            .and_then(|list| {
                list.items
                    .iter()
                    .find(|item| item.is_podcast_playlist_flag != 0)
            })
            .ok_or(Error::PodcastItemListMissing)?;

        let mut tracks = IndexMap::with_capacity(track_items.len());
        track_items.iter().for_each(|item| {
            tracks.insert(item.uid, Track::from(item));
        });

        let mut playlists = IndexMap::with_capacity(playlist_items.len());
        playlist_items.iter().for_each(|item| {
            playlists.insert(item.uid, Playlist::from(item));
        });

        let mut podcast_headers: Vec<&PlaylistEntry> = Vec::new();
        let mut podcast_groups: HashMap<u32, Vec<&PlaylistEntry>> = HashMap::new();
        for entry in &podcast_playlist.entries {
            if entry.podcast_group_flag != 0 {
                podcast_headers.push(entry);
            }
            podcast_groups
                .entry(entry.podcast_group_ref)
                .or_default()
                .push(entry);
        }

        let mut podcasts = IndexMap::with_capacity(podcast_headers.len());
        for header in podcast_headers {
            let entries = podcast_groups
                .remove(&header.podcast_episode_id)
                .unwrap_or_default()
                .into_iter()
                .filter(|entry| entry.podcast_group_flag == 0);

            podcasts.insert(header.podcast_uid, Podcast::from(header, entries));
        }

        let mut by_title: HashMap<String, Vec<u64>> = HashMap::new();
        let mut by_album: HashMap<String, Vec<u64>> = HashMap::new();
        let mut by_artist: HashMap<String, Vec<u64>> = HashMap::new();
        let mut by_album_artist: HashMap<String, Vec<u64>> = HashMap::new();

        for (uid, track) in &tracks {
            if let Some(str) = track.get_string(DataType::Title) {
                by_title.entry(str.to_string()).or_default().push(*uid);
            }
        }

        for (uid, track) in &tracks {
            if let Some(str) = track.get_string(DataType::Album) {
                by_album.entry(str.to_string()).or_default().push(*uid);
            }
        }

        for (uid, track) in &tracks {
            if let Some(str) = track.get_string(DataType::Artist) {
                by_artist.entry(str.to_string()).or_default().push(*uid);
            }
        }

        for (uid, track) in &tracks {
            if let Some(str) = track.get_string(DataType::AlbumArtist) {
                by_album_artist
                    .entry(str.to_string())
                    .or_default()
                    .push(*uid);
            }
        }

        Ok(Self {
            version,
            id,
            root,
            tracks,
            playlists,
            podcasts,
            by_title,
            by_album,
            by_artist,
            by_album_artist,
        })
    }
}

impl Itdb {
    pub fn commit(&mut self) -> Result<()> {
        self.sanitize()?;
        self.reorder_tracks()?;
        self.assign_ids()?;
        self.rebuild_album_list()?;
        self.rebuild_track_list()?;
        self.rebuild_playlists()?;
        self.rebuild_podcasts()?;

        Ok(())
    }

    pub fn write_to<P: AsRef<Path>>(
        &self,
        path: P,
        fw_guid: &[u8; 8],
        hash_seeds: &Seeds,
    ) -> Result<()> {
        let path = path.as_ref();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut cursor = Cursor::new(Vec::with_capacity(self.root.total_bytes_len() as usize));
        self.root.write(&mut cursor)?;

        let mut buf = cursor.into_inner();
        let hasher = Hasher::from_bytes(&mut buf, hash_seeds)?;
        hasher.hash(fw_guid)?;

        fs::write(path, buf)?;
        Ok(())
    }

    pub fn insert_track(&mut self, track: Track) -> Result<()> {
        let mut track_item = track.into_inner();

        if track_item.uid == 0 {
            track_item.uid = random::<u64>();
            track_item.uid_copy = track_item.uid;
        }
        let playlist_item = self.master_pl_mut()?.as_mut();

        playlist_item.push(&track_item);
        self.tracks.insert(track_item.uid, Track::from(track_item));
        Ok(())
    }

    pub fn remove_track(&mut self, track: &Track) -> Result<()> {
        for (_, playlist) in &mut self.playlists {
            playlist.remove(track.uid);
        }
        self.tracks.retain(|uid, _| &track.uid != uid);

        Ok(())
    }

    pub fn sanitize(&mut self) -> Result<()> {
        use std::collections::HashSet;

        let to_remove: HashSet<u64> = self
            .tracks
            .iter()
            .filter(|(_, track)| track.get_string(DataType::Location).is_none())
            .map(|(uid, _)| *uid)
            .collect();

        if to_remove.is_empty() {
            return Ok(());
        }

        for playlist in self.playlists.values_mut() {
            for uid in &to_remove {
                playlist.remove(*uid);
            }
        }

        self.tracks.retain(|uid, _| !to_remove.contains(uid));
        Ok(())
    }

    pub fn as_raw(&self) -> &Root {
        &self.root
    }

    /// 1. Reorder tracks according to master playlist order
    fn reorder_tracks(&mut self) -> Result<()> {
        let mut tracks = IndexMap::with_capacity(self.tracks.len());
        let master_pl = self.master_pl()?;
        let uids: Vec<u64> = master_pl
            .entries
            .iter()
            .map(|entry| entry.track_uid)
            .collect();

        for uid in uids {
            if let Some(track) = self.tracks.shift_remove(&uid) {
                tracks.insert(uid, track);
            }
        }

        for (uid, track) in self.tracks.drain(..) {
            tracks.insert(uid, track);
        }
        self.tracks = tracks;

        Ok(())
    }

    /// 2. Assign incremental ids
    fn assign_ids(&mut self) -> Result<()> {
        let mut next_id = 52;
        let mut album_table = HashMap::new();
        let mut artist_table = HashMap::new();
        let mut composer_table = HashMap::new();

        for track in self.tracks.values_mut() {
            track.id = next_id;
            next_id = next_id.wrapping_add(1);

            let album = track.get_or_default_string(DataType::Album)?;
            let album_id = *album_table.entry(album).or_insert_with(|| {
                let id = next_id;

                next_id = next_id.wrapping_add(1);
                id
            });

            let artist = if let Some(album_artist) = track.get_string(DataType::AlbumArtist) {
                album_artist.into_owned()
            } else {
                track.get_or_default_string(DataType::Artist)?
            };

            let artist_id = *artist_table.entry(artist).or_insert_with(|| {
                let id = next_id;

                next_id = next_id.wrapping_add(1);
                id
            });

            let composer = track.get_or_default_string(DataType::Composer)?;
            let composer_id = *composer_table.entry(composer).or_insert_with(|| {
                let id = next_id;

                next_id = next_id.wrapping_add(1);
                id
            });

            track.album_id = album_id;
            track.artist_id = artist_id;
            track.composer_id = composer_id;
            track.database_id = self.id;
        }

        for entry in self
            .playlists
            .values_mut()
            .flat_map(|playlist| playlist.entries.iter_mut())
        {
            if let Some(track) = self.tracks.get(&entry.track_uid) {
                entry.track_id = track.id;
            }
        }

        for podcast in self.podcasts.values_mut() {
            let group_id = next_id;
            next_id = next_id.wrapping_add(1);

            let header = podcast.header_mut();
            header.podcast_episode_id = group_id;

            for entry in podcast.entries.iter_mut() {
                if let Some(track) = self.tracks.get(&entry.track_uid) {
                    entry.track_id = track.id;
                }

                let episode_id = next_id;
                next_id = next_id.wrapping_add(1);

                entry.podcast_episode_id = episode_id;
                entry.podcast_group_ref = group_id;
                entry.data_objects.clear();
                entry.upsert_data_object(
                    DataType::Type100,
                    DataObject::new_playlist_pos(episode_id),
                );
            }
        }
        Ok(())
    }

    /// 3. Rebuild the album item list
    fn rebuild_album_list(&mut self) -> Result<()> {
        let root = &mut self.root;
        let mut by_id: BTreeMap<u32, &TrackItem> = BTreeMap::new();

        for track in self.tracks.values() {
            by_id
                .entry(track.album_id)
                .or_insert_with(|| track.as_ref());
        }

        let mut album_items = Vec::with_capacity(by_id.len());
        for track_item in by_id.values() {
            album_items.push(AlbumItem::from_track_item(track_item)?);
        }

        root.album_item_list_mut()
            .ok_or(Error::AlbumItemListMissing)?
            .items = album_items;
        Ok(())
    }

    /// 4. Rebuild the track item list, consume the IndexMap
    fn rebuild_track_list(&mut self) -> Result<()> {
        let root = &mut self.root;

        let track_items: Vec<TrackItem> = self
            .tracks
            .drain(..)
            .map(|(_, track)| track.into_inner())
            .collect();

        root.track_item_list_mut()
            .ok_or(Error::TrackItemListMissing)?
            .items = track_items;
        Ok(())
    }

    /// 5. Rebuild the playlist item lists, consume the IndexMap
    fn rebuild_playlists(&mut self) -> Result<()> {
        let root = &mut self.root;

        let items: Vec<PlaylistItem> = self
            .playlists
            .drain(..)
            .map(|(_, playlist)| playlist.into_inner())
            .collect();

        let list = root
            .playlist_item_list_mut()
            .ok_or(Error::PlaylistItemListMissing)?;
        list.items = items.clone();

        let list = root
            .podcast_fmt_playlist_list_mut()
            .ok_or(Error::PlaylistItemListMissing)?;
        list.items = items;

        Ok(())
    }

    /// 6. Rebuild the podcast playlists, consume the IndexMap
    fn rebuild_podcasts(&mut self) -> Result<()> {
        let root = &mut self.root;

        let fmt_cap = self
            .podcasts
            .values()
            .map(|podcast| 1 + podcast.entries.len())
            .sum::<usize>();
        let flat_cap = self
            .podcasts
            .values()
            .map(|podcast| podcast.entries.len())
            .sum::<usize>();

        let mut fmt_entries = Vec::with_capacity(fmt_cap);
        let mut flat_entries = Vec::with_capacity(flat_cap);

        for (_, podcast) in self.podcasts.drain(..) {
            let Podcast { header, entries } = podcast;
            fmt_entries.push(header);

            let entries: Vec<PlaylistEntry> = entries;
            fmt_entries.extend(entries.iter().cloned());
            flat_entries.extend(entries);
        }

        let fmt_list = root
            .podcast_fmt_playlist_list_mut()
            .ok_or(Error::PodcastItemListMissing)?;
        let podcast_fmt_pl = fmt_list
            .items
            .iter_mut()
            .find(|pl| pl.is_podcast_playlist_flag != 0)
            .ok_or(Error::PodcastPlaylistMissing)?;
        podcast_fmt_pl.entries = fmt_entries;

        let playlist_list = root
            .playlist_item_list_mut()
            .ok_or(Error::PlaylistItemListMissing)?;
        let podcast_playlist = playlist_list
            .items
            .iter_mut()
            .find(|pl| pl.is_podcast_playlist_flag != 0)
            .ok_or(Error::PodcastPlaylistMissing)?;
        podcast_playlist.entries = flat_entries;

        Ok(())
    }

    fn master_pl(&self) -> Result<&Playlist> {
        self.playlists
            .values()
            .find(|playlist| playlist.is_master_flag != 0)
            .ok_or(Error::MasterPlaylistMissing)
    }

    fn master_pl_mut(&mut self) -> Result<&mut Playlist> {
        self.playlists
            .values_mut()
            .find(|playlist| playlist.is_master_flag != 0)
            .ok_or(Error::MasterPlaylistMissing)
    }
}
