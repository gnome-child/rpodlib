pub mod device_enum;
pub mod lockfile;
pub mod volume;

pub type Result<T> = windows::core::Result<T>;
pub type Error = windows::core::Error;
