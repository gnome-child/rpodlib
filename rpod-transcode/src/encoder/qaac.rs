use std::{
    ffi::{OsStr, OsString},
    io::Read,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread::JoinHandle,
};

use symphonia::core::audio::AudioBufferRef;
use which::which;

use crate::{
    encoder::{Encoder, wav::WavEncoder},
    error::{Error, Result},
};

#[cfg(target_os = "windows")]
const QAAC_ALIASES: &[&str] = &["qaac64.exe", "qaac.exe"];
#[cfg(target_os = "windows")]
const REFALAC_ALIASES: &[&str] = &["refalac64.exe", "refalac.exe"];

#[cfg(not(target_os = "windows"))]
const QAAC_ALIASES: &[&str] = &["qaac"];
#[cfg(not(target_os = "windows"))]
const REFALAC_ALIASES: &[&str] = &["refalac"];

const ENV_QAAC: &str = "QAAC_BIN";
const ENV_REFALAC: &str = "REFALAC_BIN";

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Quality {
    Low = 0,
    Med = 1,
    High = 2,
}

pub fn resolve_exe(lossless: bool) -> Result<PathBuf> {
    if let Some(path) = match lossless {
        true => std::env::var_os(ENV_REFALAC),
        false => std::env::var_os(ENV_QAAC),
    } {
        let path = PathBuf::from(path);

        if path.is_file() {
            return Ok(PathBuf::from(path));
        }
    }

    let candidates: &[&str] = match lossless {
        true => REFALAC_ALIASES,
        false => QAAC_ALIASES,
    };

    for name in candidates {
        if let Ok(path) = which(name) {
            return Ok(path);
        }
    }

    let tried = candidates.join(", ");

    Err(Error::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!(
            "qaac/refalac not found, tried: {tried}. \nSet {ENV_QAAC} or {ENV_REFALAC} to override."
        ),
    )))
}

pub fn lossy<P: AsRef<Path>>(out: P) -> QaacBuilder {
    let out = out.as_ref().to_path_buf();

    QaacBuilder {
        args: Vec::new(),
        out,
    }
}

pub fn lossless<P: AsRef<Path>>(out: P) -> RefalacBuilder {
    let out = out.as_ref().to_path_buf();

    RefalacBuilder {
        args: Vec::new(),
        out,
    }
}

pub struct QaacBuilder {
    args: Vec<OsString>,
    out: PathBuf,
}

impl QaacBuilder {
    pub fn quality(self, quality: Quality) -> Self {
        self.arg("-q").arg((quality as u8).to_string())
    }

    pub fn tvbr(self, quality: u8) -> Self {
        self.arg("-V").arg(quality.to_string())
    }

    pub fn cvbr(self, bit_rate: u16) -> Self {
        self.arg("-v").arg(bit_rate.to_string())
    }

    pub fn cbr(self, bit_rate: u16) -> Self {
        self.arg("-c").arg(bit_rate.to_string())
    }

    pub fn arg<A: AsRef<OsStr>>(mut self, arg: A) -> Self {
        let arg = arg.as_ref().to_os_string();

        self.args.push(arg);
        self
    }

    pub fn build(self) -> Result<QaacEncoder> {
        let mut cmd = Command::new(resolve_exe(false)?);

        cmd.arg("--ignorelength")
            .arg("-")
            .args(&self.args)
            .arg("-o")
            .arg(self.out);
        QaacEncoder::from_command(cmd)
    }
}

pub struct RefalacBuilder {
    args: Vec<OsString>,
    out: PathBuf,
}

impl RefalacBuilder {
    pub fn sample_rate(self, rate: u32) -> Self {
        self.arg("-r").arg((rate).to_string())
    }

    pub fn bits_per_sample(self, bits: u16) -> Self {
        self.arg("-b").arg((bits).to_string())
    }

    pub fn dither(self, enabled: bool) -> Self {
        if !enabled {
            self.arg("--no-dither")
        } else {
            self
        }
    }

    pub fn arg<A: AsRef<OsStr>>(mut self, arg: A) -> Self {
        let arg = arg.as_ref().to_os_string();

        self.args.push(arg);
        self
    }

    pub fn build(self) -> Result<QaacEncoder> {
        let mut cmd = Command::new(resolve_exe(true)?);

        cmd.arg("--ignorelength")
            .arg("-")
            .args(&self.args)
            .arg("-o")
            .arg(self.out);
        QaacEncoder::from_command(cmd)
    }
}

pub struct QaacEncoder {
    child_proc: Child,
    inner: WavEncoder,
    stderr_buf: Arc<Mutex<Vec<u8>>>,
    stderr_thread: JoinHandle<()>,
}

impl Encoder for QaacEncoder {
    fn init(&mut self) -> Result<()> {
        self.inner.init()
    }

    fn push(&mut self, audio_buffer: AudioBufferRef<'_>) -> Result<()> {
        self.inner.push(audio_buffer)
    }

    fn finish(mut self) -> Result<u64> {
        let bytes_written = self.inner.finish()?;

        let status = self.child_proc.wait()?;
        let _ = self.stderr_thread.join();
        let code = status.code();
        let message = String::from_utf8_lossy(&self.stderr_buf.lock().unwrap()).to_string();

        if !status.success() {
            Err(Error::ProcessFailed { code, message })
        } else {
            Ok(bytes_written)
        }
    }
}

impl QaacEncoder {
    pub fn from_command(mut cmd: Command) -> Result<Self> {
        cmd.stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .stdout(Stdio::null());

        let mut child_proc = cmd.spawn()?;

        let stdin = child_proc
            .stdin
            .take()
            .ok_or_else(|| Error::Generic("qaac: no stdin"))?;

        let inner = WavEncoder::streaming(stdin);

        let mut stderr = child_proc
            .stderr
            .take()
            .ok_or_else(|| Error::Generic("qaac: no stderr"))?;

        let stderr_buf = Arc::new(Mutex::new(Vec::new()));
        let buf_clone = stderr_buf.clone();

        let stderr_thread = std::thread::spawn(move || {
            let mut tmp = [0u8; 8192];

            while let Ok(n) = stderr.read(&mut tmp) {
                if n == 0 {
                    break;
                }

                if let Ok(mut g) = buf_clone.lock() {
                    g.extend_from_slice(&tmp[..n]);
                }
            }
        });

        Ok(Self {
            child_proc,
            inner,
            stderr_buf,
            stderr_thread,
        })
    }
}
