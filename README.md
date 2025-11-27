# rpodlib

Rust workspace for working with classic iPods: detect connected devices, read and rewrite their databases, probe audio files, and transcode to formats the device supports. The project is currently under development—back up your iPod and iTunesDB before experimenting. Linux-specific device handling is not implemented yet.

## Workspace layout
- **rpod-core**: Facade crate that re-exports the other members for easier consumption.
- **rpod-device**: Discovers mounted iPods, reads device metadata (serial, FireWire GUID, model profile), manages iTunes lockfiles, and loads the iTunesDB for mutation.
- **rpod-itdb**: Binary reader/writer for iTunesDB records (tracks, playlists, podcasts, hashing seeds, etc.) with helpers to commit changes back to disk.
- **rpod-meta**: Probes audio files to extract codec details, tags, and artwork so you can decide whether transcoding is needed.
- **rpod-transcode**: Plans and executes transcoding to AAC/ALAC/MP3 or uncompressed targets based on a device's supported capabilities.
- **rpod-display**: Simple progress formatting utilities used during long-running tasks.
- **rpod-cli**: Minimal example that enumerates connected iPods and tries to load the first one.

## Key capabilities
- Enumerate device mount points, infer the iPod model/profile, and open a locked session so database writes stay safe.
- Parse existing iTunesDB data, insert new tracks, and commit changes along with required hash seeds and lockfile handling.
- Inspect audio files for codecs, tags, artwork, and durations before syncing.
- Plan and run transcoding jobs when a track is incompatible with the target device's audio capabilities.

## Supported iPods
- **iPod Video (5th gen, 2005):** White/Black 30 GB, White/Black 60 GB, U2 30 GB variants.
- **iPod Video (5.5th gen, 2006):** White/Black 30 GB, U2 30 GB, White/Black 80 GB variants.
- **iPod Classic (6th gen, 2007):** Silver/Gray 80 GB, Silver/Gray 160 GB variants.
- **iPod Classic (6.5th gen, 2008):** Silver/Gray 120 GB variants.
- **iPod Classic (7th gen, 2009):** Silver/Gray 160 GB (thin) variants.

## Quick start
1. Install the Rust toolchain (edition 2024).
2. Build the workspace:
   ```bash
   cargo build
   ```
3. Try the sample CLI (requires a mounted iPod):
   ```bash
   cargo run -p rpod-cli
   ```

## Using the core API
Basic pattern for loading a device and printing its model string:
```rust
use rpod_core::device::{enumerate_device_mounts, iPod};

fn main() -> rpod_core::error::Result<()> {
    let mut handles = enumerate_device_mounts()?;
    let ipod = iPod::try_load(&handles.remove(0))?;
    println!("{}", ipod); // e.g., "[A1234] iPod classic - 6th Generation"
    Ok(())
}
```

## Tests
Some tests expect audio fixtures and (optionally) a connected iPod. See `rpod-core/tests/fixtures` for details before running `cargo test`.
