use std::fmt::{Formatter, Result};

use pretty_hex::simple_hex;
use rpod_itdb::{SizeRange, root::Root};

use crate::fmt::{Body, TreeContext, TreeDisplay};

impl TreeDisplay for Root {
    fn tree_fmt(&self, f: &mut Formatter<'_>, ctx: &TreeContext) -> Result {
        let title = format!(
            "mhbd (header: {} bytes, total: {} bytes)",
            self.header_bytes_len(),
            self.total_bytes_len()
        );

        let footer = format!("list containers [{}]", self.list_containers.len());

        let mut body = Body::new();

        body.push_debug("persistent id:", &self.persistent_id)
            .push_fmt("version:", format_args!("v{}", self.version))
            .push_fmt(
                "language:",
                format_args!("{}", String::from_utf8_lossy(&self.lang)),
            )
            .push_fmt("timezone offset:", format_args!("{}", self.timezone_offset))
            .push_fmt(
                "database id:",
                format_args!("{:#018X}", self.generation_uid),
            )
            .push("hash58:", simple_hex(&self.hash_0x58))
            .push_fmt("unk_0x0C:", format_args!("{}", self.unk_0x0C))
            .push_fmt("unk_0x20:", format_args!("{}", self.unk_0x20))
            .push_fmt("unk_0x22:", format_args!("{}", self.unk_0x22))
            .push_fmt("unk_0x24:", format_args!("{}", self.database_uid))
            .push_fmt("unk_0x2C:", format_args!("{}", self.unk_0x2C))
            .push_fmt("unk_0x30:", format_args!("{}", self.unk_0x30))
            .push_fmt("unk_0x32:", format_args!("{}", self.unk_0x32))
            .push_fmt("unk_0x34:", format_args!("{}", self.unk_0x34))
            .push_fmt("unk_0x38:", format_args!("{}", self.unk_0x38))
            .push_fmt("unk_0x3C:", format_args!("{}", self.unk_0x3C))
            .push_fmt("unk_0x40:", format_args!("{}", self.unk_0x40))
            .push_fmt("unk_0x44:", format_args!("{}", self.unk_0x44))
            .push("hash72:", simple_hex(&self.hash_0x72))
            .push_fmt("unk_0xA0:", format_args!("{}", self.unk_0xA0))
            .push("audio lang:", simple_hex(&self.audio_lang))
            .push("subtitle lang:", simple_hex(&self.subtitle_lang));

        ctx.writeln_title(f, title)?;
        ctx.write_body(f, &mut body)?;
        ctx.writeln_empty(f, "")?;
        ctx.writeln_body(f, footer)?;

        if f.alternate() {
            for (index, child) in self.list_containers.iter().enumerate() {
                let next = ctx.descend(index + 1 == self.list_containers.len());

                child.tree_fmt(f, &next)?;
            }
        }
        Ok(())
    }
}
