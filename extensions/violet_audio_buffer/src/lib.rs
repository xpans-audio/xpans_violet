/*!
Adds a buffered audio input to xpans Violet that can fetch audio samples from
the given input on a separate thread from the renderer.
*/
use std::thread::JoinHandle;

use violet_core::{
    Connector,
    audio_input::{AudioInput, BufferedAudioInput},
};
use wreath::{MultiRingReader, MultiRingWriter, Reader, Writer, multi_ring_buf};

/**
Wraps an audio input in an `AudioBuffer`.
*/
pub trait Buffer<S>
where
    Self: AudioInput<Sample = S> + Sized,
    S: Copy + Default,
{
    fn buffer(
        self,
        read_capacity: usize,
        write_capacity: usize,
    ) -> (AudioBuffer<S>, AudioBufferTask<Self, S>);
}

impl<S, T> Buffer<S> for T
where
    Self: AudioInput<Sample = S> + Sized,
    S: Copy + Default,
{
    fn buffer(
        self,
        read_capacity: usize,
        write_capacity: usize,
    ) -> (AudioBuffer<S>, AudioBufferTask<Self, S>) {
        AudioBuffer::new(self, read_capacity, write_capacity)
    }
}

/// The buffered audio input that the renderer reads from
pub struct AudioBuffer<S>
where
    S: Copy + Default,
{
    reader: MultiRingReader<S>,
    sample_rate: u32,
}

impl<S> AudioBuffer<S>
where
    S: Copy + Default,
{
    fn new<I>(
        inner: I,
        read_capacity: usize,
        write_capacity: usize,
    ) -> (Self, AudioBufferTask<I, S>)
    where
        I: AudioInput<Sample = S> + Sized,
    {
        let channels = inner.channel_count();
        let sample_rate = inner.sample_rate();

        let (reader, writer) = multi_ring_buf(channels, read_capacity, write_capacity);

        // Advance positions by read capacity to start with full delay
        // length available for reading:
        reader.advance_read_position_by(read_capacity);
        writer.advance_write_position_by(read_capacity);

        let input = Self {
            reader,
            sample_rate,
        };
        let task = AudioBufferTask { inner, writer };

        (input, task)
    }
}

impl<S> Connector for AudioBuffer<S>
where
    S: Copy + Default,
{
    fn frames_available(&self) -> Option<usize> {
        let reads_available = self.reader.real_reads_available();
        if (reads_available == 0) && self.reader.is_closed() {
            return None;
        }
        Some(reads_available)
    }

    fn advance(&mut self, frames: usize) {
        self.reader.advance_read_position_by(frames);
    }
}

impl<S> AudioInput for AudioBuffer<S>
where
    S: Copy + Default,
{
    type Sample = S;

    fn sample(&self, channel: usize, frame: usize) -> Self::Sample {
        self.reader.read_forward(channel, frame)
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn channel_count(&self) -> usize {
        self.reader.channels()
    }
}

impl<T: Default + Copy> BufferedAudioInput for AudioBuffer<T> {
    fn buffered_sample(&self, channel: usize, frame: usize, sample: usize) -> Self::Sample {
        let frame = frame.cast_signed();
        let index = frame.saturating_sub_unsigned(sample);
        self.reader.read_relative(channel, index)
    }
}

/**
Stores the inner audio input with the buffer's writer and is used to start
the buffer process
*/
pub struct AudioBufferTask<I, S>
where
    I: AudioInput<Sample = S>,
    S: Copy + Default,
{
    inner: I,
    writer: MultiRingWriter<S>,
}

impl<I, S> AudioBufferTask<I, S>
where
    I: AudioInput<Sample = S>,
    S: Copy + Default,
{
    /// Returns an immutable reference to the inner audio input
    pub fn inner(&self) -> &I {
        &self.inner
    }

    /// Returns an mutable reference to the inner audio input
    pub fn inner_mut(&mut self) -> &mut I {
        &mut self.inner
    }

    /// Returns the inner audio input, dropping this `AudioBuffer`
    pub fn into_inner(self) -> I {
        self.inner
    }

    /**
    Runs the audio buffer process on the current thread

    Running the audio buffer process on the same thread as the renderer
    will likely not give desired results.

    `block_size` is the maximum number of samples the buffer process will
    pre-fetch before making them available to the renderer.

    `canceled` and `paused` are closures that return `true` if the buffer
    process should be canceled or paused respectively. If you don't need the
    buffer process to be cancelable or pausable, you can provide a closure
    that always returns `false`.
    */
    pub fn run(self, block_size: usize, canceled: impl Fn() -> bool, paused: impl Fn() -> bool) {
        buffer_process(self.inner, self.writer, block_size, canceled, paused);
    }

    /**
    Spawns a standard library thread that runs the audio buffer process

    `block_size` is the maximum number of samples the buffer process will
    pre-fetch before making them available to the renderer.

    `canceled` and `paused` are closures that return `true` if the buffer
    process should be canceled or paused respectively. If you don't need the
    buffer process to be cancelable or pausable, you can provide a closure
    that always returns `false`.
    */
    pub fn spawn_and_run(
        self,
        block_size: usize,
        canceled: impl Fn() -> bool + Send + 'static,
        paused: impl Fn() -> bool + Send + 'static,
    ) -> JoinHandle<()>
    where
        I: Send + 'static,
        S: 'static,
    {
        std::thread::spawn(move || self.run(block_size, canceled, paused))
    }
}

fn buffer_process<I, S>(
    mut inner: I,
    writer: MultiRingWriter<S>,
    block_size: usize,
    canceled: impl Fn() -> bool,
    paused: impl Fn() -> bool,
) where
    I: AudioInput<Sample = S>,
    S: Copy + Default,
{
    while let Some(frames) = inner.frames_available() {
        if canceled() {
            break;
        }
        if paused() {
            continue;
        }
        let frames = frames.min(writer.real_writes_available()).min(block_size);
        for frame in 0..frames {
            for channel in 0..inner.channel_count() {
                let sample = inner.sample(channel, frame);
                writer.write_forward(channel, frame, sample);
            }
        }
        writer.advance_write_position_by(frames);
        inner.advance(frames);
    }
}
