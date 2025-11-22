use std::io::{Seek, SeekFrom, Write};

use symphonia::core::audio::{AudioBufferRef, Channels};

use crate::{
    encoder::{Encoder, InterleavedBuffer},
    error::{Error, Result},
};

#[repr(u16)]
enum WavTag {
    Pcm = 0x0001,
    IeeeFloat = 0x0003,
    Extensible = 0xFFFE,
}

const WAV_MASK_KNOWN: u32 = (1u32 << 18) - 1;

const SUBFORMAT_PCM: [u8; 16] = [
    0x01, 0x00, 0x00, 0x00, // Data1 = 0x00000001 LE
    0x00, 0x00, // Data2 = 0x0000 LE
    0x10, 0x00, // Data3 = 0x0010 LE
    0x80, 0x00, 0x00, 0xAA, 0x00, 0x38, 0x9B, 0x71, // Data4 (big-endian-like raw byte order)
];

const SUBFORMAT_FLOAT: [u8; 16] = [
    0x03, 0x00, 0x00, 0x00, // Data1 = 0x00000003 LE
    0x00, 0x00, // Data2 = 0x0000 LE
    0x10, 0x00, // Data3 = 0x0010 LE
    0x80, 0x00, 0x00, 0xAA, 0x00, 0x38, 0x9B, 0x71,
];

enum Sink {
    Seekable(Box<dyn WriteSeek>),
    Streaming(Box<dyn Write>),
}

trait WriteSeek: Write + Seek {}
impl<T: Write + Seek> WriteSeek for T {}

pub struct WavEncoder {
    sink: Sink,
    buffer: InterleavedBuffer,
    streaming: bool,
    header_written: bool,
    header_fmt_size: u32,
    bytes_written: u64,
    riff_size_pos: u64,
    data_size_pos: u64,
}

impl WavEncoder {
    pub fn streaming<W: Write + 'static>(sink: W) -> Self {
        Self {
            sink: Sink::Streaming(Box::new(sink)),
            buffer: InterleavedBuffer::new(),
            streaming: true,
            header_written: false,
            header_fmt_size: 0,
            bytes_written: 0,
            riff_size_pos: 0,
            data_size_pos: 0,
        }
    }

    pub fn seekable<W: Write + Seek + 'static>(sink: W) -> Self {
        Self {
            sink: Sink::Seekable(Box::new(sink)),
            buffer: InterleavedBuffer::new(),
            streaming: false,
            header_written: false,
            header_fmt_size: 0,
            bytes_written: 0,
            riff_size_pos: 0,
            data_size_pos: 0,
        }
    }

    pub fn bytes_written(&self) -> u64 {
        self.bytes_written
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        match &mut self.sink {
            Sink::Seekable(w) => Ok(w.write_all(buf)?),
            Sink::Streaming(w) => Ok(w.write_all(buf)?),
        }
    }

    fn flush(&mut self) -> Result<()> {
        match &mut self.sink {
            Sink::Seekable(w) => Ok(w.flush()?),
            Sink::Streaming(w) => Ok(w.flush()?),
        }
    }

    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        match &mut self.sink {
            Sink::Seekable(w) => Ok(w.seek(pos)?),
            Sink::Streaming(_) => Err(std::io::Error::new(
                std::io::ErrorKind::NotSeekable,
                "Tried to seek on an unseekable stream!",
            ))?,
        }
    }

    fn write_header(
        &mut self,
        channels: Channels,
        sample_rate: u32,
        bits_per_sample: u16,
        is_float: bool,
    ) -> Result<()> {
        let channel_count = channels.count();
        let block_align = (channel_count as u32 * bits_per_sample as u32 / 8) as u16;
        let byte_rate = sample_rate * block_align as u32;

        if channel_count > 2 {
            // TODO: downmix channels to ceiling value
            return Err(Error::Generic("Multichannel is not yet supported!"));
        }

        let tag = if is_float {
            WavTag::IeeeFloat as u16
        } else {
            WavTag::Pcm as u16
        };

        let extensible = channel_count > 2 || bits_per_sample > 16 || is_float;
        let fmt_size = if !extensible { 16 } else { 40 };
        self.header_fmt_size = fmt_size;

        self.write_all(b"RIFF")?;

        if !self.streaming {
            self.riff_size_pos = self.seek(SeekFrom::Current(0))?;
        }

        self.write_all(&[0, 0, 0, 0])?;
        self.write_all(b"WAVE")?;
        self.write_all(b"fmt ")?;
        self.write_all(&le_u32(fmt_size))?;

        if fmt_size == 16 {
            self.write_all(&le_u16(tag))?;
            self.write_all(&le_u16(channel_count as u16))?;
            self.write_all(&le_u32(sample_rate))?;
            self.write_all(&le_u32(byte_rate))?;
            self.write_all(&le_u16(block_align))?;
            self.write_all(&le_u16(bits_per_sample))?;
        } else {
            self.write_all(&le_u16(WavTag::Extensible as u16))?;
            self.write_all(&le_u16(channel_count as u16))?;
            self.write_all(&le_u32(sample_rate))?;
            self.write_all(&le_u32(byte_rate))?;
            self.write_all(&le_u16(block_align))?;
            self.write_all(&le_u16(bits_per_sample))?;
            self.write_all(&le_u16(22))?;
            self.write_all(&le_u16(bits_per_sample))?;
            self.write_all(&le_u32(wav_channel_mask_from(channels)))?;

            if is_float {
                self.write_all(&SUBFORMAT_FLOAT)?;
            } else {
                self.write_all(&SUBFORMAT_PCM)?;
            }
        }
        self.write_all(b"data")?;

        if !self.streaming {
            self.data_size_pos = self.seek(SeekFrom::Current(0))?;
        }
        self.write_all(&[0, 0, 0, 0])?;

        Ok(())
    }

    fn patch_sizes(&mut self) -> Result<()> {
        let data_size = self.bytes_written as u32;
        let riff_size = 20u32
            .checked_add(self.header_fmt_size)
            .and_then(|v| v.checked_add(data_size))
            .ok_or(Error::EncoderOverflow {
                bytes: self.bytes_written,
            })?;

        self.seek(SeekFrom::Start(self.riff_size_pos))?;
        self.write_all(&le_u32(riff_size))?;
        self.seek(SeekFrom::Start(self.data_size_pos))?;
        self.write_all(&le_u32(data_size))?;
        self.seek(SeekFrom::End(0))?;
        Ok(())
    }
}

impl Encoder for WavEncoder {
    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn push(&mut self, audio_buffer: AudioBufferRef<'_>) -> Result<()> {
        if !self.header_written {
            let channels = audio_buffer.spec().channels;
            let sample_rate = audio_buffer.spec().rate;

            let (bits_per_sample, is_float) = match audio_buffer {
                AudioBufferRef::U8(_) => (8, false),
                AudioBufferRef::U16(_) => (16, false),
                AudioBufferRef::U24(_) => (24, false),
                AudioBufferRef::U32(_) => (32, false),

                AudioBufferRef::S8(_) => (8, false),
                AudioBufferRef::S16(_) => (16, false),
                AudioBufferRef::S24(_) => (24, false),
                AudioBufferRef::S32(_) => (32, false),

                AudioBufferRef::F32(_) => (32, true),
                AudioBufferRef::F64(_) => (64, true),
            };

            self.write_header(channels, sample_rate, bits_per_sample, is_float)?;
            self.header_written = true;
        }

        let bytes = {
            let from_buf = self.buffer.fill_from(audio_buffer);
            from_buf.to_vec()
        };

        self.write_all(&bytes)?;
        self.bytes_written =
            self.bytes_written
                .checked_add(bytes.len() as u64)
                .ok_or(Error::EncoderOverflow {
                    bytes: self.bytes_written,
                })?;
        Ok(())
    }

    fn finish(mut self) -> Result<u64> {
        if !self.streaming {
            self.patch_sizes()?;
        }
        self.flush()?;

        Ok(self.bytes_written)
    }
}

fn wav_channel_mask_from(ch: Channels) -> u32 {
    let m = ch.bits() & WAV_MASK_KNOWN;

    if m == 0 {
        match ch.count() {
            1 => 0x0000_0004,
            2 => 0x0000_0001 | 0x0000_0002,
            _ => 0,
        }
    } else {
        m
    }
}

fn le_u16(int: u16) -> [u8; 2] {
    int.to_le_bytes()
}

fn le_u32(int: u32) -> [u8; 4] {
    int.to_le_bytes()
}
