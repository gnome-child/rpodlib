use std::{
    fs,
    path::{Path, PathBuf},
};

use rpod_device::capabilities::audio::AudioCapability;
use rpod_display::Progress;
use rpod_meta::Manifest;

use crate::{
    encoder::qaac,
    error::{Error, Result},
    helpers::PcmCeiling,
};

pub mod decoder;
pub mod encoder;
pub mod error;

mod helpers;

#[derive(Clone, Copy)]
pub enum Target {
    Alac {
        sample_rate: u32,
        bit_depth: u16,
        channels: u16,
    },
    Aac {
        kbps: u32,
        vbr: bool,
        sample_rate: u32,
        channels: u16,
    },
    Mp3 {
        kbps: u32,
        vbr: bool,
        sample_rate: u32,
        channels: u16,
    },
    Uncompressed {
        sample_rate: u32,
        bit_depth: u16,
        channels: u16,
    },
}

impl Target {
    pub fn get_extension(&self) -> &'static str {
        match self {
            Self::Aac { .. } | Self::Alac { .. } => "m4a",
            Self::Mp3 { .. } => "mp3",
            Self::Uncompressed { .. } => "aiff",
        }
    }
}

pub enum Outcome {
    Transcoded {
        out_path: PathBuf,
        operations: Vec<String>,
        bytes_written: u64,
    },
    Skipped {
        reason: &'static str,
    },
}

pub struct TranscodeOptions {
    compress: bool,
    target: Option<Target>,
}

impl Default for TranscodeOptions {
    fn default() -> Self {
        Self {
            compress: false,
            target: None,
        }
    }
}

pub fn transcode<P: AsRef<Path>>(
    src_manifest: &Manifest,
    target_dev_capabilities: &[AudioCapability],
    opts: &TranscodeOptions,
    out_dir: P,
) -> Result<Outcome> {
    let out_dir = out_dir.as_ref();
    let plan = Plan::auto(src_manifest, target_dev_capabilities, &opts)?;

    transcode_with_plan(src_manifest, out_dir, plan, None)
}

pub fn transcode_with_progress<P: AsRef<Path>>(
    src_manifest: &Manifest,
    target_dev_capabilities: &[AudioCapability],
    opts: &TranscodeOptions,
    out_dir: P,
    progress: Option<&mut dyn Progress>,
) -> Result<Outcome> {
    let out_dir = out_dir.as_ref();
    let plan = Plan::auto(src_manifest, target_dev_capabilities, &opts)?;

    transcode_with_plan(src_manifest, out_dir, plan, progress)
}

pub fn auto_transcode<P: AsRef<Path>>(
    src_manifest: &Manifest,
    target_dev_capabilities: &[AudioCapability],
    out_dir: P,
) -> Result<Outcome> {
    let out_dir = out_dir.as_ref();
    let opts = TranscodeOptions::default();
    let plan = Plan::auto(src_manifest, target_dev_capabilities, &opts)?;

    transcode_with_plan(src_manifest, out_dir, plan, None)
}

pub fn auto_transcode_with_progress<P: AsRef<Path>>(
    src_manifest: &Manifest,
    target_dev_capabilities: &[AudioCapability],
    out_dir: P,
    progress: Option<&mut dyn Progress>,
) -> Result<Outcome> {
    let out_dir = out_dir.as_ref();
    let opts = TranscodeOptions::default();
    let plan = Plan::auto(src_manifest, target_dev_capabilities, &opts)?;

    transcode_with_plan(src_manifest, out_dir, plan, progress)
}

enum Plan {
    Transcode {
        target: Target,
        operations: Vec<String>,
    },
    Skip {
        reason: &'static str,
    },
}

impl Plan {
    fn auto(
        manifest: &Manifest,
        dev_capabilities: &[AudioCapability],
        opts: &TranscodeOptions,
    ) -> Result<Self> {
        if helpers::conforms(&manifest.format_info, dev_capabilities) {
            Ok(Plan::Skip {
                reason: "File conforms to device capabilities",
            })
        } else {
            if let Some(target) = opts.target {
                Ok(Plan::Transcode {
                    target,
                    operations: vec!["Target override was supplied".to_string()],
                })
            } else {
                let preferred_cap = helpers::prefer(dev_capabilities, opts).ok_or(
                    Error::Generic("Couldn't infer a transcode target from device capabilities"),
                )?;

                let pcm_ceiling = PcmCeiling::from_cap(preferred_cap);
                let src_sample_rate = manifest.format_info.sample_rate;
                let src_channel_count = manifest.format_info.channel_count;
                let src_bits_per = manifest.format_info.codec_info.bits_per_sample() as u16;
                let src_avg_kbps = manifest.format_info.avg_kbps;
                let src_is_vbr = manifest.format_info.codec_info.is_vbr();

                let mut operations: Vec<String> = Vec::new();

                let tgt_sample_rate =
                    |sample_rate: u32, max: u32, operations: &mut Vec<String>| -> u32 {
                        if sample_rate > max {
                            operations.push(format!(
                                "Sample rate decreased from {} Hz to {} Hz",
                                sample_rate, max
                            ));
                        }
                        sample_rate.min(max)
                    };

                let tgt_bits_per =
                    |bits_per_sample: u16, max: u16, operations: &mut Vec<String>| -> u16 {
                        if bits_per_sample > max {
                            operations.push(format!(
                                "Bits per sample decreased from {} to {}",
                                bits_per_sample, max
                            ));
                        }
                        bits_per_sample.min(max)
                    };

                let tgt_channel_count =
                    |channel_count: u16, max: u16, operations: &mut Vec<String>| -> u16 {
                        if channel_count > max {
                            operations.push(format!(
                                "Channels decreased from {} to {}",
                                channel_count, max
                            ));
                        }
                        channel_count.min(max)
                    };

                Ok(match preferred_cap {
                    AudioCapability::AppleLossless(_) => {
                        let sample_rate = tgt_sample_rate(
                            src_sample_rate,
                            pcm_ceiling.max_sample_rate,
                            &mut operations,
                        );
                        let bit_depth = tgt_bits_per(
                            src_bits_per,
                            pcm_ceiling.max_bits_per_sample,
                            &mut operations,
                        );
                        let channels = tgt_channel_count(
                            src_channel_count as u16,
                            pcm_ceiling.max_channels,
                            &mut operations,
                        );

                        if operations.is_empty() {
                            operations.push("Normalize to device-friendly alac".to_string());
                        }

                        Plan::Transcode {
                            target: Target::Alac {
                                sample_rate,
                                bit_depth,
                                channels,
                            },
                            operations,
                        }
                    }

                    AudioCapability::Aiff(_) => {
                        let sample_rate = tgt_sample_rate(
                            src_sample_rate,
                            pcm_ceiling.max_sample_rate,
                            &mut operations,
                        );
                        let bit_depth = tgt_bits_per(
                            src_bits_per,
                            pcm_ceiling.max_bits_per_sample,
                            &mut operations,
                        );
                        let channels = tgt_channel_count(
                            src_channel_count as u16,
                            pcm_ceiling.max_channels,
                            &mut operations,
                        );

                        if operations.is_empty() {
                            operations
                                .push("Normalize to device-friendly uncompressed".to_string());
                        }

                        Plan::Transcode {
                            target: Target::Uncompressed {
                                sample_rate,
                                bit_depth,
                                channels,
                            },
                            operations,
                        }
                    }

                    AudioCapability::Aac(cap) => {
                        let sample_rate = tgt_sample_rate(
                            src_sample_rate,
                            pcm_ceiling.max_sample_rate,
                            &mut operations,
                        );
                        let channels = tgt_channel_count(
                            src_channel_count as u16,
                            pcm_ceiling.max_channels,
                            &mut operations,
                        );
                        let kbps = src_avg_kbps.clamp(96, 256);
                        let vbr = cap.vbr && src_is_vbr;

                        if operations.is_empty() {
                            operations.push("Normalize to device-friendly aac".to_string());
                        }

                        Plan::Transcode {
                            target: Target::Aac {
                                kbps,
                                sample_rate,
                                channels,
                                vbr,
                            },
                            operations,
                        }
                    }

                    AudioCapability::Mp3(cap) => {
                        let sample_rate = tgt_sample_rate(
                            src_sample_rate,
                            pcm_ceiling.max_sample_rate,
                            &mut operations,
                        );
                        let channels = tgt_channel_count(
                            src_channel_count as u16,
                            pcm_ceiling.max_channels,
                            &mut operations,
                        );
                        let kbps = if src_avg_kbps > cap.max_data_rate {
                            operations.push("".to_string());
                            cap.max_data_rate
                        } else {
                            src_avg_kbps
                        };
                        let vbr = src_is_vbr;

                        if operations.is_empty() {
                            operations.push("Normalize to device-friendly mp3".to_string());
                        }

                        Plan::Transcode {
                            target: Target::Mp3 {
                                kbps,
                                vbr,
                                sample_rate,
                                channels,
                            },
                            operations,
                        }
                    }

                    _ => Plan::Skip {
                        reason: "Skipped for unknown reason",
                    },
                })
            }
        }
    }
}

fn transcode_with_plan(
    src_manifest: &Manifest,
    out_dir: &Path,
    plan: Plan,
    progress: Option<&mut dyn Progress>,
) -> Result<Outcome> {
    match plan {
        Plan::Transcode { target, operations } => {
            let operations = operations.clone();
            let mut out_path = out_dir.join(&src_manifest.name);
            let mut bytes_written = 0;

            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }

            match target {
                // TODO: handle channels
                Target::Alac {
                    sample_rate,
                    bit_depth,
                    ..
                } => {
                    out_path = out_path.with_extension(target.get_extension());

                    let refalac = qaac::lossless(&out_path)
                        .sample_rate(sample_rate)
                        .bits_per_sample(bit_depth)
                        .build()?;

                    bytes_written = decoder::decode_into(&src_manifest.path, refalac, progress)?;
                }

                Target::Aac { .. } => {
                    out_path = out_path.with_extension(target.get_extension());

                    let qaac = qaac::lossy(&out_path)
                        .quality(qaac::Quality::High)
                        .build()?;

                    bytes_written = decoder::decode_into(&src_manifest.path, qaac, progress)?;
                }

                // TODO: the rest
                _ => {}
            }

            src_manifest
                .tags
                .save_to(&out_path)
                .map_err(|_| Error::MetadataCopy {
                    path: out_path.to_path_buf(),
                })?;

            Ok(Outcome::Transcoded {
                out_path,
                operations,
                bytes_written,
            })
        }

        Plan::Skip { reason } => Ok(Outcome::Skipped { reason }),
    }
}
