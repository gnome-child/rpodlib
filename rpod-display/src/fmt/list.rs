use std::fmt::{Formatter, Result};

use rpod_itdb::{
    SizeRange,
    list::{AlbumItemList, List, ListContainer, PlaylistItemList, TrackItemList},
};

use crate::fmt::{TreeContext, TreeDisplay};

impl TreeDisplay for ListContainer {
    fn tree_fmt(&self, f: &mut Formatter<'_>, ctx: &TreeContext) -> Result {
        let title = format!(
            "mhsd (header: {} bytes, total: {} bytes)",
            self.header_bytes_len(),
            self.total_bytes_len()
        );
        let list_type = format!("type: {} ({:?})", self.list_type as u32, self.list_type);

        ctx.writeln_empty(f, "")?;
        ctx.writeln_title(f, title)?;
        ctx.writeln_body(f, list_type)?;
        self.list.tree_fmt(f, &ctx.descend(true))?;
        Ok(())
    }
}

impl TreeDisplay for List {
    fn tree_fmt(&self, f: &mut Formatter<'_>, ctx: &TreeContext) -> Result {
        match self {
            List::TrackItems(item) => item.tree_fmt(f, ctx),
            List::LibraryPlaylists(item) => item.tree_fmt(f, ctx),
            List::PodcastFmtLibPlaylists(item) => item.tree_fmt(f, ctx),
            List::AlbumItems(item) => item.tree_fmt(f, ctx),
            List::SpecialPlaylists(item) => item.tree_fmt(f, ctx),
        }
    }
}

impl TreeDisplay for TrackItemList {
    fn tree_fmt(&self, f: &mut Formatter<'_>, ctx: &TreeContext) -> Result {
        let title = format!(
            "mhlt (header: {} bytes, total: {} bytes)",
            self.header_bytes_len(),
            self.total_bytes_len()
        );
        let footer = format!("items [{}]", self.items.len());

        ctx.writeln_empty(f, "")?;
        ctx.writeln_title(f, title)?;
        ctx.writeln_body(f, footer)?;

        if f.alternate() {
            for (index, child) in self.items.iter().enumerate() {
                let next = ctx.descend(index + 1 == self.items.len());

                child.tree_fmt(f, &next)?;
            }
        }
        Ok(())
    }
}

impl TreeDisplay for AlbumItemList {
    fn tree_fmt(&self, f: &mut Formatter<'_>, ctx: &TreeContext) -> Result {
        let title = format!(
            "mhla (header: {} bytes, total: {} bytes)",
            self.header_bytes_len(),
            self.total_bytes_len()
        );
        let footer = format!("items [{}]", self.items.len());

        ctx.writeln_empty(f, "")?;
        ctx.writeln_title(f, title)?;
        ctx.writeln_body(f, footer)?;
        Ok(())
    }
}

impl TreeDisplay for PlaylistItemList {
    fn tree_fmt(&self, f: &mut Formatter<'_>, ctx: &TreeContext) -> Result {
        let title = format!(
            "mhlp (header: {} bytes, total: {} bytes)",
            self.header_bytes_len(),
            self.total_bytes_len()
        );
        let footer = format!("items [{}]", self.items.len());

        ctx.writeln_empty(f, "")?;
        ctx.writeln_title(f, title)?;
        ctx.writeln_body(f, footer)?;
        Ok(())
    }
}
