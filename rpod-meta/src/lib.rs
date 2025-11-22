use std::{
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

use crate::{descriptive::Tags, error::Result, technical::FormatInfo};

pub mod descriptive;
pub mod error;
pub mod technical;

pub struct ProbeOptions {
    pub ignore_properties: bool,
    pub ignore_tags: bool,
    pub ignore_artwork: bool,
}

impl Default for ProbeOptions {
    fn default() -> Self {
        Self {
            ignore_properties: false,
            ignore_tags: false,
            ignore_artwork: false,
        }
    }
}

impl ProbeOptions {
    pub fn ignore_properties(mut self) -> Self {
        self.ignore_properties = true;
        self
    }

    pub fn ignore_tags(mut self) -> Self {
        self.ignore_tags = true;
        self
    }

    pub fn ignore_artwork(mut self) -> Self {
        self.ignore_artwork = true;
        self
    }
}

#[derive(Debug)]
pub struct Manifest {
    pub path: PathBuf,
    pub name: String,
    pub ext: String,
    pub size: u64,
    pub last_modified: SystemTime,
    pub format_info: FormatInfo,
    pub tags: Tags,
}

impl Manifest {
    pub fn from_path<P: AsRef<Path>>(path: P, probe_opts: &ProbeOptions) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let fs_meta = fs::metadata(&path)?;
        let name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .replace(".", " -");
        let ext = path
            .extension()
            .map(|os_str| os_str.to_string_lossy().to_string())
            .unwrap_or_default();
        let size = fs_meta.len();
        let last_modified = fs_meta.modified()?;
        let format_info = FormatInfo::probe(&path)?;
        let tags = Tags::probe(&path, &probe_opts)?;

        Ok(Self {
            path,
            name,
            ext,
            size,
            last_modified,
            format_info,
            tags,
        })
    }
}
