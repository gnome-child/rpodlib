use std::fmt::{Formatter, Result};

use rpod_itdb::{
    SizeRange,
    playlist::{PlaylistEntry, PlaylistItem},
};

use crate::fmt::{Body, TreeContext, TreeDisplay};

impl TreeDisplay for PlaylistItem {
    fn tree_fmt(&self, f: &mut Formatter<'_>, ctx: &TreeContext) -> Result {
        let title = format!(
            "mhyp (header: {} bytes, total: {} bytes)",
            self.header_bytes_len(),
            self.total_bytes_len()
        );
        let footer1 = format!("data objects [{}]", self.data_objects.len());
        let footer2 = format!("playlist entries [{}]", self.data_objects.len());

        let mut body = Body::new();

        body.push_fmt(
            "master playlist:",
            format_args!("{}", self.is_master_flag != 0),
        )
        .push("flag 0x15:", self.flag_0x15)
        .push("flag 0x16:", self.flag_0x16)
        .push("flag 0x17:", self.flag_0x17)
        .push("time created:", self.hfs_timestamp_created)
        .push("uid:", self.uid)
        .push("unk 0x24:", self.unk_0x24)
        .push_fmt(
            "podcast playlist:",
            format_args!("{}", self.is_podcast_playlist_flag != 0),
        )
        .push("sort order:", self.sort_order)
        .push("unk 0x30:", self.unk_0x30)
        .push("unk 0x34:", self.unk_0x34)
        .push("unk 0x38:", self.unk_0x38)
        .push("database id:", self.database_id)
        .push("persistent id copy:", self.persistent_id_copy)
        .push("unk 0x4C:", self.unk_0x4C)
        .push("unk 0x50:", self.unk_0x50)
        .push("unk 0x54:", self.unk_0x54)
        .push("time modified:", self.hfs_timestamp_modified);

        ctx.writeln_empty(f, "")?;
        ctx.writeln_title(f, title)?;
        ctx.write_body(f, &mut body)?;
        ctx.writeln_empty(f, "")?;
        ctx.writeln_body(f, footer1)?;

        if f.alternate() {
            for (index, child) in self.data_objects.iter().enumerate() {
                let next = ctx.descend(index + 1 == self.data_objects.len());

                child.tree_fmt(f, &next)?;
            }
        }

        ctx.writeln_empty(f, "")?;
        ctx.writeln_body(f, footer2)?;

        if f.alternate() {
            for (index, child) in self.entries.iter().enumerate() {
                let next = ctx.descend(index + 1 == self.entries.len());

                child.tree_fmt(f, &next)?;
            }
        }
        Ok(())
    }
}

impl TreeDisplay for PlaylistEntry {
    fn tree_fmt(&self, f: &mut Formatter<'_>, ctx: &TreeContext) -> Result {
        let title = format!(
            "mhip (header: {} bytes, total: {} bytes)",
            self.header_bytes_len(),
            self.total_bytes_len()
        );
        let footer = format!("data objects [{}]", self.data_objects.len());

        let mut body = Body::new();

        body.push("unk 0x16:", self.unk_0x16)
            .push_fmt(
                "podcast group flag:",
                format_args!("{}", self.podcast_group_flag != 0),
            )
            .push("unk 0x18:", self.unk_0x18)
            .push("unk 0x19:", self.unk_0x19)
            .push("podcast group id:", self.podcast_group_id)
            .push("track id:", self.track_id)
            .push("timestamp:", self.timestamp)
            .push("podcast group ref:", self.podcast_group_ref)
            .push("podcast uid:", self.podcast_uid)
            .push("track uid:", self.track_uid)
            .push("unk 0x34:", self.unk_0x34)
            .push("unk 0x38:", self.unk_0x38)
            .push("uid:", self.uid);

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
