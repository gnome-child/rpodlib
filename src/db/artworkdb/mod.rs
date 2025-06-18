#![allow(unused, non_camel_case_types, non_snake_case)]

use binrw::binrw;

#[binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub(crate) enum Record {
    #[brw(magic = b"mhfd")]
    mhbd(Unimplemented),

    #[brw(magic = b"mhsd")]
    mhsd(Unimplemented),

    #[brw(magic = b"mhli")]
    mhli(Unimplemented),

    #[brw(magic = b"mhla")]
    mhla(Unimplemented),

    #[brw(magic = b"mhlf")]
    mhlf(Unimplemented),

    #[brw(magic = b"mhii")]
    mhii(Unimplemented),

    #[brw(magic = b"mhif")]
    mhif(Unimplemented),

    #[brw(magic = b"mhod")]
    mhod(Unimplemented),

    #[brw(magic = b"mhni")]
    mhni(Unimplemented),

    #[brw(magic = b"mhaf")]
    mhaf(Unimplemented),
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub(crate) struct Unimplemented {
    header_len: u32,
    len: u32,

    #[br(count = len - 12)]
    bytes: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use binrw::BinRead;

    use super::Record;

    #[test]
    fn parse_artworkdb() {
        let bytes = include_bytes!("./sample/ArtworkDB");
        let mut cursor = Cursor::new(&bytes[..]);
        let mut root: Record = Record::read(&mut cursor).expect("failed");
    }
}
