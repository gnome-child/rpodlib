use std::{
    fmt::{Display, Formatter},
    time::SystemTime,
};

use chrono::{DateTime, TimeZone, Utc};

const HFS_EPOCH_DIFF: u64 = 2_082_844_800;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp {
    utc: DateTime<Utc>,
}

impl Timestamp {
    pub fn now() -> Self {
        Self { utc: Utc::now() }
    }
}

impl From<SystemTime> for Timestamp {
    fn from(value: SystemTime) -> Self {
        let utc = DateTime::<Utc>::from(value);

        Self { utc }
    }
}

impl From<u64> for Timestamp {
    fn from(value: u64) -> Self {
        let unix_secs = value.checked_sub(HFS_EPOCH_DIFF).unwrap_or(0);
        let utc = Utc.timestamp_opt(unix_secs as i64, 0).unwrap();

        Self { utc }
    }
}

impl From<u32> for Timestamp {
    fn from(value: u32) -> Self {
        let unix_secs = value.checked_sub(HFS_EPOCH_DIFF as u32).unwrap_or(0);
        let utc = Utc.timestamp_opt(unix_secs as i64, 0).unwrap();

        Self { utc }
    }
}

impl From<Timestamp> for SystemTime {
    fn from(value: Timestamp) -> Self {
        value.utc.into()
    }
}

impl From<Timestamp> for u64 {
    fn from(value: Timestamp) -> Self {
        let unix_secs = value.utc.timestamp();

        unix_secs as u64 + HFS_EPOCH_DIFF
    }
}

impl From<Timestamp> for u32 {
    fn from(value: Timestamp) -> Self {
        let unix_secs = value.utc.timestamp();

        (unix_secs as u64 + HFS_EPOCH_DIFF) as u32
    }
}

impl Display for Timestamp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.utc.to_rfc3339())
    }
}
