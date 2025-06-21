#![allow(unused, non_camel_case_types, non_snake_case)]

use binrw::binrw;

#[binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub(crate) enum Record {
    #[brw(magic = b"mhfd")]
    mhfd(Master),

    #[brw(magic = b"mhsd")]
    mhsd(ListContainer),

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

#[binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub(crate) struct Master {
    header_len: u32,
    len: u32,
    unk_0x0C: u32,
    unk_0x10: u32,

    #[bw(calc = children.len() as u32)]
    child_count: u32,

    unk_0x18: u32,
    next_mhii_id: u32,
    unk_0x20: u64,
    unk_0x28: u64,
    unk_0x30: u32,
    unk_0x34: u32,
    unk_0x38: u32,
    unk_0x3C: u32,
    unk_0x40: u32,

    #[brw(pad_before = 64)]
    #[br(count = child_count)]
    children: Vec<Record>,
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub(crate) struct ListContainer {
    #[bw(calc = 96)]
    header_len: u32,

    len: u32,

    #[bw(calc = list.as_u32())]
    list_type: u32,

    #[brw(pad_before = 80)]
    #[br(args { list_type: list_type })]
    list: List,
}

#[binrw]
#[brw(little)]
#[br(import { list_type: u32 })]
#[derive(Debug, Clone)]
pub enum List {
    #[br(pre_assert(list_type == 0x01))]
    #[brw(magic = b"mhli")]
    Images(RecordList),

    #[br(pre_assert(list_type == 0x02))]
    #[brw(magic = b"mhla")]
    Albums(RecordList),

    #[br(pre_assert(list_type == 0x03))]
    #[brw(magic = b"mhlf")]
    Files(RecordList),
}

impl List {
    pub fn as_u32(&self) -> u32 {
        match self {
            List::Images(_) => 0x01,
            List::Albums(_) => 0x02,
            List::Files(_) => 0x03,
        }
    }
}

#[binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub(crate) struct RecordList {
    #[bw(calc = 92)]
    header_len: u32,

    #[bw(calc = children.len() as u32)]
    child_count: u32,

    #[brw(pad_before = 80)]
    #[br(count = child_count)]
    children: Vec<Record>,
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
