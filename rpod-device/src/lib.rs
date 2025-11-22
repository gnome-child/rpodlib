#![allow(non_camel_case_types)]

use std::{
    collections::BTreeSet,
    fmt::Display,
    fs,
    io::Cursor,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};

use binrw::BinRead;
use rpod_itdb::{Itdb, Track, hash::Seeds, root::Root};

use crate::{
    capabilities::audio::AudioCapability,
    error::{Error, Result},
    model::{ALL_PROFILES, APPLE_VID, DeviceProfile, ProfileQuery, find_profile},
    platform::device_enum::enumerate_mount_points,
    platform::lockfile::LockFile,
    sys_info::SystemInfo,
};

pub mod capabilities;
pub mod error;
pub mod model;
pub mod sys_info;

#[cfg(target_os = "linux")]
#[path = "linux/mod.rs"]
pub mod platform;

#[cfg(target_os = "windows")]
#[path = "windows/mod.rs"]
pub mod platform;

pub struct iPod {
    inner: iPodDevice,
}

impl Deref for iPod {
    type Target = iPodDevice;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl iPod {
    pub fn try_load(handle: &DeviceHandle) -> Result<Self> {
        Ok(Self {
            inner: iPodDevice::try_load(handle)?,
        })
    }

    pub fn lock(&mut self) -> Result<LockGuard<'_>> {
        if !fs::exists(self.file_sys.lockfile_path())? {
            let lockfile = LockFile::open(&self.inner.file_sys.lockfile_path())?;

            Ok(LockGuard {
                inner: &mut self.inner,
                _lockfile: lockfile,
            })
        } else {
            Err(Error::LockFileExists)
        }
    }
}

pub struct LockGuard<'a> {
    inner: &'a mut iPodDevice,
    _lockfile: LockFile,
}

impl<'a> Deref for LockGuard<'a> {
    type Target = iPodDevice;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a> DerefMut for LockGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}

impl<'a> LockGuard<'a> {
    pub fn insert_track(&mut self, track: Track) -> Result<()> {
        Ok(self.database.insert_track(track)?)
    }

    pub fn commit(&mut self) -> Result<()> {
        self.database.commit()?;

        Ok(self.database.write_to(
            &self.file_sys.itunesdb_path(),
            &self.fw_guid,
            &self.hash_seeds,
        )?)
    }
}

pub struct iPodDevice {
    pub file_sys: FileSystem,
    pub sys_info: SystemInfo,
    pub profile: &'static DeviceProfile,
    pub database: Itdb,
    fw_guid: [u8; 8],
    hash_seeds: Seeds,
}

impl iPodDevice {
    pub fn try_load(device_handle: &DeviceHandle) -> Result<Self> {
        let device_handle = device_handle.clone();
        let volume_info = VolumeInfo::from_path(&device_handle.mount_point)?;
        let file_sys = FileSystem::new(device_handle, volume_info);
        let sys_info = SystemInfo::parse(&file_sys.sys_info_path())?;

        let mut prof_query = ProfileQuery::new();
        prof_query.with_usb_pid(file_sys.device_handle.usb_pid);

        if let Some(serial_num) = sys_info.serial_number() {
            prof_query.with_serial_num(serial_num);
        }
        let profile = find_profile(&prof_query)?;

        let bytes = fs::read(file_sys.itunesdb_path())?;
        let hash_seeds = Seeds::extract(&bytes)?;
        let mut cursor = Cursor::new(bytes);
        let root = Root::read(&mut cursor)?;
        let database = Itdb::try_from(root)?;
        let firewire_guid = sys_info.firewire_guid().ok_or(Error::MissingFireWireGUID)?;
        let mut fw_guid = [0u8; 8];

        for (index, chunk) in firewire_guid.as_bytes().chunks(2).enumerate() {
            fw_guid[index] = u8::from_str_radix(str::from_utf8(chunk).expect("bad fw_guid"), 16)
                .unwrap_or_default();
        }

        Ok(Self {
            file_sys,
            sys_info,
            profile,
            database,
            hash_seeds,
            fw_guid,
        })
    }

    pub fn model_string(&self) -> String {
        fn ordinal(int: u16) -> String {
            let suffix = match int {
                0 => "",
                1 => "st",
                2 => "nd",
                3 => "rd",
                _ => "th",
            };
            format!("{}{}", int, suffix)
        }

        let revision = if self.profile.revision > 0 {
            format!(", {} revision", ordinal(self.profile.revision))
        } else {
            "".to_string()
        };

        format!(
            "[{}] {} - {} Generation{} ({:?}, {} GB)",
            self.profile.identifiers.model_num,
            self.profile.name,
            ordinal(self.profile.generation),
            revision,
            self.profile.color,
            self.profile.storage_gb,
        )
    }

    pub fn serial_number(&self) -> Option<&str> {
        self.sys_info.serial_number()
    }

    pub fn firewire_guid(&self) -> Option<&str> {
        self.sys_info.firewire_guid()
    }

    pub fn audio_capabilities(&self) -> Vec<AudioCapability> {
        self.sys_info.audio_capabilities()
    }
}

impl Display for iPod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.model_string())
    }
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct DeviceHandle {
    pub usb_pid: u16,
    pub mount_point: PathBuf,
}

impl DeviceHandle {
    pub fn new(usb_pid: u16, mount_point: &Path) -> Self {
        let mount_point = mount_point.to_path_buf();

        Self {
            usb_pid,
            mount_point,
        }
    }
}

pub struct VolumeInfo {
    pub free_available: u64,
    pub total_capacity: u64,
    pub total_free: u64,
}

pub fn enumerate_device_mounts() -> Result<Vec<DeviceHandle>> {
    let vid = APPLE_VID;
    let unique_pids: BTreeSet<u16> = ALL_PROFILES
        .iter()
        .map(|profile| profile.identifiers.usb_pid)
        .collect();

    let mut dev_handles = Vec::new();

    for pid in unique_pids {
        let mount_points = enumerate_mount_points(vid, pid)?;

        for mount_point in mount_points {
            dev_handles.push(DeviceHandle::new(pid, &mount_point));
        }
    }
    Ok(dev_handles)
}

pub struct FileSystem {
    device_handle: DeviceHandle,
    volume_info: VolumeInfo,
}

impl FileSystem {
    // iPod directories
    const IPOD_CONTROL: &'static str = "iPod_Control";
    const DEVICE: &'static str = "Device";
    const ITUNES: &'static str = "iTunes";
    const ARTWORK: &'static str = "Artwork";
    const MUSIC: &'static str = "Music";

    pub fn new(device_handle: DeviceHandle, volume_info: VolumeInfo) -> Self {
        Self {
            device_handle,
            volume_info,
        }
    }

    pub fn capacity(&self) -> u64 {
        self.volume_info.total_capacity
    }

    pub fn free_space(&self) -> u64 {
        self.volume_info.total_free
    }

    pub fn mount_point(&self) -> &Path {
        &self.device_handle.mount_point
    }

    pub fn ipod_control(&self) -> PathBuf {
        self.mount_point().join(Self::IPOD_CONTROL)
    }

    pub fn device_folder(&self) -> PathBuf {
        self.ipod_control().join(Self::DEVICE)
    }

    pub fn itunes_folder(&self) -> PathBuf {
        self.ipod_control().join(Self::ITUNES)
    }

    pub fn music_folder(&self) -> PathBuf {
        self.ipod_control().join(Self::MUSIC)
    }

    pub fn artwork_folder(&self) -> PathBuf {
        self.ipod_control().join(Self::ARTWORK)
    }

    pub fn sys_info_path(&self) -> PathBuf {
        self.device_folder().join("ExtendedSysInfoXml")
    }

    pub fn itunesdb_path(&self) -> PathBuf {
        self.itunes_folder().join("iTunesDB")
    }

    pub fn lockfile_path(&self) -> PathBuf {
        self.itunes_folder().join("iTunesLock")
    }
}
