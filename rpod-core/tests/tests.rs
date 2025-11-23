use std::{
    fs,
    io::Cursor,
    path::{Path, PathBuf},
    str::FromStr,
    sync::OnceLock,
};

use binrw::{BinRead, BinWrite};
use rpod_device::{DeviceHandle, iPod};
use rpod_display::fmt::{TreeContext, TreeDisplayExt};
use rpod_itdb::{
    SizeRange, Track,
    hash::{Hasher, Seeds},
    list::List,
    root::Root,
    track_item::TrackItem,
};
use rpod_meta::{Manifest, ProbeOptions};
use rpod_transcode::{Outcome, auto_transcode};
use thiserror::Error;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Binrw(#[from] binrw::Error),

    #[error(transparent)]
    Meta(#[from] rpod_meta::error::Error),

    #[error(transparent)]
    Device(#[from] rpod_device::error::Error),

    #[error(transparent)]
    Transcode(#[from] rpod_transcode::error::Error),

    #[error(transparent)]
    Database(#[from] rpod_itdb::error::Error),
}

static TEST_DIR: OnceLock<PathBuf> = OnceLock::new();

fn test_dir() -> &'static Path {
    TEST_DIR
        .get_or_init(|| {
            PathBuf::from_str(env!("CARGO_MANIFEST_DIR"))
                .expect("CARGO_MANIFEST_DIR missing")
                .join("tests")
        })
        .as_path()
}

fn fixtures() -> PathBuf {
    test_dir().join("fixtures")
}

fn audio_files() -> PathBuf {
    fixtures().join("audio_files")
}

fn out_dir() -> PathBuf {
    test_dir().join("out")
}

fn fake_ipod_mount() -> PathBuf {
    fixtures().join("fake_ipod")
}

fn test_audio_files() -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();

    for entry in fs::read_dir(audio_files())? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            paths.push(path);
        }
    }
    Ok(paths)
}

#[test]
fn dump_db() -> Result<()> {
    let mut device_handles = rpod_core::device::enumerate_device_mounts()?;

    if device_handles.is_empty() {
        println!("No supported iPods found, falling back to mock ipod...");

        let dev_handle = DeviceHandle::new(0x1261, &fake_ipod_mount());
        device_handles.push(dev_handle);
    }

    let mut ipod = iPod::try_load(&device_handles[0])?;
    println!("loaded: {}", ipod.model_string());

    std::fs::copy(
        ipod.file_sys.itunesdb_path(),
        ipod.file_sys.itunesdb_path().with_extension("bak"),
    )?;

    // let raw_db = ipod.database.as_raw();

    // for set in &raw_db.list_containers {
    //     println!("{:#?}", set.list_type);

    //     match &set.list {
    //         List::TrackItems(list) => println!("  {} items", list.items.len()),
    //         List::LibraryPlaylists(list) => println!("  {} items", list.items.len()),
    //         List::AlbumItems(list) => println!("  {} items", list.items.len()),
    //         List::PodcastFmtLibPlaylists(list) => println!("  {} items", list.items.len()),
    //         List::SpecialPlaylists(list) => println!("  {} items", list.items.len()),
    //     }
    // }

    let tree_context = TreeContext::begin_unicode();

    println!(
        "{:#}",
        ipod.database.as_raw().to_tree_string_pretty(tree_context)
    );

    let mut ipod_locked = ipod.lock()?;

    // let capabilities = ipod_locked.audio_capabilities();
    // let music_dir = ipod_locked.file_sys.music_folder();
    // let audio_file = test_audio_files()?[0].clone();
    // let manifest = Manifest::from_path(&audio_file, &ProbeOptions::default())?;
    // let outcome = auto_transcode(&manifest, &capabilities, &music_dir)?;
    // let track_item = match outcome {
    //     Outcome::Transcoded {
    //         out_path,
    //         operations,
    //         ..
    //     } => {
    //         println!("transcoded:");

    //         for op in operations {
    //             println!("  {}", op);
    //         }

    //         let out_manifest = Manifest::from_path(&out_path, &ProbeOptions::default())?;
    //         TrackItem::from_manifest(&out_manifest)
    //     }
    //     Outcome::Skipped { reason } => {
    //         println!("skipped: {}", reason);
    //         TrackItem::from_manifest(&manifest)
    //     }
    // };
    // ipod_locked.insert_track(Track::from(&track_item))?;
    ipod_locked.commit()?;

    let mut device_handles = rpod_core::device::enumerate_device_mounts()?;

    if device_handles.is_empty() {
        println!("No supported iPods found, falling back to mock ipod...");

        let dev_handle = DeviceHandle::new(0x1261, &fake_ipod_mount());
        device_handles.push(dev_handle);
    }

    let ipod = iPod::try_load(&device_handles[0])?;
    // let raw_db = ipod.database.as_raw();

    // for set in &raw_db.list_containers {
    //     println!("{:#?}", set.list_type);

    //     match &set.list {
    //         List::TrackItems(list) => println!("  {} items", list.items.len()),
    //         List::LibraryPlaylists(list) => println!("  {} items", list.items.len()),
    //         List::AlbumItems(list) => println!("  {} items", list.items.len()),
    //         List::PodcastFmtLibPlaylists(list) => println!("  {} items", list.items.len()),
    //         List::SpecialPlaylists(list) => println!("  {} items", list.items.len()),
    //     }
    // }
    let tree_context = TreeContext::begin_unicode();

    println!(
        "{:#}",
        ipod.database.as_raw().to_tree_string_pretty(tree_context)
    );

    // fs::remove_file(ipod_locked.file_sys.itunesdb_path())?;
    // fs::rename(
    //     ipod_locked.file_sys.itunesdb_path().with_extension("bak"),
    //     ipod_locked.file_sys.itunesdb_path(),
    // )?;
    Ok(())
}

// #[test]
fn compare_hash() -> Result<()> {
    let mut device_handles = rpod_core::device::enumerate_device_mounts()?;

    if device_handles.is_empty() {
        println!("No supported iPods found, falling back to mock ipod...");

        let dev_handle = DeviceHandle::new(0x1261, &fake_ipod_mount());
        device_handles.push(dev_handle);
    }

    for handle in device_handles {
        let ipod = iPod::try_load(&handle)?;

        println!("found {}", ipod.model_string());

        let bytes = fs::read(ipod.file_sys.itunesdb_path())?;
        let mut cursor = Cursor::new(bytes);
        let root = Root::read(&mut cursor)?;

        let old_58 = root.hash_0x58;
        let old_72 = root.hash_0x72;

        let firewire_guid = ipod.sys_info.firewire_guid().ok_or(Error::Device(
            rpod_device::error::Error::MissingFireWireGUID,
        ))?;
        let mut seed = [0u8; 8];

        for (index, chunk) in firewire_guid.as_bytes().chunks(2).enumerate() {
            seed[index] =
                u8::from_str_radix(str::from_utf8(chunk).unwrap(), 16).unwrap_or_default();
        }

        let mut cursor = Cursor::new(Vec::with_capacity(root.total_bytes_len() as usize));
        root.write(&mut cursor)?;

        let mut buf = cursor.into_inner();
        let seeds = Seeds::extract(&buf)?;
        let hasher = Hasher::from_bytes(&mut buf, &seeds)?;
        hasher.hash(&seed)?;

        let mut new_58 = [0u8; 20];
        new_58.copy_from_slice(&buf[0x58..0x6C]);

        let mut new_72 = [0u8; 46];
        new_72.copy_from_slice(&buf[0x72..0xA0]);

        if old_58 != new_58 {
            println!("hash58 mismatch!");
            println!("  old: {:02X?}", old_58);
            println!("  new: {:02X?}", new_58);
            panic!("hash58 did not round-trip");
        }

        if old_72 != new_72 {
            println!("hash72 mismatch!");
            println!("  old: {:02X?}", old_72);
            println!("  new: {:02X?}", new_72);
            panic!("hash72 did not round-trip");
        }
        println!("hashes match for {}", ipod.model_string());
    }
    Ok(())
}

// #[test]
fn make_new_db() -> Result<()> {
    let device_handles = rpod_core::device::enumerate_device_mounts()?;

    if device_handles.is_empty() {
        println!("No supported iPods found");
        Ok(())
    } else {
        for handle in device_handles {
            let mut ipod = iPod::try_load(&handle)?;
            let mut ipod_locked = ipod.lock()?;

            println!("found {}", ipod_locked.model_string());

            std::fs::copy(
                ipod_locked.file_sys.itunesdb_path(),
                ipod_locked.file_sys.itunesdb_path().with_extension("bak"),
            )?;

            let capabilities = ipod_locked.audio_capabilities();
            let music_dir = ipod_locked.file_sys.music_folder();
            let audio_file = test_audio_files()?[0].clone();
            let manifest = Manifest::from_path(&audio_file, &ProbeOptions::default())?;
            let outcome = auto_transcode(&manifest, &capabilities, &music_dir)?;
            let track_item = match outcome {
                Outcome::Transcoded {
                    out_path,
                    operations,
                    ..
                } => {
                    println!("transcoded:");

                    for op in operations {
                        println!("  {}", op);
                    }

                    let out_manifest = Manifest::from_path(&out_path, &ProbeOptions::default())?;
                    TrackItem::from_manifest(&out_manifest)
                }
                Outcome::Skipped { reason } => {
                    println!("skipped: {}", reason);
                    TrackItem::from_manifest(&manifest)
                }
            };
            let tree_context = TreeContext::begin_unicode();

            println!("{:#}", track_item.to_tree_string_pretty(tree_context));
            ipod_locked.insert_track(Track::from(&track_item))?;
            ipod_locked.commit()?;
        }
        Ok(())
    }
}

// #[test]
fn test_make_track_items() -> Result<()> {
    let test_audio_files = test_audio_files()?;
    let dev_handle = DeviceHandle::new(0x1261, &fake_ipod_mount());
    let mut fake_ipod = iPod::try_load(&dev_handle)?;
    let capabilities = fake_ipod.audio_capabilities();

    println!("iPod: {}", fake_ipod);
    let mut guard = fake_ipod.lock()?;

    for path in test_audio_files {
        let manifest = Manifest::from_path(&path, &ProbeOptions::default())?;

        let outcome = auto_transcode(&manifest, &capabilities, &out_dir())?;
        let track_item;

        match outcome {
            Outcome::Transcoded { out_path, .. } => {
                let out_manifest = Manifest::from_path(&out_path, &ProbeOptions::default())?;
                track_item = TrackItem::from_manifest(&out_manifest);
                let tree_context = TreeContext::begin_unicode();

                println!("{:#}", track_item.to_tree_string_pretty(tree_context));
            }
            Outcome::Skipped { reason } => {
                track_item = TrackItem::from_manifest(&manifest);
                let tree_context = TreeContext::begin_unicode();

                println!("Skipped transcoding: {}", reason);
                println!("{:#}", track_item.to_tree_string_pretty(tree_context));
            }
        }

        guard.insert_track(Track::from(track_item))?;
    }
    guard.commit()?;

    let bytes = fs::read(fake_ipod.file_sys.itunes_folder().join("iTunesDB-new"))?;
    let mut cursor = std::io::Cursor::new(bytes);
    let database = rpod_itdb::root::Root::read(&mut cursor).unwrap();
    let tree_context = TreeContext::begin_unicode();
    println!("{}", database.to_tree_string_pretty(tree_context));

    Ok(())
}
