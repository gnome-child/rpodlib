use std::{fs, path::Path};

use rpod_display::Progress;
use symphonia::{
    core::{
        codecs::{CODEC_TYPE_NULL, DecoderOptions},
        formats::FormatOptions,
        io::{MediaSourceStream, MediaSourceStreamOptions},
        meta::{Limit, MetadataOptions},
        probe::Hint,
    },
    default,
};

use crate::{
    encoder::Encoder,
    error::{Error, Result},
};

pub fn decode_into<P: AsRef<Path>, E: Encoder>(
    src_path: P,
    mut encoder: E,
    mut progress: Option<&mut dyn Progress>,
) -> Result<u64> {
    use symphonia::core::errors::Error as SymphoniaError;

    let src_path = src_path.as_ref();
    let file = fs::File::open(&src_path)?;
    let stream = MediaSourceStream::new(Box::new(file), MediaSourceStreamOptions::default());

    let mut hint = Hint::new();

    if let Some(ext) = src_path.extension().and_then(|ext| ext.to_str()) {
        hint.with_extension(ext);
    }

    if let Some(progress) = progress.as_deref_mut() {
        progress.on_stage("probing...");
    }

    let probe = default::get_probe().format(
        &hint,
        stream,
        &FormatOptions::default(),
        &MetadataOptions {
            limit_metadata_bytes: Limit::Maximum(0),
            limit_visual_bytes: Limit::Maximum(0),
        },
    )?;

    let mut format = probe.format;

    let track = format
        .default_track()
        .or_else(|| {
            format
                .tracks()
                .iter()
                .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        })
        .ok_or_else(|| SymphoniaError::Unsupported("Missing audio track"))?;

    let track_id = track.id;

    let time_base = track.codec_params.time_base;
    let n_frames = track.codec_params.n_frames;

    let mut decoder =
        default::get_codecs().make(&track.codec_params, &DecoderOptions::default())?;

    encoder.init()?;

    if let Some(progress) = progress.as_deref_mut() {
        progress.on_stage("transcoding...");
    }

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(SymphoniaError::IoError(err))
                if err.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(SymphoniaError::ResetRequired) => {
                decoder.reset();
                continue;
            }
            Err(err) => return Err(Error::from(err)),
        };

        if packet.track_id() != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(bytes) => bytes,
            Err(SymphoniaError::DecodeError(_)) => continue, // bad frame; skip
            Err(err) => return Err(Error::from(err)),
        };
        encoder.push(decoded)?;

        if let (Some(time_base), Some(n_frames)) = (time_base, n_frames) {
            let time = time_base.calc_time(packet.ts);
            let dur = time_base.calc_time(n_frames);
            let elapsed = time.seconds as f64 + time.frac;
            let total = dur.seconds as f64 + dur.frac;

            if total > 0.0 {
                if let Some(progress) = progress.as_deref_mut() {
                    progress.on_ratio(((elapsed / total) as f32).min(0.999));
                }
            }
        }
    }

    let written = encoder.finish()?;

    if let Some(progress) = progress.as_deref_mut() {
        progress.on_stage("finished");
        progress.on_ratio(1.0);
    }
    Ok(written)
}
