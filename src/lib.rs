#![allow(unused, non_camel_case_types)]

use std::path::PathBuf;

pub(crate) mod db;
pub(crate) mod util;

pub struct iPod {
    path: PathBuf,
    fwid: String,
    serial_num: String,
    product_type: String,
    build_version: String,
    itunesdb: db::itunesdb::Record,
}

impl iPod {}

#[cfg(test)]
mod tests {
    use quick_xml::{events::Event, Reader};
    use std::{fs::File, io::BufReader, path::PathBuf};

    #[test]
    fn check_fwid() {
        let path: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/db/itunesdb/sample/ExtendedSysInfoXml");

        let file = File::open(&path).expect("it broke");

        let mut reader = Reader::from_reader(BufReader::new(file));
        reader.config_mut().trim_text(true);

        let mut buf = Vec::<u8>::new();
        let mut fwid_key = false;
        let mut fwid = None;

        loop {
            match reader.read_event_into(&mut buf).expect("it broke") {
                Event::Text(event) if fwid_key => {
                    fwid = Some(event.unescape().expect("it broke").to_string());
                    break;
                }
                Event::Text(event) if event.as_ref() == b"FireWireGUID" => {
                    fwid_key = true;
                }
                Event::Eof => break,
                _ => {}
            }
            buf.clear();
        }

        assert!(fwid.is_some())
    }
}
