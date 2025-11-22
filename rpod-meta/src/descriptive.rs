use std::path::Path;

use lofty::{
    config::{ParseOptions, ParsingMode, WriteOptions},
    file::{AudioFile, TaggedFile, TaggedFileExt},
    picture::Picture,
    tag::{Accessor, ItemKey, ItemValue, Tag, TagExt, TagItem, TagType},
};

use crate::{ProbeOptions, error::Result};

#[derive(Debug, Clone, Copy)]
pub struct SaveOptions {
    pub clear_missing: bool,
    pub replace_artwork: bool,
}

impl Default for SaveOptions {
    fn default() -> Self {
        Self {
            clear_missing: false,
            replace_artwork: true,
        }
    }
}

#[derive(Debug)]
pub struct Tags {
    pub title: Option<String>,
    pub album: Option<String>,
    pub artist: Option<String>,
    pub album_artist: Option<String>,
    pub composer: Option<String>,
    pub title_sort: Option<String>,
    pub album_sort: Option<String>,
    pub artist_sort: Option<String>,
    pub album_artist_sort: Option<String>,
    pub composer_sort: Option<String>,
    pub track_no: Option<u32>,
    pub track_total: Option<u32>,
    pub disc_no: Option<u32>,
    pub disc_total: Option<u32>,
    pub genre: Option<String>,
    pub year: Option<u32>,
    pub compilation: Option<bool>,
    pub comment: Option<String>,
    pub lyrics: Option<String>,
    pub copyright: Option<String>,
    pub podcast_url: Option<String>,
    pub podcast_desc: Option<String>,
    pub podcast_flag: bool,
    pub compilation_flag: bool,
    pub artwork: Vec<Picture>,
}

impl Tags {
    pub fn probe<P: AsRef<Path>>(path: P, opts: &ProbeOptions) -> Result<Self> {
        use lofty::probe::Probe as LoftyProbe;

        let path = path.as_ref();
        let probe = LoftyProbe::open(path)?.options(
            ParseOptions::default()
                .read_properties(false)
                .read_tags(!opts.ignore_tags)
                .read_cover_art(!opts.ignore_artwork)
                .parsing_mode(ParsingMode::Relaxed),
        );

        let tagged_file = probe.read()?;

        let tags_maybe = tagged_file
            .primary_tag()
            .or_else(|| tagged_file.first_tag());

        let title = tags_maybe
            .and_then(|tags| tags.get_string(&ItemKey::TrackTitle))
            .map(|str| str.to_owned())
            .or_else(|| {
                path.file_stem()
                    .map(|str| str.to_string_lossy().into_owned())
            });

        let artist = tags_maybe
            .and_then(|tags| tags.get_string(&ItemKey::TrackArtist))
            .map(|str| str.to_owned());

        let album = tags_maybe
            .and_then(|tags| tags.get_string(&ItemKey::AlbumTitle))
            .map(|str| str.to_owned());

        let album_artist = tags_maybe
            .and_then(|tags| tags.get_string(&ItemKey::AlbumArtist))
            .map(|str| str.to_owned());

        let composer = tags_maybe
            .and_then(|tags| tags.get_string(&ItemKey::Composer))
            .map(|str| str.to_owned());

        let title_sort = tags_maybe
            .and_then(|tags| tags.get_string(&ItemKey::TrackTitleSortOrder))
            .map(|str| str.to_owned())
            .or_else(|| {
                path.file_stem()
                    .map(|str| str.to_string_lossy().into_owned())
            });

        let artist_sort = tags_maybe
            .and_then(|tags| tags.get_string(&ItemKey::TrackArtistSortOrder))
            .map(|str| str.to_owned());

        let album_sort = tags_maybe
            .and_then(|tags| tags.get_string(&ItemKey::AlbumTitleSortOrder))
            .map(|str| str.to_owned());

        let album_artist_sort = tags_maybe
            .and_then(|tags| tags.get_string(&ItemKey::AlbumArtistSortOrder))
            .map(|str| str.to_owned());

        let composer_sort = tags_maybe
            .and_then(|tags| tags.get_string(&ItemKey::ComposerSortOrder))
            .map(|str| str.to_owned());

        let genre = tags_maybe
            .and_then(|tags| tags.get_string(&ItemKey::Genre))
            .map(|str| str.to_owned());

        let track_no = tags_maybe.and_then(|tags| tags.track()).map(|index| index);

        let track_total = tags_maybe
            .and_then(|tags| tags.track_total())
            .map(|total| total);

        let disc_no = tags_maybe.and_then(|tags| tags.disk()).map(|index| index);

        let disc_total = tags_maybe
            .and_then(|tags| tags.disk_total())
            .map(|total| total);

        let year = tags_maybe.and_then(|tags| tags.year()).map(|year| year);

        let compilation = tags_maybe
            .and_then(|tags| tags.get_binary(&ItemKey::FlagCompilation, false))
            .and_then(|bin| bin.first().map(|first| *first != 0));

        let comment = tags_maybe
            .and_then(|tags| tags.get_string(&ItemKey::Comment))
            .map(|str| str.to_owned());

        let lyrics = tags_maybe
            .and_then(|tags| tags.get_string(&ItemKey::Lyrics))
            .map(|str| str.to_owned());

        let copyright = tags_maybe
            .and_then(|tags| tags.get_string(&ItemKey::CopyrightMessage))
            .map(|str| str.to_owned());

        let artwork: Vec<Picture> = tags_maybe
            .map(|tag| tag.pictures().iter().cloned().collect())
            .unwrap_or_default();

        let podcast_url = tags_maybe
            .and_then(|tags| tags.get_string(&ItemKey::PodcastUrl))
            .map(|str| str.to_owned());

        let podcast_desc = tags_maybe
            .and_then(|tags| tags.get_string(&ItemKey::PodcastDescription))
            .map(|str| str.to_owned());

        let podcast_flag = tags_maybe
            .and_then(|t| t.get_binary(&ItemKey::FlagPodcast, false))
            .map(|bytes| bytes.iter().any(|&b| b != 0))
            .unwrap_or(false);

        let compilation_flag = tags_maybe
            .and_then(|t| t.get_binary(&ItemKey::FlagCompilation, false))
            .map(|bytes| bytes.iter().any(|&b| b != 0))
            .unwrap_or(false);

        Ok(Self {
            title,
            album,
            artist,
            album_artist,
            composer,
            title_sort,
            album_sort,
            artist_sort,
            album_artist_sort,
            composer_sort,
            track_no,
            track_total,
            disc_no,
            disc_total,
            genre,
            year,
            compilation,
            comment,
            lyrics,
            copyright,
            podcast_url,
            podcast_desc,
            podcast_flag,
            compilation_flag,
            artwork,
        })
    }

    pub fn save_to<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        use lofty::probe::Probe as LoftyProbe;

        let path = path.as_ref();
        let probe = LoftyProbe::open(path)?.options(
            ParseOptions::default()
                .read_properties(false)
                .parsing_mode(ParsingMode::Relaxed),
        );

        let mut tagged_file = probe.read()?;
        let tag_type = tagged_file.primary_tag_type();
        let tag = ensure_tag(&mut tagged_file, tag_type);

        tag.clear();
        apply_text(tag, ItemKey::TrackTitle, &self.title);
        apply_text(tag, ItemKey::AlbumTitle, &self.album);
        apply_text(tag, ItemKey::TrackTitle, &self.title);
        apply_text(tag, ItemKey::AlbumTitle, &self.album);
        apply_text(tag, ItemKey::TrackArtist, &self.artist);
        apply_text(tag, ItemKey::AlbumArtist, &self.album_artist);
        apply_text(tag, ItemKey::Composer, &self.composer);
        apply_text(tag, ItemKey::TrackTitleSortOrder, &self.title_sort);
        apply_text(tag, ItemKey::AlbumTitleSortOrder, &self.album_sort);
        apply_text(tag, ItemKey::TrackArtistSortOrder, &self.artist_sort);
        apply_text(tag, ItemKey::AlbumArtistSortOrder, &self.album_artist_sort);
        apply_text(tag, ItemKey::ComposerSortOrder, &self.composer_sort);
        apply_u32(tag, ItemKey::TrackNumber, &self.track_no);
        apply_u32(tag, ItemKey::TrackTotal, &self.track_total);
        apply_u32(tag, ItemKey::DiscNumber, &self.disc_no);
        apply_u32(tag, ItemKey::DiscTotal, &self.disc_total);
        apply_text(tag, ItemKey::Genre, &self.genre);
        apply_u32(tag, ItemKey::Year, &self.year);
        apply_text(tag, ItemKey::Comment, &self.comment);
        apply_text(tag, ItemKey::Lyrics, &self.lyrics);
        apply_text(tag, ItemKey::CopyrightMessage, &self.copyright);
        apply_text(tag, ItemKey::PodcastUrl, &self.podcast_url);
        apply_text(tag, ItemKey::PodcastDescription, &self.podcast_desc);
        apply_bool(tag, ItemKey::FlagPodcast, &self.podcast_flag);

        for pic in &self.artwork {
            tag.push_picture(pic.clone());
        }
        tagged_file.save_to_path(path, WriteOptions::default())?;
        Ok(())
    }
}

fn ensure_tag(tagged_file: &mut TaggedFile, tag_type: TagType) -> &mut Tag {
    if tagged_file.tag_mut(tag_type).is_none() {
        tagged_file.insert_tag(Tag::new(tag_type));
    }
    tagged_file
        .tag_mut(tag_type)
        .expect("insert_tag then tag_mut must succeed")
}

fn apply_text(tag: &mut Tag, key: ItemKey, val: &Option<String>) -> bool {
    match val {
        Some(val) => tag.insert(TagItem::new(key, ItemValue::Text(val.clone()))),
        None => false,
    }
}

fn apply_u32(tag: &mut Tag, key: ItemKey, val: &Option<u32>) -> bool {
    match val {
        Some(val) => tag.insert(TagItem::new(key, ItemValue::Text(val.to_string()))),
        None => false,
    }
}

fn apply_bool(tag: &mut Tag, key: ItemKey, val: &bool) -> bool {
    if *val {
        tag.insert(TagItem::new(key, ItemValue::Binary(vec![1])))
    } else {
        false
    }
}
