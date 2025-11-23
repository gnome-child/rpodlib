use std::fmt::{Formatter, Result};

use rpod_itdb::{SizeRange, album_item::AlbumItem};

use crate::fmt::{Body, TreeContext, TreeDisplay};

impl TreeDisplay for AlbumItem {
    fn tree_fmt(&self, f: &mut Formatter<'_>, ctx: &TreeContext) -> Result {
        let title = format!(
            "mhia (header: {} bytes, total: {} bytes)",
            self.header_bytes_len(),
            self.total_bytes_len()
        );
        let footer = format!("data objects [{}]", self.data_objects.len());

        let mut body = Body::new();

        body.push("album id:", self.album_id)
            .push("sql id:", self.sql_id)
            .push("unk 0x1C:", self.unk_0x1C)
            .push("first track uid:", self.first_track_uid);

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
