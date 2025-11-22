use std::{fs, path::Path};

use symphonia::core::{
    codecs::{self, CodecType},
    formats::{FormatOptions, FormatReader},
    io::{MediaSourceStream, MediaSourceStreamOptions},
    meta::{Limit, MetadataOptions},
    probe::Hint,
    units::Time,
};

use crate::error::{Error, Result};

#[derive(Debug)]
pub enum CodecInfo {
    Raw(Uncompressed),
    Flac(Lossless),
    Alac(Lossless),
    Mp3(Mp3),
    Aac,
    Opus,
}

impl CodecInfo {
    pub fn is_uncompressed(&self) -> bool {
        matches!(self, Self::Raw(_))
    }

    pub fn is_float_pcm(&self) -> bool {
        matches!(self, Self::Raw(Uncompressed { float: true, .. }))
    }

    pub fn is_lossless(&self) -> bool {
        matches!(self, Self::Flac(_) | Self::Alac(_))
    }

    pub fn is_lossy(&self) -> bool {
        matches!(self, Self::Mp3(_) | Self::Aac | Self::Opus)
    }

    pub fn bits_per_sample(&self) -> u32 {
        match *self {
            Self::Raw(Uncompressed {
                bits_per_sample, ..
            }) => bits_per_sample,
            Self::Flac(Lossless { bits_per_sample }) => bits_per_sample,
            Self::Alac(Lossless { bits_per_sample }) => bits_per_sample,
            _ => 0,
        }
    }

    pub fn is_vbr(&self) -> bool {
        matches!(self, Self::Mp3(Mp3 { vbr: true, .. }))
    }

    pub fn mp3_prefetch_span_bytes(&self) -> u32 {
        match *self {
            Self::Mp3(Mp3 {
                end_prefetch_span_bytes,
                ..
            }) => end_prefetch_span_bytes,
            _ => 0,
        }
    }

    pub fn codec_description(&self) -> &'static str {
        match self {
            Self::Raw(_) => "Uncompressed audio file",
            Self::Flac(_) => "FLAC audio file",
            Self::Alac(_) => "Apple Lossless audio file",
            Self::Mp3(_) => "MPEG audio file",
            Self::Aac => "AAC audio file",
            Self::Opus => "Opus audio file",
        }
    }
}

#[derive(Debug)]
pub struct Uncompressed {
    pub bits_per_sample: u32,
    pub float: bool,
}

#[derive(Debug)]
pub struct Lossless {
    pub bits_per_sample: u32,
}

#[derive(Debug)]
pub struct Mp3 {
    pub vbr: bool,
    pub end_prefetch_span_bytes: u32,
}

#[derive(Debug)]
pub struct FormatInfo {
    pub codec_info: CodecInfo,
    pub duration_ms: u64,
    pub channel_count: usize,
    pub sample_rate: u32,
    pub frame_count: u64,
    pub delay_frames: u32,
    pub padding_frames: u32,
    pub avg_kbps: u32,
}

impl FormatInfo {
    pub fn probe<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let file = fs::File::open(path)?;
        let stream = MediaSourceStream::new(Box::new(file), MediaSourceStreamOptions::default());

        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|ext| ext.to_str()) {
            hint.with_extension(ext);
        }

        let probe = symphonia::default::get_probe().format(
            &hint,
            stream,
            &FormatOptions {
                enable_gapless: true,
                ..FormatOptions::default()
            },
            &MetadataOptions {
                limit_metadata_bytes: Limit::Maximum(0),
                limit_visual_bytes: Limit::Maximum(0),
            },
        )?;

        let mut format_reader = probe.format;

        Self::from_reader(path, format_reader.as_mut())
    }

    pub fn from_reader(path: &Path, format_reader: &mut dyn FormatReader) -> Result<Self> {
        let track = format_reader.default_track().ok_or(Error::NoAudioTracks {
            path: path.to_path_buf(),
        })?;

        let time_base = track.codec_params.time_base.ok_or(Error::TimeBaseUnknown {
            path: path.to_path_buf(),
        })?;

        let channel_count = track
            .codec_params
            .channels
            .map(|channels| channels.count())
            .unwrap_or(2);

        let frame_count = track
            .codec_params
            .n_frames
            .ok_or(Error::MissingFrameCount {
                path: path.to_path_buf(),
            })?;

        let sample_rate = track
            .codec_params
            .sample_rate
            .ok_or(Error::MissingSampleRate {
                path: path.to_path_buf(),
            })?;

        let track_id = track.id;
        let codec_type = track.codec_params.codec;
        let bits_per_sample = track.codec_params.bits_per_sample.unwrap_or(0);
        let delay_frames = track.codec_params.delay.unwrap_or(0);
        let padding_frames = track.codec_params.padding.unwrap_or(0);

        const TAIL: usize = 8;

        let mut first_timestamp = None;
        let mut last_timestamp: u64 = 0;
        let mut total_bytes: u64 = 0;
        let mut packet_count: u64 = 0;
        let mut mean: f64 = 0.0;
        let mut m2: f64 = 0.0;
        let mut tail_sizes = [0u64; TAIL];
        let mut tail_durs = [0u64; TAIL];
        let mut tail_byte_total: u64 = 0;
        let mut tail_dur_total: u64 = 0;
        let mut tail_count: usize = 0;

        loop {
            match format_reader.next_packet() {
                Ok(packet) if packet.track_id() == track_id => {
                    let size = packet.buf().len() as u64;
                    total_bytes = total_bytes.saturating_add(size);
                    packet_count += 1;

                    let x = size as f64;
                    let delta = x - mean;
                    mean += delta / (packet_count as f64);
                    m2 += delta * (x - mean);

                    first_timestamp.get_or_insert(packet.ts);
                    last_timestamp = packet.ts.saturating_add(packet.dur);

                    let idx = (tail_count as usize) % TAIL;

                    if tail_count >= TAIL {
                        tail_byte_total = tail_byte_total.saturating_sub(tail_sizes[idx]);
                        tail_dur_total = tail_dur_total.saturating_sub(tail_durs[idx]);
                    }
                    tail_sizes[idx] = size;
                    tail_durs[idx] = packet.dur;
                    tail_byte_total = tail_byte_total.saturating_add(size);
                    tail_dur_total = tail_dur_total.saturating_add(packet.dur);
                    tail_count += 1;
                }
                Ok(_) => continue,
                Err(_) => break,
            }
        }

        let total_ts = match first_timestamp {
            Some(f) if last_timestamp >= f => last_timestamp - f,
            _ => 0,
        };

        let t_for_kbps = time_base.calc_time(total_ts);
        let dur_ms = time_to_ms(t_for_kbps);
        let bits_total = (total_bytes as u128) * 8;
        let avg_kbps: u32 = if dur_ms > 0 {
            let dur = dur_ms as u128;
            ((bits_total + dur / 2) / dur).min(u32::MAX as u128) as u32
        } else {
            0
        };

        let codec_info = if float_pcm_codec(codec_type) {
            CodecInfo::Raw(Uncompressed {
                bits_per_sample,
                float: true,
            })
        } else if int_pcm_codec(codec_type) {
            CodecInfo::Raw(Uncompressed {
                bits_per_sample,
                float: false,
            })
        } else if codec_type == codecs::CODEC_TYPE_ALAC {
            CodecInfo::Alac(Lossless { bits_per_sample })
        } else if codec_type == codecs::CODEC_TYPE_FLAC {
            CodecInfo::Flac(Lossless { bits_per_sample })
        } else if codec_type == codecs::CODEC_TYPE_AAC {
            CodecInfo::Aac
        } else if codec_type == codecs::CODEC_TYPE_MP3 {
            let vbr = if packet_count >= 8 && mean > 0.0 {
                let variance = m2 / (packet_count as f64);

                (variance.sqrt() / mean) > 0.10
            } else {
                false
            };

            let end_prefetch_span_bytes = if tail_count >= 8 {
                let span = total_bytes.saturating_sub(tail_byte_total);

                span.min(u32::MAX as u64) as u32
            } else {
                0
            };

            CodecInfo::Mp3(Mp3 {
                vbr,
                end_prefetch_span_bytes,
            })
        } else {
            return Err(Error::UnsupportedCodec {
                codec: codec_type,
                path: path.to_path_buf(),
            });
        };

        let duration_ms = time_to_ms(time_base.calc_time(frame_count));

        Ok(Self {
            codec_info,
            duration_ms,
            channel_count,
            sample_rate,
            frame_count,
            delay_frames,
            padding_frames,
            avg_kbps,
        })
    }
}

fn float_pcm_codec(codec: CodecType) -> bool {
    match codec {
        codecs::CODEC_TYPE_PCM_F32LE
        | codecs::CODEC_TYPE_PCM_F32LE_PLANAR
        | codecs::CODEC_TYPE_PCM_F32BE
        | codecs::CODEC_TYPE_PCM_F32BE_PLANAR
        | codecs::CODEC_TYPE_PCM_F64LE
        | codecs::CODEC_TYPE_PCM_F64LE_PLANAR
        | codecs::CODEC_TYPE_PCM_F64BE
        | codecs::CODEC_TYPE_PCM_F64BE_PLANAR => true,
        _ => false,
    }
}

fn int_pcm_codec(codec: CodecType) -> bool {
    match codec {
        codecs::CODEC_TYPE_PCM_S32LE
        | codecs::CODEC_TYPE_PCM_S32LE_PLANAR
        | codecs::CODEC_TYPE_PCM_S32BE
        | codecs::CODEC_TYPE_PCM_S32BE_PLANAR
        | codecs::CODEC_TYPE_PCM_S24LE
        | codecs::CODEC_TYPE_PCM_S24LE_PLANAR
        | codecs::CODEC_TYPE_PCM_S24BE
        | codecs::CODEC_TYPE_PCM_S24BE_PLANAR
        | codecs::CODEC_TYPE_PCM_S16LE
        | codecs::CODEC_TYPE_PCM_S16LE_PLANAR
        | codecs::CODEC_TYPE_PCM_S16BE
        | codecs::CODEC_TYPE_PCM_S16BE_PLANAR
        | codecs::CODEC_TYPE_PCM_S8
        | codecs::CODEC_TYPE_PCM_S8_PLANAR
        | codecs::CODEC_TYPE_PCM_U32LE
        | codecs::CODEC_TYPE_PCM_U32LE_PLANAR
        | codecs::CODEC_TYPE_PCM_U32BE
        | codecs::CODEC_TYPE_PCM_U32BE_PLANAR
        | codecs::CODEC_TYPE_PCM_U24LE
        | codecs::CODEC_TYPE_PCM_U24LE_PLANAR
        | codecs::CODEC_TYPE_PCM_U24BE
        | codecs::CODEC_TYPE_PCM_U24BE_PLANAR
        | codecs::CODEC_TYPE_PCM_U16LE
        | codecs::CODEC_TYPE_PCM_U16LE_PLANAR
        | codecs::CODEC_TYPE_PCM_U16BE
        | codecs::CODEC_TYPE_PCM_U16BE_PLANAR
        | codecs::CODEC_TYPE_PCM_U8
        | codecs::CODEC_TYPE_PCM_U8_PLANAR
        | codecs::CODEC_TYPE_PCM_ALAW
        | codecs::CODEC_TYPE_PCM_MULAW => true,
        _ => false,
    }
}

fn time_to_ms(time: Time) -> u64 {
    let frac = time.frac.clamp(0.0, 0.999_999_999);
    let whole_ms = (time.seconds) * 1_000;
    let mut frac_ms = (frac * 1_000.0).floor() as u64;

    if frac_ms > 1_000 {
        frac_ms = 1_000;
    }
    whole_ms + frac_ms
}
