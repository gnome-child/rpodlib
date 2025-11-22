use rpod_device::capabilities::audio::AudioCapability;
use rpod_meta::technical::{CodecInfo, FormatInfo};

use crate::TranscodeOptions;

pub(crate) struct PcmCeiling {
    pub max_sample_rate: u32,
    pub max_channels: u16,
    pub max_bits_per_sample: u16,
}

impl PcmCeiling {
    pub(crate) fn from_cap(cap: &AudioCapability) -> Self {
        match cap {
            AudioCapability::Aiff(cap) => Self {
                max_sample_rate: cap.max_sample_rate,
                max_channels: if cap.multichannel {
                    8
                } else if cap.stereo {
                    2
                } else if cap.mono {
                    1
                } else {
                    2
                },
                max_bits_per_sample: cap.max_bit_depth,
            },

            AudioCapability::Mp3(cap) => Self {
                max_sample_rate: cap.max_sample_rate,
                max_channels: if cap.stereo || cap.multichannel {
                    2
                } else if cap.mono {
                    1
                } else {
                    2
                },
                max_bits_per_sample: 16,
            },

            AudioCapability::Aac(cap) => Self {
                max_sample_rate: cap.max_sample_rate,
                max_channels: 2,
                max_bits_per_sample: 16,
            },

            AudioCapability::AppleLossless(cap) => Self {
                max_sample_rate: cap.max_sample_rate,
                max_channels: if cap.multichannel {
                    8
                } else if cap.stereo {
                    2
                } else if cap.mono {
                    1
                } else {
                    2
                },
                max_bits_per_sample: cap.max_truncated_bit_depth,
            },

            _ => Self {
                max_sample_rate: 48_000,
                max_channels: 2,
                max_bits_per_sample: 16,
            },
        }
    }
}

pub(crate) fn prefer<'a>(
    caps: &'a [AudioCapability],
    opts: &TranscodeOptions,
) -> Option<&'a AudioCapability> {
    let rank = |fmt: &&AudioCapability| -> (u16, std::cmp::Reverse<u32>, std::cmp::Reverse<u16>) {
        let base = if opts.compress {
            match fmt {
                AudioCapability::Aac(_) => 0,
                AudioCapability::Mp3(_) => 1,
                AudioCapability::AppleLossless(_) => 2,
                AudioCapability::Aiff(_) => 3,
                AudioCapability::Audible(_) => 98,
                _ => 99,
            }
        } else {
            match fmt {
                AudioCapability::AppleLossless(_) => 0,
                AudioCapability::Aac(_) => 1,
                AudioCapability::Mp3(_) => 2,
                AudioCapability::Aiff(_) => 3,
                AudioCapability::Audible(_) => 98,
                _ => 99,
            }
        };

        let pcm = PcmCeiling::from_cap(fmt);
        (
            base,
            std::cmp::Reverse(pcm.max_sample_rate),
            std::cmp::Reverse(pcm.max_channels),
        )
    };
    caps.iter().min_by_key(rank)
}

pub(crate) fn conforms(fmt: &FormatInfo, caps: &[AudioCapability]) -> bool {
    let sample_rate = fmt.sample_rate;
    let channel_count = fmt.channel_count;

    for cap in caps {
        let limits = PcmCeiling::from_cap(cap);

        let ok_sample_rate =
            sample_rate <= limits.max_sample_rate && channel_count <= limits.max_channels as usize;

        match (cap, &fmt.codec_info) {
            (AudioCapability::AppleLossless(_), CodecInfo::Alac(_)) if ok_sample_rate => {
                return true;
            }
            (AudioCapability::Aac(_), CodecInfo::Aac) if ok_sample_rate => return true,
            (AudioCapability::Mp3(_), CodecInfo::Mp3(_)) if ok_sample_rate => return true,
            (AudioCapability::Aiff(_), CodecInfo::Raw(_)) if ok_sample_rate => return true,
            _ => {}
        }
    }
    false
}
