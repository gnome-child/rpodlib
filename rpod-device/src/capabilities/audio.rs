use plist::{Dictionary, Value};

#[derive(Debug)]
#[non_exhaustive]
pub enum AudioCapability {
    Aiff(Aiff),
    Mp3(Mp3),
    Aac(Aac),
    AppleLossless(AppleLossless),
    Audible(Audible),
}

#[derive(Debug)]
pub struct Aiff {
    pub mono: bool,
    pub stereo: bool,
    pub multichannel: bool,
    pub max_sample_rate: u32,
    pub max_bit_depth: u16,
}

impl From<Dictionary> for Aiff {
    fn from(value: Dictionary) -> Self {
        let mono = value
            .get("Mono")
            .and_then(Value::as_boolean)
            .unwrap_or(false);
        let stereo = value
            .get("Stereo")
            .and_then(Value::as_boolean)
            .unwrap_or(false);
        let multichannel = value
            .get("Multichannel")
            .and_then(Value::as_boolean)
            .unwrap_or(false);
        let max_sample_rate = value
            .get("MaximumSampleRate")
            .and_then(Value::as_unsigned_integer)
            .unwrap_or(44_100) as u32;
        let max_bit_depth = value
            .get("MaximumBitDepth")
            .and_then(Value::as_unsigned_integer)
            .unwrap_or(16) as u16;

        Self {
            mono,
            stereo,
            multichannel,
            max_sample_rate,
            max_bit_depth,
        }
    }
}

#[derive(Debug)]
pub struct Mp3 {
    pub mono: bool,
    pub stereo: bool,
    pub multichannel: bool,
    pub max_sample_rate: u32,
    pub max_data_rate: u32,
}

impl From<Dictionary> for Mp3 {
    fn from(value: Dictionary) -> Self {
        let mono = value
            .get("Mono")
            .and_then(Value::as_boolean)
            .unwrap_or(false);
        let stereo = value
            .get("Stereo")
            .and_then(Value::as_boolean)
            .unwrap_or(false);
        let multichannel = value
            .get("Multichannel")
            .and_then(Value::as_boolean)
            .unwrap_or(false);
        let max_sample_rate = value
            .get("MaximumSampleRate")
            .and_then(Value::as_unsigned_integer)
            .unwrap_or(44_100) as u32;
        let max_data_rate = value
            .get("MaximumDataRate")
            .and_then(Value::as_unsigned_integer)
            .unwrap_or(320) as u32;

        Self {
            mono,
            stereo,
            multichannel,
            max_sample_rate,
            max_data_rate,
        }
    }
}

#[derive(Debug)]
pub struct Aac {
    pub apple_drm: bool,
    pub max_sample_rate: u32,
    pub vbr: bool,
    pub perceptual_noise_sub: bool,
}

impl From<Dictionary> for Aac {
    fn from(value: Dictionary) -> Self {
        let apple_drm = value
            .get("AppleDRM")
            .and_then(Value::as_boolean)
            .unwrap_or(false);
        let max_sample_rate = value
            .get("MaximumSampleRate")
            .and_then(Value::as_unsigned_integer)
            .unwrap_or(48_000) as u32;

        let (vbr, perceptual_noise_sub) = match value.get("LC").and_then(Value::as_dictionary) {
            Some(lc) => {
                let vbr = lc
                    .get("VariableBitRate")
                    .and_then(Value::as_boolean)
                    .unwrap_or(false);
                let perceptual_noise_sub = lc
                    .get("PerceptualNoiseSubsitution")
                    .and_then(Value::as_boolean)
                    .unwrap_or(false);
                (vbr, perceptual_noise_sub)
            }
            None => (false, false),
        };

        Self {
            apple_drm,
            max_sample_rate,
            vbr,
            perceptual_noise_sub,
        }
    }
}

#[derive(Debug)]
pub struct AppleLossless {
    pub apple_drm: bool,
    pub mono: bool,
    pub stereo: bool,
    pub multichannel: bool,
    pub max_sample_rate: u32,
    pub max_bit_depth: u16,
    pub max_truncated_bit_depth: u16,
}

impl From<Dictionary> for AppleLossless {
    fn from(value: Dictionary) -> Self {
        let apple_drm = value
            .get("AppleDRM")
            .and_then(Value::as_boolean)
            .unwrap_or(false);
        let mono = value
            .get("Mono")
            .and_then(Value::as_boolean)
            .unwrap_or(false);
        let stereo = value
            .get("Stereo")
            .and_then(Value::as_boolean)
            .unwrap_or(false);
        let multichannel = value
            .get("Multichannel")
            .and_then(Value::as_boolean)
            .unwrap_or(false);
        let max_sample_rate = value
            .get("MaximumSampleRate")
            .and_then(Value::as_unsigned_integer)
            .unwrap_or(48_000) as u32;
        let max_bit_depth = value
            .get("MaximumBitDepthUntruncated")
            .and_then(Value::as_unsigned_integer)
            .unwrap_or(16) as u16;
        let max_truncated_bit_depth = value
            .get("MaximumBitDepth")
            .and_then(Value::as_unsigned_integer)
            .unwrap_or(max_bit_depth as u64) as u16;

        Self {
            apple_drm,
            mono,
            stereo,
            multichannel,
            max_sample_rate,
            max_bit_depth,
            max_truncated_bit_depth,
        }
    }
}

#[derive(Debug)]
pub struct Audible {
    pub type1: bool,
    pub type2: bool,
    pub type3: bool,
    pub type4: bool,
    pub aac: bool,
}

impl From<Dictionary> for Audible {
    fn from(value: Dictionary) -> Self {
        let type1 = value
            .get("Type1")
            .and_then(Value::as_boolean)
            .unwrap_or(false);
        let type2 = value
            .get("Type2")
            .and_then(Value::as_boolean)
            .unwrap_or(false);
        let type3 = value
            .get("Type3")
            .and_then(Value::as_boolean)
            .unwrap_or(false);
        let type4 = value
            .get("Type4")
            .and_then(Value::as_boolean)
            .unwrap_or(false);
        let aac = value
            .get("AAC")
            .and_then(Value::as_boolean)
            .unwrap_or(false);

        Self {
            type1,
            type2,
            type3,
            type4,
            aac,
        }
    }
}
