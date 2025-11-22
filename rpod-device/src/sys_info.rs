use std::{fs, path::Path};

use plist::{Dictionary, Value};

use crate::{
    capabilities::audio::{Aac, Aiff, AppleLossless, Audible, AudioCapability, Mp3},
    error::Result,
};

#[derive(Debug)]
pub enum ChecksumType {
    None,
    Hash58,
    Hash72,
    HashAB,
    Unknown,
}

#[derive(Debug)]
pub enum QuerySegment {
    Key(String),
    Index(usize),
}

impl From<&str> for QuerySegment {
    fn from(value: &str) -> Self {
        Self::Key(value.to_owned())
    }
}

impl From<usize> for QuerySegment {
    fn from(value: usize) -> Self {
        Self::Index(value)
    }
}

pub struct SystemInfo {
    inner: plist::Value,
}

impl SystemInfo {
    pub fn parse(path: &Path) -> Result<Self> {
        let file = fs::File::open(path)?;

        Ok(Self {
            inner: plist::Value::from_reader_xml(file)?,
        })
    }

    pub fn query(&self, query: &SysInfoQuery) -> Option<&Value> {
        let mut visiting = &self.inner;

        for segment in query.segments() {
            match (visiting, segment) {
                (Value::Dictionary(dict), QuerySegment::Key(key)) => visiting = dict.get(key)?,
                (Value::Array(array), QuerySegment::Index(index)) => {
                    visiting = array.get(*index)?
                }
                _ => return None,
            }
        }
        Some(visiting)
    }

    pub fn get_dict(&self, query: &SysInfoQuery) -> Option<&Dictionary> {
        self.query(query).and_then(Value::as_dictionary)
    }

    pub fn get_str(&self, query: &SysInfoQuery) -> Option<&str> {
        self.query(query).and_then(Value::as_string)
    }

    pub fn get_u64(&self, query: &SysInfoQuery) -> Option<u64> {
        self.query(query).and_then(Value::as_unsigned_integer)
    }

    pub fn get_u32(&self, query: &SysInfoQuery) -> Option<u32> {
        self.query(query)
            .and_then(Value::as_unsigned_integer)
            .and_then(|int| u32::try_from(int).ok())
    }

    pub fn get_bool(&self, query: &SysInfoQuery) -> Option<bool> {
        self.query(query).and_then(Value::as_boolean)
    }

    pub fn serial_number(&self) -> Option<&str> {
        self.get_str(&SysInfoQuery::with_key("SerialNumber"))
    }

    pub fn firewire_guid(&self) -> Option<&str> {
        self.get_str(&SysInfoQuery::with_key("FireWireGUID"))
    }

    pub fn audio_capabilities(&self) -> Vec<AudioCapability> {
        let mut audio_capabilities = Vec::new();

        if let Some(codec_dict) = self.get_dict(&SysInfoQuery::with_key("AudioCodecs")) {
            if let Some(Value::Dictionary(dict)) = codec_dict.get("AIFF") {
                audio_capabilities.push(AudioCapability::Aiff(Aiff::from(dict.clone())));
            }

            if let Some(Value::Dictionary(dict)) = codec_dict.get("MP3") {
                audio_capabilities.push(AudioCapability::Mp3(Mp3::from(dict.clone())));
            }

            if let Some(Value::Dictionary(dict)) = codec_dict.get("AAC") {
                audio_capabilities.push(AudioCapability::Aac(Aac::from(dict.clone())));
            }

            if let Some(Value::Dictionary(dict)) = codec_dict.get("AppleLossless") {
                audio_capabilities.push(AudioCapability::AppleLossless(AppleLossless::from(
                    dict.clone(),
                )));
            }

            if let Some(Value::Dictionary(dict)) = codec_dict.get("Audible") {
                audio_capabilities.push(AudioCapability::Audible(Audible::from(dict.clone())));
            }
        }
        audio_capabilities
    }
}

pub struct SysInfoQuery {
    buf: Vec<QuerySegment>,
}

impl SysInfoQuery {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    pub fn with_key(key: &str) -> Self {
        let mut query = SysInfoQuery::new();

        query.push_key(key);
        query
    }

    pub fn push_key(&mut self, key: &str) -> &mut Self {
        self.buf.push(QuerySegment::from(key));
        self
    }

    pub fn push_index(&mut self, index: usize) -> &mut Self {
        self.buf.push(QuerySegment::from(index));
        self
    }

    pub fn pop(&mut self) {
        self.buf.pop();
    }

    pub fn clear(&mut self) {
        self.buf.clear();
    }

    pub fn segments(&self) -> impl Iterator<Item = &QuerySegment> {
        self.buf.iter()
    }
}
