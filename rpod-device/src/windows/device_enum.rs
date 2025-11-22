use std::path::PathBuf;

use regex::Regex;

use windows::{
    Win32::{
        Devices::DeviceAndDriverInstallation::{
            CM_Get_Device_ID_Size, CM_Get_Device_IDW, CM_Get_Parent, CR_SUCCESS,
            DIGCF_DEVICEINTERFACE, DIGCF_PRESENT, HDEVINFO, SP_DEVICE_INTERFACE_DATA,
            SP_DEVICE_INTERFACE_DETAIL_DATA_W, SP_DEVINFO_DATA, SetupDiDestroyDeviceInfoList,
            SetupDiEnumDeviceInterfaces, SetupDiGetClassDevsW, SetupDiGetDeviceInterfaceDetailW,
        },
        Foundation::{CloseHandle, ERROR_INSUFFICIENT_BUFFER, ERROR_NO_MORE_ITEMS},
        Storage::FileSystem::{
            CreateFileW, FILE_FLAG_BACKUP_SEMANTICS, FILE_SHARE_READ, FILE_SHARE_WRITE,
            OPEN_EXISTING,
        },
        System::{
            IO::DeviceIoControl,
            Ioctl::{
                GUID_DEVINTERFACE_DISK, IOCTL_STORAGE_GET_DEVICE_NUMBER, STORAGE_DEVICE_NUMBER,
            },
        },
    },
    core::PCWSTR,
};

use super::{Error, Result};

pub fn enumerate_mount_points(vid: u16, pid: u16) -> Result<Vec<PathBuf>> {
    let set = open_interface_set()?;

    let mut mount_points = Vec::new();

    for interface in iter_disk_interfaces(&set) {
        let interface = interface?;
        let (cur_vid, cur_pid) = get_vid_pid(&set, &interface)?;

        if cur_vid != vid || cur_pid != pid {
            continue;
        }

        let path = get_device_path(&set, &interface)?;
        let (dev_num, part_num) = get_device_numbers(&path)?;

        let letter = if part_num == 0 {
            (1..=16).find_map(|part_num| get_letter_for_dev(dev_num, part_num))
        } else {
            get_letter_for_dev(dev_num, part_num)
        };

        if let Some(letter) = letter {
            mount_points.push(std::path::PathBuf::from(format!("{}:\\", letter)));
        }
    }
    Ok(mount_points)
}

struct DeviceInfoSet(HDEVINFO);

impl Drop for DeviceInfoSet {
    fn drop(&mut self) {
        unsafe {
            SetupDiDestroyDeviceInfoList(self.0).ok();
        }
    }
}

fn open_interface_set() -> Result<DeviceInfoSet> {
    unsafe {
        SetupDiGetClassDevsW(
            Some(&GUID_DEVINTERFACE_DISK),
            None,
            None,
            DIGCF_PRESENT | DIGCF_DEVICEINTERFACE,
        )
    }
    .map(DeviceInfoSet)
}

fn iter_disk_interfaces(
    set: &DeviceInfoSet,
) -> impl Iterator<Item = Result<SP_DEVICE_INTERFACE_DATA>> + '_ {
    (0u32..)
        .map(move |index| {
            let mut data = SP_DEVICE_INTERFACE_DATA::default();
            data.cbSize = size_of::<SP_DEVICE_INTERFACE_DATA>() as u32;

            unsafe {
                SetupDiEnumDeviceInterfaces(set.0, None, &GUID_DEVINTERFACE_DISK, index, &mut data)
            }
            .map(|_| data)
        })
        .take_while(|r| !matches!(r, Err(e) if e.code() == ERROR_NO_MORE_ITEMS.into()))
}

fn get_device_path(set: &DeviceInfoSet, interface: &SP_DEVICE_INTERFACE_DATA) -> Result<Vec<u16>> {
    let mut req_size = 0u32;

    unsafe {
        let _ =
            SetupDiGetDeviceInterfaceDetailW(set.0, interface, None, 0, Some(&mut req_size), None);
    }

    let mut buf = vec![0u32; req_size as usize];
    let detail_ptr = buf.as_mut_ptr() as *mut SP_DEVICE_INTERFACE_DETAIL_DATA_W;

    unsafe {
        (*detail_ptr).cbSize = if cfg!(target_pointer_width = "64") {
            8
        } else {
            6
        };
    }

    unsafe {
        SetupDiGetDeviceInterfaceDetailW(set.0, interface, Some(detail_ptr), req_size, None, None)?;
    }

    let wide_ptr = unsafe { (*detail_ptr).DevicePath.as_ptr() };

    let len = (0..)
        .take_while(|&i| unsafe { *wide_ptr.add(i) != 0 })
        .count();

    let owned = unsafe { core::slice::from_raw_parts(wide_ptr, len + 1) }.to_vec();

    Ok(owned)
}

fn get_vid_pid(set: &DeviceInfoSet, iface: &SP_DEVICE_INTERFACE_DATA) -> Result<(u16, u16)> {
    let mut dev_info_data: SP_DEVINFO_DATA = unsafe { std::mem::zeroed() };
    dev_info_data.cbSize = size_of::<SP_DEVINFO_DATA>() as u32;

    match unsafe {
        SetupDiGetDeviceInterfaceDetailW(set.0, iface, None, 0, None, Some(&mut dev_info_data))
    } {
        Err(e) if e.code() == ERROR_INSUFFICIENT_BUFFER.into() => {} // Expected
        Err(e) => return Err(e),
        Ok(_) => {}
    }

    let mut parent = 0u32;
    let cr = unsafe { CM_Get_Parent(&mut parent, dev_info_data.DevInst, 0) };

    if cr != CR_SUCCESS {
        return Err(windows::core::Error::from_win32());
    }

    let mut id_size = 0u32;
    let cr = unsafe { CM_Get_Device_ID_Size(&mut id_size, parent, 0) };

    if cr != CR_SUCCESS {
        return Err(windows::core::Error::from_win32());
    }

    let mut id_buf = vec![0u16; (id_size + 1) as usize]; // +1 for null
    let cr = unsafe { CM_Get_Device_IDW(parent, &mut id_buf, 0) };

    if cr != CR_SUCCESS {
        return Err(Error::from_win32());
    }

    let id = String::from_utf16_lossy(&id_buf)
        .trim_end_matches('\0')
        .to_ascii_lowercase();
    let regex = Regex::new(r"vid_([0-9a-f]{4})&pid_([0-9a-f]{4})").unwrap();

    if let Some(caps) = regex.captures(&id) {
        let vid = u16::from_str_radix(&caps[1], 16).unwrap_or(0);
        let pid = u16::from_str_radix(&caps[2], 16).unwrap_or(0);

        Ok((vid, pid))
    } else {
        Ok((0, 0)) // Or Err if no match is failure
    }
}

fn get_device_numbers(path_utf16: &[u16]) -> Result<(u32, u32)> {
    let handle = unsafe {
        CreateFileW(
            PCWSTR(path_utf16.as_ptr()),
            0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS,
            None,
        )
    }?;

    if handle.is_invalid() {
        return Err(windows::core::Error::from_win32());
    }

    let mut devnum = STORAGE_DEVICE_NUMBER::default();
    let out_len = std::mem::size_of::<STORAGE_DEVICE_NUMBER>() as u32;
    let mut bytes_ret = 0u32;
    let out_ptr: *mut std::ffi::c_void = &mut devnum as *mut _ as *mut std::ffi::c_void;

    unsafe {
        DeviceIoControl(
            handle,
            IOCTL_STORAGE_GET_DEVICE_NUMBER,
            None,
            0,
            Some(out_ptr),
            out_len,
            Some(&mut bytes_ret),
            None,
        )?;
    }

    unsafe {
        CloseHandle(handle)?;
    }
    Ok((devnum.DeviceNumber, devnum.PartitionNumber))
}

fn get_letter_for_dev(dev: u32, part: u32) -> Option<char> {
    for letter in b'A'..=b'Z' {
        let mut name = [0u16; 7]; // "\\?\X:\0"

        name[0] = '\\' as u16;
        name[1] = '\\' as u16;
        name[2] = '?' as u16;
        name[3] = '\\' as u16;
        name[4] = letter as u16;
        name[5] = ':' as u16;

        let handle = unsafe {
            CreateFileW(
                PCWSTR(name.as_ptr()),
                0,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                None,
                OPEN_EXISTING,
                FILE_FLAG_BACKUP_SEMANTICS,
                None,
            )
        };

        if handle.is_err() {
            continue;
        }

        let handle = handle.unwrap();

        if handle.is_invalid() {
            continue;
        }

        let mut num = STORAGE_DEVICE_NUMBER::default();
        let mut ret = 0u32;

        let ok = unsafe {
            windows::Win32::System::IO::DeviceIoControl(
                handle,
                IOCTL_STORAGE_GET_DEVICE_NUMBER,
                None,
                0,
                Some(&mut num as *mut _ as *mut std::ffi::c_void),
                std::mem::size_of::<STORAGE_DEVICE_NUMBER>() as u32,
                Some(&mut ret),
                None,
            )
        }
        .is_ok();

        unsafe {
            CloseHandle(handle).ok();
        }

        if ok && num.DeviceNumber == dev && num.PartitionNumber == part {
            return Some(letter as char);
        }
    }
    None
}
