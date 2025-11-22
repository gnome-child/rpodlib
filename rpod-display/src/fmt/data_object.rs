use std::fmt::{Formatter, Result};

use rpod_itdb::{
    SizeRange,
    data_object::{DataObject, Payload},
};

use crate::fmt::{Body, TreeContext, TreeDisplay};

impl TreeDisplay for DataObject {
    fn tree_fmt(&self, f: &mut Formatter<'_>, ctx: &TreeContext) -> Result {
        let title = format!(
            "mhod (header: {} bytes, total: {} bytes)",
            self.header_bytes_len(),
            self.total_bytes_len()
        );

        let mut body = Body::new();

        body.push_fmt(
            "data type:",
            format_args!("{} ({:?})", self.data_type as u32, self.data_type),
        );

        match &self.payload {
            Payload::Utf16String(payload) => {
                body.push_fmt(
                    "data:",
                    format_args!("{}", String::from_utf16_lossy(&payload.chars)),
                );
            }
            Payload::Utf8Chars(payload) => {
                body.push_fmt(
                    "data:",
                    format_args!("{}", String::from_utf8_lossy(&payload.chars)),
                );
            }
            _ => {} // ignore other payload kinds
        }

        ctx.writeln_empty(f, "")?;
        ctx.writeln_title(f, title)?;
        ctx.write_body(f, &mut body)?;
        Ok(())
    }
}
