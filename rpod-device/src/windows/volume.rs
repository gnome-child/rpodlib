use std::{os::windows::ffi::OsStrExt, path::Path};

use windows::{Win32::Storage::FileSystem::GetDiskFreeSpaceExW, core::PCWSTR};

use crate::VolumeInfo;

use super::Result;

impl VolumeInfo {
    pub fn from_path(path: &Path) -> Result<VolumeInfo> {
        let mut wide_str: Vec<u16> = path.as_os_str().encode_wide().collect();

        if !wide_str.ends_with(&[0]) {
            wide_str.push(0);
        }

        let mut volume_info = VolumeInfo {
            free_available: 0,
            total_capacity: 0,
            total_free: 0,
        };

        unsafe {
            GetDiskFreeSpaceExW(
                PCWSTR(wide_str.as_ptr()),
                Some(&mut volume_info.free_available),
                Some(&mut volume_info.total_capacity),
                Some(&mut volume_info.total_free),
            )?
        };
        Ok(volume_info)
    }
}
