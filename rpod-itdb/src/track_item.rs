#![allow(non_snake_case, unused)]

use std::path::{Component, Path};

use binrw::binrw;
use rand::random;
use rpod_meta::{
    Manifest,
    descriptive::Tags,
    technical::{CodecInfo, FormatInfo},
};
use symphonia::core::codecs;

use crate::{
    DataObjectContainer, DataType, MediaType, SizeRange, TRACK_ITEM_MAX_SIZE, TRACK_ITEM_MIN_SIZE,
    data_object::DataObject, hfs,
};

#[binrw]
#[brw(little, magic = b"mhit")]
#[derive(Debug, Clone, PartialEq)]
pub struct TrackItem {
    #[bw(calc = self.header_bytes_len())]
    header_len: u32,

    #[bw(calc = self.total_bytes_len())]
    len: u32,

    #[bw(calc = self.data_objects.len() as u32)]
    data_object_count: u32,

    pub id: u32, // unique id for the track, used by playlists
    pub visible: u32,
    pub file_type_fourcc: [u8; 4], // looks big endian. file extension padded with spaces (ie: ' 3PM')
    pub mp3_vbr_flag: u8,
    pub mp3_flag: u8,
    pub compilation_flag: u8,
    pub rating: u8,
    pub hfs_time_last_modified: u32, // all timestamps are likely in apples hfs+ format
    pub file_size_bytes_u32: u32,
    pub duration_ms: u32,
    pub album_index: u32,
    pub album_track_count: u32,
    pub release_year: u32,
    pub bitrate: u32,
    pub sample_rate: u32, // number stored here is sample rate of file mult by 0x10000
    pub playback_volume_adj: u32, // can be anything from -255 to 255, adjust volume on playback (replaygain??) set in itunes
    pub start_offset_ms: u32,
    pub stop_offset_ms: u32,
    pub soundcheck: u32, // works with replay gain, soundcheck = 1000 * 10^(-.1 * y) where y is adjustment in dB
    pub play_count_1: u32,
    pub play_count_2: u32, // need to see if this is ever different from above play count
    pub hfs_time_last_played: u32,
    pub album_disc_index: u32,
    pub album_disc_count: u32,
    pub drm_user_id: u32, // most likely just needs to be zero unless the user has drm protected files
    pub hfs_time_date_added: u32,
    pub bookmark_ms: u32, // used for .aa and .m4b files, ipod might actually set this in play counts file instead
    pub uid: u64,         // id used to link across database files (eg mhit -> mhii in artworkdb)
    pub unchecked_flag: u8, // unchecked in itunes true/false
    pub last_rating: u8,  // rating from itunes before sync goes here for some reason
    pub bpm: u16,
    pub artwork_count: u16,
    pub audio_format_tag: u16, // 0xFFFF for mp3/aac, 0x0 for uncompressed (wav), 0x1 for audible, see unk_0x90
    pub artwork_size_bytes: u32, // size of artwork (likely in metadata of audio file?)
    pub unk_0x84: u32,         // always seems to be 0
    pub IEEE_f32_sample_rate: f32, // sample rate as IEEE f32?
    pub hfs_time_release_date: u32,
    pub unk_0x90: u16, // encoding-related info? if above is 0xFFFF, this is 60
    pub explicit_flag: u16,
    pub unk_0x94: u32, // 0x01010100 if has apple drm?
    pub unk_0x98: u32,
    pub skip_count: u32,
    pub hfs_time_last_skipped: u32,
    pub has_artwork: u8, // 0x02 for tracks without artwork, 0x01 for tracks with artwork
    pub skip_on_shuffle_flag: u8, // recommended set to true for intro tracks/podcasts, always set with remember_playback_pos
    pub remember_playback_position_flag: u8, // set to true for files that aren't audiobooks to enable bookmark field
    pub podcast_flag: u8, // 0x1 won't show artist name, if podcast must be set to 0x01 or 0x02
    pub uid_copy: u64,    // copy of the persistent id
    pub has_lyrics_flag: u8,
    pub is_movie_flag: u8,
    pub podcast_unplayed: u8, // 0x01 for non podcasts, 0x02 marks podcasts with a bullet (not played)
    pub unk_0xB3: u8,         // seems to be always 0
    pub bookmark_ms_copy: u32, // seems to be always 0
    pub samples_before_start_gapless: u32,
    pub sample_count: u64,
    pub unk_0xC4: u32, // seems to be always 0
    pub samples_before_end_gapless: u32,
    pub unk_0xCC: u32, // seems to be always 1 for non audiobooks, some kind of number for audiobooks
    pub media_type: MediaType, // VERY IMPORTANT, denotes media type
    pub season_number: u32, // for tv shows only
    pub episode_number: u32, // for tv shows only
    pub unk_0xDC: u32, // seems to be 0x01 for protected files?

    // In samples, these are copied to unk_0x01B0..01D8
    pub unk_0xE0: u32,
    pub unk_0xE4: u32,
    pub unk_0xE8: u32,
    pub unk_0xEC: u32,
    pub unk_0xF0: u32,
    pub unk_0xF4: u32,

    pub end_prefetch_bytes: u32, // size in bytes from first synch frame, can be 0 for AAC
    pub unk_0xFC: u32,
    pub is_gapless_track_flag: u16,
    pub is_gapless_album_flag: u16,
    pub unk_0x0104: u32,
    pub unk_0x0108: u32,
    pub unk_0x010C: u32,
    pub unk_0x0110: u32,
    pub unk_0x0114: u32,
    pub unk_0x0118: u64,
    pub album_id: u32,
    pub database_id: u64, // seems to be set to across track_items in database, possibly an id
    pub file_size_bytes_u64: u64, // seems to be the size of the track in bytes again, possibly as u64
    pub unk_0x0134: u64,          // seems to be set to 0x80808080_80800000
    pub unk_0x013C: u32,          // 4
    pub unk_0x0140: u32,          // 4
    pub unk_0x0144: u32,          // 4
    pub unk_0x0148: u16,          // 2  ==> total 14 bytes
    pub unk_0x014B: u32,
    pub unk_0x014F: u32,
    pub unk_0x0153: u32,
    pub unk_0x0157: u32,
    pub unk_0x015B: u32,
    pub unk_0x015F: u16, // tossed a u16 in here
    pub mhii_link: u32,  // need more research
    pub unk_0x0164: u32,
    pub unk_0x0168: u32, // seems to always be 32
    pub unk_0x016C: u32,
    pub unk_0x0170: u32,
    pub unk_0x0174: u32,
    pub unk_0x0178: u32,
    pub unk_0x017C: u32,
    pub unk_0x0180: u32,
    pub unk_0x0184: u32,
    pub unk_0x0188: u32,
    pub unk_0x018C: u32,
    pub unk_0x0190: u32, // 9 * 4 = 36 bytes
    pub unk_0x0194: u32, // set to 1 on a few podcasts
    pub unk_0x0198: u32,
    pub unk_0x019C: u32,
    pub unk_0x01A0: u32,
    pub unk_0x01A4: u32,
    pub unk_0x01A8: u32,
    pub unk_0x01AC: u32,

    // In samples, these are copied to unk_0x01B0..01D8
    pub unk_0x01B0: u32,
    pub unk_0x01B4: u32,
    pub unk_0x01B8: u32,
    pub unk_0x01BC: u32,
    pub unk_0x01C0: u32,
    pub unk_0x01C4: u32,
    pub unk_0x01C8: u32,
    pub unk_0x01CC: u32,
    pub unk_0x01D0: u32,
    pub unk_0x01D4: u32,
    pub unk_0x01D8: u32,

    pub unk_0x01DC: u32,
    pub artist_id: u32, // some kind of u32 id, same for all track items in db, libgpod thinks artist though
    pub unk_0x01E4: u32,
    pub unk_0x01E8: u32,
    pub unk_0x01EC: u32,
    pub unk_0x01F0: u32,
    pub composer_id: u32, // another id, libgpod thinks its composer
    pub unk_0x01F8: u32,
    pub unk_0x01FC: u32,
    pub unk_0x0200: u32,
    pub unk_0x0204: u32,
    pub unk_0x0208: u32,
    pub unk_0x020C: u32, // observed to be 0x0000_0002 or 0x0000_0001 on some podcasts and audiobooks?
    pub unk_0x0210: u32,
    pub unk_0x0214: u32,
    pub unk_0x0218: u32,
    pub unk_0x021C: u32,
    pub unk_0x0220: u32,
    pub unk_0x0224: u32,
    pub unk_0x0228: u32,
    pub unk_0x022C: u32, // rogue 0x0100_0000, bitfield? seems to be set on some podcasts

    #[br(count = header_len - Self::MIN_SIZE)]
    padding: Vec<u8>,

    #[br(count = data_object_count)]
    pub data_objects: Vec<DataObject>,
}

impl Default for TrackItem {
    fn default() -> Self {
        Self {
            id: 0,
            visible: 1,                  // show up by default
            file_type_fourcc: [b' '; 4], // space-padded FOURCC
            mp3_vbr_flag: 0,
            mp3_flag: 0,
            compilation_flag: 0,
            rating: 0,
            hfs_time_last_modified: 0,
            file_size_bytes_u32: 0,
            duration_ms: 0,
            album_index: 0,
            album_track_count: 0,
            release_year: 0,
            bitrate: 0,
            sample_rate: 0,
            playback_volume_adj: 0,
            start_offset_ms: 0,
            stop_offset_ms: 0,
            soundcheck: 0,
            play_count_1: 0,
            play_count_2: 0,
            hfs_time_last_played: 0,
            album_disc_index: 0,
            album_disc_count: 0,
            drm_user_id: 0,
            hfs_time_date_added: 0,
            bookmark_ms: 0,
            uid: 0,
            unchecked_flag: 0, // checked
            last_rating: 0,
            bpm: 0,
            artwork_count: 0,
            audio_format_tag: 0, // set to 0xFFFF later for mp3/aac if needed
            artwork_size_bytes: 0,
            unk_0x84: 0,
            IEEE_f32_sample_rate: 0.0,
            hfs_time_release_date: 0,

            // meaning unknown, mp3 always 0x0000000c, (flip 0x00 to 0x01 if played in itunes)
            // aac always 0x01000033, audible files 0x01000029, wav 0x0
            unk_0x90: 0,

            explicit_flag: 0,
            unk_0x94: 0,
            unk_0x98: 0,
            skip_count: 0,
            hfs_time_last_skipped: 0,
            has_artwork: 0x02, // 0x02 = no artwork
            skip_on_shuffle_flag: 0,
            remember_playback_position_flag: 0,
            podcast_flag: 0,
            uid_copy: 0,
            has_lyrics_flag: 0,
            is_movie_flag: 0,
            podcast_unplayed: 0x01, // non-podcast
            unk_0xB3: 0,
            bookmark_ms_copy: 0,
            samples_before_start_gapless: 0,
            sample_count: 0,
            unk_0xC4: 0,
            samples_before_end_gapless: 0,
            unk_0xCC: 1,
            media_type: MediaType::Unknown,
            season_number: 0,
            episode_number: 0,
            unk_0xDC: 0,
            unk_0xE0: 0,
            unk_0xE4: 0,
            unk_0xE8: 0,
            unk_0xEC: 0,
            unk_0xF0: 0,
            unk_0xF4: 0,
            end_prefetch_bytes: 0,
            unk_0xFC: 0,
            is_gapless_track_flag: 1,
            is_gapless_album_flag: 0,
            unk_0x0104: 0,
            unk_0x0108: 0,
            unk_0x010C: 0,
            unk_0x0110: 0,
            unk_0x0114: 0,
            unk_0x0118: 0,
            album_id: 0,
            database_id: 0,
            file_size_bytes_u64: 0,
            unk_0x0134: 0x0000_8080_8080_8080,

            // expanded former 14-byte pad
            unk_0x013C: 0,
            unk_0x0140: 0,
            unk_0x0144: 0,
            unk_0x0148: 0,
            unk_0x014B: 0,
            unk_0x014F: 0,
            unk_0x0153: 0,
            unk_0x0157: 0, // podcasty field
            unk_0x015B: 0,
            unk_0x015F: 0,
            mhii_link: 0,
            unk_0x0164: 0,
            unk_0x0168: 0x0000_0020,
            unk_0x016C: 0,

            // expanded former 36-byte pad (9 * u32)
            unk_0x0170: 0,
            unk_0x0174: 0,
            unk_0x0178: 0,
            unk_0x017C: 0,
            unk_0x0180: 0,
            unk_0x0184: 0,
            unk_0x0188: 0,
            unk_0x018C: 0,
            unk_0x0190: 0,

            // subsequent unknowns
            unk_0x0194: 0, // podcasty field
            unk_0x0198: 0,
            unk_0x019C: 0,
            unk_0x01A0: 0,
            unk_0x01A4: 0,
            unk_0x01A8: 0,
            unk_0x01AC: 0,
            unk_0x01B0: 0,
            unk_0x01B4: 0,
            unk_0x01B8: 0,
            unk_0x01BC: 0,
            unk_0x01C0: 0,
            unk_0x01C4: 0,
            unk_0x01C8: 0,
            unk_0x01CC: 0,
            unk_0x01D0: 0,
            unk_0x01D4: 0,
            unk_0x01D8: 0,
            unk_0x01DC: 0,
            artist_id: 0,
            unk_0x01E4: 0,
            unk_0x01E8: 0,
            unk_0x01EC: 0,
            unk_0x01F0: 0,
            composer_id: 0,
            unk_0x01F8: 0,
            unk_0x01FC: 0,
            unk_0x0200: 0,
            unk_0x0204: 0,
            unk_0x0208: 0,
            unk_0x020C: 0,
            unk_0x0210: 0,
            unk_0x0214: 0,
            unk_0x0218: 0,
            unk_0x021C: 0,
            unk_0x0220: 0,
            unk_0x0224: 0,
            unk_0x0228: 0,
            unk_0x022C: 0,
            padding: vec![0u8; (Self::MAX_SIZE - Self::MIN_SIZE) as usize],
            data_objects: Vec::new(),
        }
    }
}

impl SizeRange for TrackItem {
    const MIN_SIZE: u32 = TRACK_ITEM_MIN_SIZE;
    const MAX_SIZE: u32 = TRACK_ITEM_MAX_SIZE;

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

impl DataObjectContainer for TrackItem {
    fn data_objects(&self) -> &[DataObject] {
        &self.data_objects
    }

    fn data_objects_mut(&mut self) -> &mut Vec<DataObject> {
        &mut self.data_objects
    }
}

impl TrackItem {
    pub fn from_manifest(manifest: &Manifest) -> Self {
        let mut track_item = TrackItem::default();

        track_item.file_size_bytes_u32 = manifest.size as u32;
        track_item.file_size_bytes_u64 = manifest.size;
        track_item.hfs_time_last_modified = u32::from(hfs::Timestamp::from(manifest.last_modified));
        track_item.hfs_time_date_added = u32::from(hfs::Timestamp::now());

        track_item
            .generate_uid()
            .set_tags(&manifest.tags)
            .set_file_ext(&manifest.ext)
            .set_format(&manifest.format_info)
            .set_location(&manifest.path);
        track_item
    }

    pub fn generate_uid(&mut self) -> &mut Self {
        self.uid = random::<u64>();
        self.uid_copy = self.uid;
        self
    }

    pub fn set_file_ext(&mut self, file_ext: &str) -> &mut Self {
        let mut fourcc = [b' '; 4];

        for (index, byte) in file_ext.as_bytes().iter().enumerate() {
            if index < 4 {
                fourcc[index] = byte.to_ascii_uppercase();
            }
        }
        fourcc.reverse();

        self.file_type_fourcc = fourcc;
        self
    }

    pub fn set_media_type(&mut self, media_type: MediaType) -> &mut Self {
        match media_type {
            MediaType::Podcast => {
                self.podcast_flag = 0x01;
                self.podcast_unplayed = 0x02;
                self.skip_on_shuffle_flag = 0x01;
                self.remember_playback_position_flag = 0x01;
                self.unk_0x020C = 0x00000002;
            }
            MediaType::Audiobook => {
                self.skip_on_shuffle_flag = 0x01;
                self.remember_playback_position_flag = 0x01;
                self.unk_0x020C = 0x00000002;
            }
            _ => {}
        }
        self.media_type = media_type;
        self
    }

    pub fn set_format(&mut self, format_info: &FormatInfo) -> &mut Self {
        self.sample_rate = format_info.sample_rate << 16;
        self.IEEE_f32_sample_rate = format_info.sample_rate as f32;
        self.bitrate = format_info.avg_kbps;
        self.duration_ms = format_info.duration_ms as u32;
        self.sample_count = format_info.frame_count;
        self.samples_before_start_gapless = format_info.delay_frames;
        self.samples_before_end_gapless = format_info.padding_frames;

        match &format_info.codec_info {
            CodecInfo::Mp3(mp3) => {
                self.mp3_flag = 1;
                self.mp3_vbr_flag = mp3.vbr as u8;
                self.unk_0xCC = 1;
                self.audio_format_tag = 0xFFFF;
                self.unk_0x90 = 0x000C;
                self.end_prefetch_bytes = mp3.end_prefetch_span_bytes;
            }
            CodecInfo::Alac(_) | CodecInfo::Aac => {
                self.audio_format_tag = 0xFFFF;
                self.unk_0x90 = 0x0033;
            }
            _ => {}
        }
        _ = self.upsert_string(
            DataType::FileDescriptor,
            format_info.codec_info.codec_description(),
        );
        self
    }

    pub fn set_location(&mut self, location: &Path) -> &mut Self {
        let mut out = String::with_capacity(64);

        for component in location.components().filter_map(|component| {
            if let Component::Normal(part) = component {
                Some(part)
            } else {
                None
            }
        }) {
            out.push(':');
            out.push_str(&component.to_string_lossy());
        }

        _ = self.upsert_string(DataType::Location, &out);
        self
    }

    pub fn set_tags(&mut self, tags: &Tags) -> &mut Self {
        if let Some(value) = &tags.year {
            self.release_year = *value;
        }

        if let Some(value) = &tags.track_no {
            self.album_index = *value;
        }

        if let Some(value) = &tags.track_total {
            self.album_track_count = *value;
        }

        if let Some(value) = &tags.disc_no {
            self.album_disc_index = *value;
        }

        if let Some(value) = &tags.disc_total {
            self.album_disc_count = *value;
        }

        if let Some(value) = &tags.title {
            let _ = self.upsert_string(DataType::Title, value);
        }

        if let Some(value) = &tags.album {
            let _ = self.upsert_string(DataType::Album, value);
        }

        if let Some(value) = &tags.artist {
            let _ = self.upsert_string(DataType::Artist, value);
        }

        if let Some(value) = &tags.album_artist {
            let _ = self.upsert_string(DataType::AlbumArtist, value);
        }

        if let Some(value) = &tags.composer {
            let _ = self.upsert_string(DataType::Composer, value);
        }

        if let Some(value) = &tags.genre {
            let _ = self.upsert_string(DataType::Genre, value);
        }

        if let Some(value) = &tags.comment {
            let _ = self.upsert_string(DataType::Comment, value);
        }

        if let Some(value) = &tags.copyright {
            let _ = self.upsert_string(DataType::Copyright, value);
        }

        if let Some(value) = &tags.title_sort {
            let _ = self.upsert_string(DataType::TitleSort, value);
        }

        if let Some(value) = &tags.album_sort {
            let _ = self.upsert_string(DataType::AlbumSort, value);
        }

        if let Some(value) = &tags.artist_sort {
            let _ = self.upsert_string(DataType::ArtistSort, value);
        }

        if let Some(value) = &tags.album_artist_sort {
            let _ = self.upsert_string(DataType::AlbumArtistSort, value);
        }

        if let Some(value) = &tags.composer_sort {
            let _ = self.upsert_string(DataType::ComposerSort, value);
        }

        if let Some(value) = &tags.podcast_url {
            let _ = self.upsert_string(DataType::PodcastRssUrl, value);

            if self.media_type != MediaType::Podcast {
                self.set_media_type(MediaType::Podcast);
            }
        }

        if let Some(value) = &tags.podcast_desc {
            let _ = self.upsert_string(DataType::Description, value);

            if self.media_type != MediaType::Podcast {
                self.set_media_type(MediaType::Podcast);
            }
        }

        if tags.podcast_flag {
            if self.media_type != MediaType::Podcast {
                self.set_media_type(MediaType::Podcast);
            }
        }
        self
    }

    pub fn file_type_normalized(&self) -> String {
        let mut fourcc = self.file_type_fourcc;
        fourcc.reverse();

        str::from_utf8(&fourcc)
            .unwrap_or("")
            .trim_end_matches(|end: char| end == '\0' || end.is_ascii_whitespace())
            .to_ascii_lowercase()
    }
}
