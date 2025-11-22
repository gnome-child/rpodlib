use std::collections::BTreeMap;

use crate::error::{Error, Result};

pub const APPLE_VID: u16 = 0x05AC;
pub const IPOD_CLASSIC_PID: u16 = 0x1261;

pub const ALL_PROFILES: &[DeviceProfile] = &[
    // iPod Video — 5th Gen (2005)
    video::rev_0::WHITE_30GB,
    video::rev_0::BLACK_30GB,
    video::rev_0::WHITE_60GB,
    video::rev_0::BLACK_60GB,
    video::rev_0::U2_30GB,
    // iPod Video — 5.5th Gen / Enhanced (2006)
    video::rev_1::WHITE_30GB,
    video::rev_1::BLACK_30GB,
    video::rev_1::U2_30GB,
    video::rev_1::WHITE_80GB,
    video::rev_1::BLACK_80GB,
    // iPod Classic — 6th Gen launch (2007, 80/160 GB)
    classic::rev_0::SILVER_80GB,
    classic::rev_0::GRAY_80GB,
    classic::rev_0::SILVER_160GB,
    classic::rev_0::GRAY_160GB,
    // iPod Classic — 6.5th Gen (2008, 120 GB only)
    classic::rev_1::SILVER_120GB,
    classic::rev_1::GRAY_120GB,
    // iPod Classic — 7th Gen (2009, thin 160 GB)
    classic::rev_2::SILVER_160GB,
    classic::rev_2::GRAY_160GB,
];

pub mod video {
    use super::{Color, DevIdentifiers, DeviceProfile};

    // All iPod Video (5th gen line) use A1136
    const MODEL_A: &str = "A1136";
    const USB_PID: u16 = 0x1261;

    // Base ctor
    const fn video_cfg(
        hw_config: &'static str,
        alias: &'static str,
        revision: u16,
        color: Color,
        storage_gb: u32,
        serial_salts: &'static [&'static str],
    ) -> DeviceProfile {
        DeviceProfile {
            identifiers: DevIdentifiers {
                hw_config,
                model_num: MODEL_A,
                usb_pid: USB_PID,
                serial_salts,
            },
            name: "iPod Video",
            alias,
            generation: 5,
            revision,
            color,
            storage_gb,
        }
    }

    /// 2005 — iPod Video "5th Gen"
    pub mod rev_0 {
        use super::*;

        pub const WHITE_30GB: DeviceProfile = video_cfg(
            "A002",
            "iPod Video 5th Gen",
            0,
            Color::White,
            30,
            &["SZ9", "WEC", "WED", "WEG", "WEH", "WEL"],
        );

        pub const BLACK_30GB: DeviceProfile = video_cfg(
            "A146",
            "iPod Video 5th Gen",
            0,
            Color::Black,
            30,
            &["TXK", "TXM", "WEE", "WEF", "WEJ", "WEK"], // NOTE: "WEE" collides with 5.5G black
        );

        pub const WHITE_60GB: DeviceProfile = video_cfg(
            "A003",
            "iPod Video 5th Gen",
            0,
            Color::White,
            60,
            &["SZA", "SZU"],
        );

        pub const BLACK_60GB: DeviceProfile = video_cfg(
            "A147",
            "iPod Video 5th Gen",
            0,
            Color::Black,
            60,
            &["TXL", "TXN"],
        );

        pub const U2_30GB: DeviceProfile = video_cfg(
            "A452",
            "iPod Video 5th Gen (U2)",
            0,
            Color::U2BlackRed,
            30,
            &[],
        );
    }

    /// 2006 — iPod Video "5.5th Gen / Enhanced"
    pub mod rev_1 {
        use super::*;

        pub const WHITE_30GB: DeviceProfile = video_cfg(
            "A444",
            "iPod Video 5.5th Gen",
            1,
            Color::White,
            30,
            &["V9K", "V9L", "WU9"],
        );

        pub const BLACK_30GB: DeviceProfile = video_cfg(
            "A446",
            "iPod Video 5.5th Gen",
            1,
            Color::Black,
            30,
            &["VQM", "V9M", "V9N", "WEE"], // "WEE" collision with 5G black
        );

        pub const U2_30GB: DeviceProfile = video_cfg(
            "A664",
            "iPod Video 5.5th Gen (U2)",
            1,
            Color::U2BlackRed,
            30,
            &["W9G"],
        );

        pub const WHITE_80GB: DeviceProfile = video_cfg(
            "A448",
            "iPod Video 5.5th Gen",
            1,
            Color::White,
            80,
            &["V9P", "V9Q"],
        );

        pub const BLACK_80GB: DeviceProfile = video_cfg(
            "A450",
            "iPod Video 5.5th Gen",
            1,
            Color::Black,
            80,
            &["V9R", "V9S", "V95", "V96", "WUC"],
        );
    }
}

pub mod classic {
    use super::{Color, DevIdentifiers, DeviceProfile};

    // All iPod Classic (2007–2009) use A1238
    const MODEL_A: &str = "A1238";
    const USB_PID: u16 = 0x1261;

    const fn classic_cfg(
        hw_config: &'static str,
        alias: &'static str,
        revision: u16,
        color: Color,
        storage_gb: u32,
        serial_salts: &'static [&'static str],
    ) -> DeviceProfile {
        DeviceProfile {
            identifiers: DevIdentifiers {
                hw_config,
                model_num: MODEL_A,
                usb_pid: USB_PID,
                serial_salts,
            },
            name: "iPod Classic",
            alias,
            generation: 6,
            revision,
            color,
            storage_gb,
        }
    }

    /// 2007 — iPod Classic “6th Gen” (80/160 GB)
    pub mod rev_0 {
        use super::*;

        pub const SILVER_80GB: DeviceProfile = classic_cfg(
            "B029",
            "iPod Classic 6th Gen",
            0,
            Color::Silver,
            80,
            &["Y5N"],
        );

        pub const GRAY_80GB: DeviceProfile =
            classic_cfg("B147", "iPod Classic 6th Gen", 0, Color::Gray, 80, &["YMV"]);

        pub const SILVER_160GB: DeviceProfile = classic_cfg(
            "B145",
            "iPod Classic 6th Gen",
            0,
            Color::Silver,
            160,
            &["YMU"],
        );

        pub const GRAY_160GB: DeviceProfile = classic_cfg(
            "B150",
            "iPod Classic 6th Gen",
            0,
            Color::Gray,
            160,
            &["YMX"],
        );
    }

    /// 2008 — iPod Classic “6.5th Gen” (120 GB only)
    pub mod rev_1 {
        use super::*;

        pub const SILVER_120GB: DeviceProfile = classic_cfg(
            "B562",
            "iPod Classic 6.5th Gen",
            1,
            Color::Silver,
            120,
            &["2C5"],
        );

        pub const GRAY_120GB: DeviceProfile = classic_cfg(
            "B565",
            "iPod Classic 6.5th Gen",
            1,
            Color::Gray,
            120,
            &["2C7"],
        );
    }

    /// 2009 — iPod Classic “7th Gen” (thin 160 GB)
    pub mod rev_2 {
        use super::*;

        pub const SILVER_160GB: DeviceProfile = classic_cfg(
            "C293",
            "iPod Classic 7th Gen",
            2,
            Color::Silver,
            160,
            &["9ZS"],
        );

        pub const GRAY_160GB: DeviceProfile = classic_cfg(
            "C297",
            "iPod Classic 7th Gen",
            2,
            Color::Gray,
            160,
            &["9ZU"],
        );
    }
}

#[derive(Debug)]
pub enum Color {
    Black,
    White,
    Silver,
    Gray,
    U2BlackRed,
}

#[derive(Debug)]
pub struct DeviceProfile {
    pub identifiers: DevIdentifiers,
    pub name: &'static str,
    pub alias: &'static str,
    pub generation: u16,
    pub revision: u16,
    pub color: Color,
    pub storage_gb: u32,
}

#[derive(Debug)]
pub struct DevIdentifiers {
    pub hw_config: &'static str,
    pub model_num: &'static str,
    pub serial_salts: &'static [&'static str],
    pub usb_pid: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct ProfileQuery<'a> {
    pub hw_config: Option<&'a str>,
    pub model_num: Option<&'a str>,
    pub serial_num: Option<&'a str>,
    pub usb_pid: Option<u16>,
}

impl<'a> ProfileQuery<'a> {
    pub fn new() -> Self {
        Self {
            hw_config: None,
            model_num: None,
            usb_pid: None,
            serial_num: None,
        }
    }

    pub fn with_hw_config(&mut self, hw_config: &'a str) -> &mut Self {
        self.hw_config = Some(hw_config);
        self
    }

    pub fn with_model_num(&mut self, model_num: &'a str) -> &mut Self {
        self.model_num = Some(model_num);
        self
    }

    pub fn with_usb_pid(&mut self, usb_pid: u16) -> &mut Self {
        self.usb_pid = Some(usb_pid);
        self
    }

    pub fn with_serial_num(&mut self, serial_num: &'a str) -> &mut Self {
        self.serial_num = Some(serial_num);
        self
    }
}

pub fn find_profile(prof_query: &ProfileQuery) -> Result<&'static DeviceProfile> {
    let scored = profiles_scored(prof_query)?;

    if let Some((_, top)) = scored.iter().next_back() {
        if top.len() == 1 {
            Ok(top[0])
        } else {
            Err(Error::AmbiguousProfile { matches: top.len() })
        }
    } else {
        Err(Error::ProfileNotFound)
    }
}

fn profiles_scored(prof_query: &ProfileQuery) -> Result<BTreeMap<u8, Vec<&'static DeviceProfile>>> {
    let mut results: BTreeMap<u8, Vec<&DeviceProfile>> = BTreeMap::new();

    for profile in ALL_PROFILES {
        let score = score_profile(profile, prof_query);

        if score > 0 {
            results.entry(score).or_default().push(profile);
        }
    }

    if results.is_empty() {
        Err(Error::ProfileNotFound)
    } else {
        Ok(results)
    }
}

fn score_profile(profile: &DeviceProfile, prof_query: &ProfileQuery) -> u8 {
    let mut score = 0;

    if let Some(hw_config) = prof_query.hw_config {
        if profile.identifiers.hw_config == hw_config {
            score += 1;
        }
    }

    if let Some(model_num) = prof_query.model_num {
        if profile.identifiers.model_num == model_num {
            score += 1;
        }
    }

    if let Some(usb_pid) = prof_query.usb_pid {
        if profile.identifiers.usb_pid == usb_pid {
            score += 1;
        }
    }

    if let Some(serial_num) = prof_query.serial_num {
        let needle = &serial_num[serial_num.len() - 3..];

        if profile
            .identifiers
            .serial_salts
            .iter()
            .any(|&salt| salt == needle)
        {
            score += 1;
        }
    }
    score
}
