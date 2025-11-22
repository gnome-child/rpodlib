use std::{
    fs,
    os::windows::fs::OpenOptionsExt,
    path::{Path, PathBuf},
};

use windows::Win32::Storage::FileSystem::FILE_FLAG_DELETE_ON_CLOSE;

#[derive(Debug)]
pub struct LockFile {
    _file: fs::File,
    _path: PathBuf,
}

impl LockFile {
    pub fn open<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let _path = path.as_ref().to_path_buf();
        fs::create_dir_all(&_path.parent().expect("should exist"))?;

        let _file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .share_mode(0)
            .custom_flags(FILE_FLAG_DELETE_ON_CLOSE.0)
            .open(&path)?;

        Ok(Self { _file, _path })
    }
}
