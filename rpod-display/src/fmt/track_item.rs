use std::fmt::{Formatter, Result};

use rpod_itdb::{SizeRange, track_item::TrackItem};

use crate::fmt::{Body, TreeContext, TreeDisplay};

impl TreeDisplay for TrackItem {
    fn tree_fmt(&self, f: &mut Formatter<'_>, ctx: &TreeContext) -> Result {
        let title = format!(
            "mhit (header: {} bytes, total: {} bytes)",
            self.header_bytes_len(),
            self.total_bytes_len()
        );
        let footer = format!("data objects [{}]", self.data_objects.len());

        let mut body = Body::new();

        body.push_fmt("id:", format_args!("{}", self.id))
            .push_fmt("visible:", format_args!("{}", self.visible))
            .push_fmt(
                "file type (norm):",
                format_args!("{}", self.file_type_normalized()),
            )
            .push_fmt(
                "file_type_fourcc:",
                format_args!("{:?}", self.file_type_fourcc),
            )
            .push_fmt("mp3_vbr_flag:", format_args!("{}", self.mp3_vbr_flag))
            .push_fmt("mp3_flag:", format_args!("{}", self.mp3_flag))
            .push_fmt(
                "compilation_flag:",
                format_args!("{}", self.compilation_flag),
            )
            .push_fmt("rating:", format_args!("{}", self.rating))
            .push_fmt(
                "hfs_time_last_modified:",
                format_args!("{}", self.hfs_time_last_modified),
            )
            .push_fmt(
                "file_size_bytes_u32:",
                format_args!("{}", self.file_size_bytes_u32),
            )
            .push_fmt("duration_ms:", format_args!("{}", self.duration_ms))
            .push_fmt("album_index:", format_args!("{}", self.album_index))
            .push_fmt(
                "album_track_count:",
                format_args!("{}", self.album_track_count),
            )
            .push_fmt("release_year:", format_args!("{}", self.release_year))
            .push_fmt("bitrate (kbps):", format_args!("{}", self.bitrate))
            .push_fmt(
                "sample_rate (fixed 16.16):",
                format_args!("{}", self.sample_rate),
            )
            .push_fmt(
                "playback_volume_adj:",
                format_args!("{}", self.playback_volume_adj),
            )
            .push_fmt("start_offset_ms:", format_args!("{}", self.start_offset_ms))
            .push_fmt("stop_offset_ms:", format_args!("{}", self.stop_offset_ms))
            .push_fmt("soundcheck:", format_args!("{}", self.soundcheck))
            .push_fmt("play_count_1:", format_args!("{}", self.play_count_1))
            .push_fmt("play_count_2:", format_args!("{}", self.play_count_2))
            .push_fmt(
                "hfs_time_last_played:",
                format_args!("{}", self.hfs_time_last_played),
            )
            .push_fmt(
                "album_disc_index:",
                format_args!("{}", self.album_disc_index),
            )
            .push_fmt(
                "album_disc_count:",
                format_args!("{}", self.album_disc_count),
            )
            .push_fmt("drm_user_id:", format_args!("{}", self.drm_user_id))
            .push_fmt(
                "hfs_time_date_added:",
                format_args!("{}", self.hfs_time_date_added),
            )
            .push_fmt("bookmark_ms:", format_args!("{}", self.bookmark_ms))
            .push_fmt("uid:", format_args!("{:#018X}", self.uid))
            .push_fmt("unchecked_flag:", format_args!("{}", self.unchecked_flag))
            .push_fmt("last_rating:", format_args!("{}", self.last_rating))
            .push_fmt("bpm:", format_args!("{}", self.bpm))
            .push_fmt("artwork_count:", format_args!("{}", self.artwork_count))
            .push_fmt(
                "audio_format_tag:",
                format_args!("{:#06X}", self.audio_format_tag),
            )
            .push_fmt(
                "artwork_size_bytes:",
                format_args!("{}", self.artwork_size_bytes),
            )
            .push_fmt("unk_0x84:", format_args!("{:#010X}", self.unk_0x84))
            .push_fmt(
                "IEEE_f32_sample_rate:",
                format_args!("{}", self.IEEE_f32_sample_rate),
            )
            .push_fmt(
                "hfs_time_release_date:",
                format_args!("{}", self.hfs_time_release_date),
            )
            .push_fmt("unk_0x90:", format_args!("{:#06X}", self.unk_0x90))
            .push_fmt("explicit_flag:", format_args!("{}", self.explicit_flag))
            .push_fmt("unk_0x94:", format_args!("{:#010X}", self.unk_0x94))
            .push_fmt("unk_0x98:", format_args!("{:#010X}", self.unk_0x98))
            .push_fmt("skip_count:", format_args!("{}", self.skip_count))
            .push_fmt(
                "hfs_time_last_skipped:",
                format_args!("{}", self.hfs_time_last_skipped),
            )
            .push_fmt("has_artwork:", format_args!("{}", self.has_artwork))
            .push_fmt(
                "skip_on_shuffle_flag:",
                format_args!("{}", self.skip_on_shuffle_flag),
            )
            .push_fmt(
                "remember_playback_position_flag:",
                format_args!("{}", self.remember_playback_position_flag),
            )
            .push_fmt("podcast_flag:", format_args!("{}", self.podcast_flag))
            .push_fmt("uid_copy:", format_args!("{:#018X}", self.uid_copy))
            .push_fmt("has_lyrics_flag:", format_args!("{}", self.has_lyrics_flag))
            .push_fmt("is_movie_flag:", format_args!("{}", self.is_movie_flag))
            .push_fmt(
                "podcast_unplayed:",
                format_args!("{}", self.podcast_unplayed),
            )
            .push_fmt("unk_0xB3:", format_args!("{}", self.unk_0xB3))
            .push_fmt(
                "bookmark_ms_copy:",
                format_args!("{}", self.bookmark_ms_copy),
            )
            .push_fmt(
                "samples_before_start_gapless:",
                format_args!("{}", self.samples_before_start_gapless),
            )
            .push_fmt(
                "samples_count_gapless:",
                format_args!("{}", self.sample_count),
            )
            .push_fmt("unk_0xC4:", format_args!("{:#010X}", self.unk_0xC4))
            .push_fmt(
                "samples_before_end_gapless:",
                format_args!("{}", self.samples_before_end_gapless),
            )
            .push_fmt("unk_0xCC:", format_args!("{}", self.unk_0xCC))
            .push_fmt(
                "media_type:",
                format_args!("{} ({:?})", self.media_type as u32, self.media_type),
            )
            .push_fmt("season_number:", format_args!("{}", self.season_number))
            .push_fmt("episode_number:", format_args!("{}", self.episode_number))
            .push_fmt("unk_0xDC:", format_args!("{:#010X}", self.unk_0xDC))
            .push_fmt("unk_0xE0:", format_args!("{:#010X}", self.unk_0xE0))
            .push_fmt("unk_0xE4:", format_args!("{:#010X}", self.unk_0xE4))
            .push_fmt("unk_0xE8:", format_args!("{:#010X}", self.unk_0xE8))
            .push_fmt("unk_0xEC:", format_args!("{:#010X}", self.unk_0xEC))
            .push_fmt("unk_0xF0:", format_args!("{:#010X}", self.unk_0xF0))
            .push_fmt("unk_0xF4:", format_args!("{:#010X}", self.unk_0xF4))
            .push_fmt("gapless_data:", format_args!("{}", self.end_prefetch_bytes))
            .push_fmt("unk_0xFC:", format_args!("{:#010X}", self.unk_0xFC))
            .push_fmt(
                "is_gapless_track_flag:",
                format_args!("{}", self.is_gapless_track_flag),
            )
            .push_fmt(
                "is_gapless_album_flag:",
                format_args!("{}", self.is_gapless_album_flag),
            )
            .push_fmt("unk_0x0104:", format_args!("{:#010X}", self.unk_0x0104))
            .push_fmt("unk_0x0108:", format_args!("{:#010X}", self.unk_0x0108))
            .push_fmt("unk_0x010C:", format_args!("{:#010X}", self.unk_0x010C))
            .push_fmt("unk_0x0110:", format_args!("{:#010X}", self.unk_0x0110))
            .push_fmt("unk_0x0114:", format_args!("{:#010X}", self.unk_0x0114))
            .push_fmt("unk_0x0118:", format_args!("{:#018X}", self.unk_0x0118))
            .push_fmt("album_id:", format_args!("{}", self.album_id))
            .push_fmt("database_id:", format_args!("{:#018X}", self.database_id))
            .push_fmt(
                "file_size_bytes_u64:",
                format_args!("{}", self.file_size_bytes_u64),
            )
            .push_fmt("unk_0x0134:", format_args!("{:#018X}", self.unk_0x0134))
            .push_fmt("unk_0x013C:", format_args!("{:#010X}", self.unk_0x013C))
            .push_fmt("unk_0x0140:", format_args!("{:#010X}", self.unk_0x0140))
            .push_fmt("unk_0x0144:", format_args!("{:#010X}", self.unk_0x0144))
            .push_fmt("unk_0x0148:", format_args!("{:#06X}", self.unk_0x0148))
            .push_fmt("unk_0x014B:", format_args!("{:#010X}", self.unk_0x014B))
            .push_fmt("unk_0x014F:", format_args!("{:#010X}", self.unk_0x014F))
            .push_fmt("unk_0x0153:", format_args!("{:#010X}", self.unk_0x0153))
            .push_fmt("unk_0x0157:", format_args!("{:#010X}", self.unk_0x0157))
            .push_fmt("unk_0x015B:", format_args!("{:#010X}", self.unk_0x015B))
            .push_fmt("unk_0x015F:", format_args!("{:#06X}", self.unk_0x015F))
            .push_fmt("mhii_link:", format_args!("{}", self.mhii_link))
            .push_fmt("unk_0x0164:", format_args!("{:#010X}", self.unk_0x0164))
            .push_fmt("unk_0x0168:", format_args!("{:#010X}", self.unk_0x0168))
            .push_fmt("unk_0x016C:", format_args!("{:#010X}", self.unk_0x016C))
            .push_fmt("unk_0x0170:", format_args!("{:#010X}", self.unk_0x0170))
            .push_fmt("unk_0x0174:", format_args!("{:#010X}", self.unk_0x0174))
            .push_fmt("unk_0x0178:", format_args!("{:#010X}", self.unk_0x0178))
            .push_fmt("unk_0x017C:", format_args!("{:#010X}", self.unk_0x017C))
            .push_fmt("unk_0x0180:", format_args!("{:#010X}", self.unk_0x0180))
            .push_fmt("unk_0x0184:", format_args!("{:#010X}", self.unk_0x0184))
            .push_fmt("unk_0x0188:", format_args!("{:#010X}", self.unk_0x0188))
            .push_fmt("unk_0x018C:", format_args!("{:#010X}", self.unk_0x018C))
            .push_fmt("unk_0x0190:", format_args!("{:#010X}", self.unk_0x0190))
            .push_fmt("unk_0x0194:", format_args!("{:#010X}", self.unk_0x0194))
            .push_fmt("unk_0x0198:", format_args!("{:#010X}", self.unk_0x0198))
            .push_fmt("unk_0x019C:", format_args!("{:#010X}", self.unk_0x019C))
            .push_fmt("unk_0x01A0:", format_args!("{:#010X}", self.unk_0x01A0))
            .push_fmt("unk_0x01A4:", format_args!("{:#010X}", self.unk_0x01A4))
            .push_fmt("unk_0x01A8:", format_args!("{:#010X}", self.unk_0x01A8))
            .push_fmt("unk_0x01AC:", format_args!("{:#010X}", self.unk_0x01AC))
            .push_fmt("unk_0x01B0:", format_args!("{:#010X}", self.unk_0x01B0))
            .push_fmt("unk_0x01B4:", format_args!("{:#010X}", self.unk_0x01B4))
            .push_fmt("unk_0x01B8:", format_args!("{:#010X}", self.unk_0x01B8))
            .push_fmt("unk_0x01BC:", format_args!("{:#010X}", self.unk_0x01BC))
            .push_fmt("unk_0x01C0:", format_args!("{:#010X}", self.unk_0x01C0))
            .push_fmt("unk_0x01C4:", format_args!("{:#010X}", self.unk_0x01C4))
            .push_fmt("unk_0x01C8:", format_args!("{:#010X}", self.unk_0x01C8))
            .push_fmt("unk_0x01CC:", format_args!("{:#010X}", self.unk_0x01CC))
            .push_fmt("unk_0x01D0:", format_args!("{:#010X}", self.unk_0x01D0))
            .push_fmt("unk_0x01D4:", format_args!("{:#010X}", self.unk_0x01D4))
            .push_fmt("unk_0x01D8:", format_args!("{:#010X}", self.unk_0x01D8))
            .push_fmt("unk_0x01DC:", format_args!("{:#010X}", self.unk_0x01DC))
            .push_fmt("artist_id:", format_args!("{}", self.artist_id))
            .push_fmt("unk_0x01E4:", format_args!("{:#010X}", self.unk_0x01E4))
            .push_fmt("unk_0x01E8:", format_args!("{:#010X}", self.unk_0x01E8))
            .push_fmt("unk_0x01EC:", format_args!("{:#010X}", self.unk_0x01EC))
            .push_fmt("unk_0x01F0:", format_args!("{:#010X}", self.unk_0x01F0))
            .push_fmt("composer_id:", format_args!("{}", self.composer_id))
            .push_fmt("unk_0x01F8:", format_args!("{:#010X}", self.unk_0x01F8))
            .push_fmt("unk_0x01FC:", format_args!("{:#010X}", self.unk_0x01FC))
            .push_fmt("unk_0x0200:", format_args!("{:#010X}", self.unk_0x0200))
            .push_fmt("unk_0x0204:", format_args!("{:#010X}", self.unk_0x0204))
            .push_fmt("unk_0x0208:", format_args!("{:#010X}", self.unk_0x0208))
            .push_fmt("unk_0x020C:", format_args!("{:#010X}", self.unk_0x020C))
            .push_fmt("unk_0x0210:", format_args!("{:#010X}", self.unk_0x0210))
            .push_fmt("unk_0x0214:", format_args!("{:#010X}", self.unk_0x0214))
            .push_fmt("unk_0x0218:", format_args!("{:#010X}", self.unk_0x0218))
            .push_fmt("unk_0x021C:", format_args!("{:#010X}", self.unk_0x021C))
            .push_fmt("unk_0x0220:", format_args!("{:#010X}", self.unk_0x0220))
            .push_fmt("unk_0x0224:", format_args!("{:#010X}", self.unk_0x0224))
            .push_fmt("unk_0x0228:", format_args!("{:#010X}", self.unk_0x0228))
            .push_fmt("unk_0x022C:", format_args!("{:#010X}", self.unk_0x022C));

        ctx.writeln_empty(f, "")?;
        ctx.writeln_title(f, title)?;
        ctx.write_body(f, &mut body)?;
        ctx.writeln_empty(f, "")?;
        ctx.writeln_body(f, footer)?;

        if f.alternate() {
            for (index, child) in self.data_objects.iter().enumerate() {
                let next = ctx.descend(index + 1 == self.data_objects.len());

                child.tree_fmt(f, &next)?;
            }
        }
        Ok(())
    }
}
