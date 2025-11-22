use indexmap::IndexMap;

use crate::{
    error::{Error, Result},
    playlist::{PlaylistEntry, PlaylistItem},
    track_item::TrackItem,
};

pub struct Track(TrackItem);

impl From<&TrackItem> for Track {
    fn from(value: &TrackItem) -> Self {
        Self(value.clone())
    }
}

impl Track {
    pub fn into_inner(self) -> TrackItem {
        self.0
    }
}

pub struct Playlist(PlaylistItem);

impl From<&PlaylistItem> for Playlist {
    fn from(value: &PlaylistItem) -> Self {
        Self(value.clone())
    }
}

impl Playlist {
    pub fn into_inner(self) -> PlaylistItem {
        self.0
    }

    pub fn is_master_playlist(&self) -> bool {
        self.0.is_master_flag != 0
    }
}

pub struct Podcast {
    header: PlaylistEntry,
    episodes: Vec<PlaylistEntry>,
}

pub struct Itdb {
    version: u32,
    id: u64,
    tracks: IndexMap<u64, Track>,
    playlists: IndexMap<u64, Playlist>,
    podcasts: IndexMap<u64, Podcast>,
}

impl Itdb {
    fn master_pl(&self) -> Result<&Playlist> {
        self.playlists
            .values()
            .find(|playlist| playlist.is_master_playlist())
            .ok_or(Error::MasterPlaylistMissing)
    }
}
