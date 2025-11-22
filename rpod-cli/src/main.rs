use rpod_core::error::Result;

fn enumerate() -> Result<()> {
    let device_handles = rpod_core::device::enumerate_device_mounts()?;
    let _ipod = rpod_core::device::iPod::try_load(&device_handles[0])?;

    Ok(())
}

fn main() -> Result<()> {
    enumerate()?;
    Ok(())
}
