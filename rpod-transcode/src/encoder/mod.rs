use symphonia::core::{
    audio::{AudioBufferRef, RawSample, RawSampleBuffer, SignalSpec},
    sample::{Sample, i24, u24},
};

use crate::error::Result;

pub mod ffmpeg;
pub mod qaac;
pub mod wav;

pub trait Encoder {
    fn init(&mut self) -> Result<()>;
    fn push(&mut self, audio_buffer: AudioBufferRef<'_>) -> Result<()>;
    fn finish(self) -> Result<u64>;
}

struct InterleavedBuffer {
    buffer_u8: Option<RawSampleBuffer<u8>>,
    buffer_u16: Option<RawSampleBuffer<u16>>,
    buffer_u24: Option<RawSampleBuffer<u24>>,
    buffer_u32: Option<RawSampleBuffer<u32>>,

    buffer_i8: Option<RawSampleBuffer<i8>>,
    buffer_i16: Option<RawSampleBuffer<i16>>,
    buffer_i24: Option<RawSampleBuffer<i24>>,
    buffer_i32: Option<RawSampleBuffer<i32>>,

    buffer_f32: Option<RawSampleBuffer<f32>>,
    buffer_f64: Option<RawSampleBuffer<f64>>,
}

impl InterleavedBuffer {
    fn new() -> Self {
        Self {
            buffer_u8: None,
            buffer_u16: None,
            buffer_u24: None,
            buffer_u32: None,
            buffer_i8: None,
            buffer_i16: None,
            buffer_i24: None,
            buffer_i32: None,
            buffer_f32: None,
            buffer_f64: None,
        }
    }

    fn ensure<T: Sample + RawSample>(
        slot: &mut Option<RawSampleBuffer<T>>,
        frames: usize,
        spec: SignalSpec,
    ) -> &mut RawSampleBuffer<T> {
        let need_samples = frames * spec.channels.count();
        let lacks = slot.as_ref().map_or(true, |b| b.capacity() < need_samples);
        if lacks {
            *slot = Some(RawSampleBuffer::<T>::new(frames as u64, spec));
        }
        slot.as_mut().unwrap()
    }

    fn fill_from<'a>(&'a mut self, audio_buffer: AudioBufferRef<'_>) -> &'a [u8] {
        let spec = *audio_buffer.spec();
        let frames = audio_buffer.frames();

        let bytes = match audio_buffer {
            AudioBufferRef::U8(_) => {
                let buf = Self::ensure(&mut self.buffer_u8, frames, spec);
                buf.clear();
                buf.copy_interleaved_ref(audio_buffer);
                buf.as_bytes()
            }

            AudioBufferRef::U16(_) => {
                let buf = Self::ensure(&mut self.buffer_u16, frames, spec);
                buf.clear();
                buf.copy_interleaved_ref(audio_buffer);
                buf.as_bytes()
            }

            AudioBufferRef::U24(_) => {
                let buf = Self::ensure(&mut self.buffer_u24, frames, spec);
                buf.clear();
                buf.copy_interleaved_ref(audio_buffer);
                buf.as_bytes()
            }

            AudioBufferRef::U32(_) => {
                let buf = Self::ensure(&mut self.buffer_u32, frames, spec);
                buf.clear();
                buf.copy_interleaved_ref(audio_buffer);
                buf.as_bytes()
            }

            AudioBufferRef::S8(_) => {
                let buf = Self::ensure(&mut self.buffer_i8, frames, spec);
                buf.clear();
                buf.copy_interleaved_ref(audio_buffer);
                buf.as_bytes()
            }

            AudioBufferRef::S16(_) => {
                let buf = Self::ensure(&mut self.buffer_i16, frames, spec);
                buf.clear();
                buf.copy_interleaved_ref(audio_buffer);
                buf.as_bytes()
            }

            AudioBufferRef::S24(_) => {
                let buf = Self::ensure(&mut self.buffer_i24, frames, spec);
                buf.clear();
                buf.copy_interleaved_ref(audio_buffer);
                buf.as_bytes()
            }

            AudioBufferRef::S32(_) => {
                let buf = Self::ensure(&mut self.buffer_i32, frames, spec);
                buf.clear();
                buf.copy_interleaved_ref(audio_buffer);
                buf.as_bytes()
            }

            AudioBufferRef::F32(_) => {
                let buf = Self::ensure(&mut self.buffer_f32, frames, spec);
                buf.clear();
                buf.copy_interleaved_ref(audio_buffer);
                buf.as_bytes()
            }

            AudioBufferRef::F64(_) => {
                let buf = Self::ensure(&mut self.buffer_f64, frames, spec);
                buf.clear();
                buf.copy_interleaved_ref(audio_buffer);
                buf.as_bytes()
            }
        };
        bytes
    }
}
