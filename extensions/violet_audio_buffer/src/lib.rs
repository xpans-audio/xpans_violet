/*!
Adds a buffered audio input to xpans Violet that can fetch audio samples from
the given input on a separate thread from the renderer.
*/
use std::{ops::Add, thread::JoinHandle};

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
        lookahead_length: usize,
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
        lookahead_length: usize,
    ) -> (AudioBuffer<S>, AudioBufferTask<Self, S>) {
        AudioBuffer::new(self, read_capacity, write_capacity, lookahead_length)
    }
}

/// The buffered audio input that the renderer reads from
pub struct AudioBuffer<S>
where
    S: Copy + Default,
{
    reader: MultiRingReader<S>,
    sample_rate: u32,
    lookahead_length: usize,
}

impl<S> AudioBuffer<S>
where
    S: Copy + Default,
{
    fn new<I>(
        inner: I,
        read_capacity: usize,
        write_capacity: usize,
        lookahead_length: usize,
    ) -> (Self, AudioBufferTask<I, S>)
    where
        I: AudioInput<Sample = S> + Sized,
    {
        let channels = inner.channel_count();
        let sample_rate = inner.sample_rate();

        let write_capacity = write_capacity.max(lookahead_length);

        let (reader, writer) = multi_ring_buf(channels, read_capacity, write_capacity);

        // Advance positions by read capacity to start with full delay
        // length available for reading:
        reader.advance_read_position_by(read_capacity);
        writer.advance_write_position_by(read_capacity);

        let input = Self {
            reader,
            sample_rate,
            lookahead_length,
        };
        let task = AudioBufferTask::new(inner, writer, lookahead_length);

        (input, task)
    }
}

impl<S> Connector for AudioBuffer<S>
where
    S: Copy + Default,
{
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn channel_count(&self) -> usize {
        self.reader.channels()
    }

    fn frames_available(&self) -> Option<usize> {
        let reads_available = self
            .reader
            .real_reads_available()
            .saturating_sub(self.lookahead_length);

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
}

impl<T: Default + Copy> BufferedAudioInput for AudioBuffer<T> {
    fn buffered_sample(&self, channel: usize, frame: usize, sample: isize) -> Self::Sample {
        let index = frame.cast_signed().add(sample);
        self.reader.read_relative(channel, index)
    }

    fn max_delay_length(&self) -> usize {
        self.reader.read_capacity()
    }

    fn max_lookahead_length(&self) -> usize {
        self.lookahead_length
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
    task: PrivateTask<I, S>,
    canceled: Option<Box<dyn Fn() -> bool + Send + 'static>>,
    paused: Option<Box<dyn Fn() -> bool + Send + 'static>>,
}

impl<I, S> AudioBufferTask<I, S>
where
    I: AudioInput<Sample = S>,
    S: Copy + Default,
{
    fn new(inner: I, writer: MultiRingWriter<S>, lookahead_length: usize) -> Self {
        Self {
            task: PrivateTask {
                inner,
                writer,
                lookahead_length,
            },
            canceled: None,
            paused: None,
        }
    }
    /// Returns an immutable reference to the inner audio input
    pub fn inner(&self) -> &I {
        &self.task.inner
    }

    /// Returns an mutable reference to the inner audio input
    pub fn inner_mut(&mut self) -> &mut I {
        &mut self.task.inner
    }

    /// Returns the inner audio input, dropping this `AudioBuffer`
    pub fn into_inner(self) -> I {
        self.task.inner
    }

    /**
    Cancels the buffer process whenever the given closure returns `true`.

    The maximum time between the closure returning `true` and the buffer
    process actually canceling may depend on the block size, as the closure
    is called before each block is written.
    */
    pub fn cancelable(&mut self, canceled: impl Fn() -> bool + Send + 'static) -> &mut Self {
        self.canceled = Some(Box::new(canceled));
        self
    }

    /**
    Pauses the buffer process whenever the given closure returns `true`,
    and resumes the buffer process when the closure returns `false` again.

    The maximum time between the closure returning `true` and the buffer
    process actually canceling may depend on the block size, as the closure
    is called before each block is written.
    */
    pub fn pausable(&mut self, paused: impl Fn() -> bool + Send + 'static) -> &mut Self {
        self.paused = Some(Box::new(paused));
        self
    }

    /**
    Makes the buffer process not cancelable if it was previously made
    cancelable.
    */
    pub fn not_cancelable(&mut self) -> &mut Self {
        self.canceled = None;
        self
    }

    /**
    Makes the buffer process not pausable if it was previously made
    pausable.
    */
    pub fn not_pausable(&mut self) -> &mut Self {
        self.paused = None;
        self
    }

    /**
    Runs the audio buffer process on the current thread

    Running the audio buffer process on the same thread as the renderer
    will likely not give desired results.

    `block_size` is the maximum number of samples the buffer process will
    pre-fetch before making them available to the renderer.
    */
    pub fn run(self, block_size: usize) {
        let task = self.task;
        let canceled = self.canceled;
        let paused = self.paused;

        fn always_false() -> bool {
            false
        }

        /*
        I used this match statement in hopes that the compiler will optimize
        away any unnecessary `if` statements in the buffer process, since
        `always_false` is called using static dispatch.
        */
        match (canceled, paused) {
            (None, None) => task.run(block_size, always_false, always_false),
            (None, Some(paused)) => task.run(block_size, always_false, paused),
            (Some(canceled), None) => task.run(block_size, canceled, always_false),
            (Some(canceled), Some(pausable)) => task.run(block_size, canceled, pausable),
        }
    }

    /**
    Spawns a standard library thread that runs the audio buffer process

    `block_size` is the maximum number of samples the buffer process will
    pre-fetch before making them available to the renderer.
    */
    pub fn spawn_and_run(self, block_size: usize) -> JoinHandle<()>
    where
        I: Send + 'static,
        S: 'static,
    {
        std::thread::spawn(move || self.run(block_size))
    }
}

/**
This exists to make mixing and matching the canceling and pausing closures
less painful.
*/
struct PrivateTask<I, S>
where
    I: AudioInput<Sample = S>,
    S: Copy + Default,
{
    inner: I,
    writer: MultiRingWriter<S>,
    lookahead_length: usize,
}

impl<I, S> PrivateTask<I, S>
where
    I: AudioInput<Sample = S>,
    S: Copy + Default,
{
    fn run(self, block_size: usize, canceled: impl Fn() -> bool, paused: impl Fn() -> bool) {
        buffer_process(
            self.inner,
            self.writer,
            self.lookahead_length,
            block_size,
            canceled,
            paused,
        );
    }
}

fn buffer_process<I, S>(
    mut inner: I,
    writer: MultiRingWriter<S>,
    lookahead_length: usize,
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
            for channel in 0..writer.channels() {
                let sample = inner.sample(channel, frame);
                writer.write_forward(channel, frame, sample);
            }
        }
        writer.advance_write_position_by(frames);
        inner.advance(frames);
    }

    // Write silence to fill the lookahead once the inner input has ended:
    let mut final_lookahead_frames = 0;
    while final_lookahead_frames < lookahead_length {
        let frames = writer.real_writes_available();
        for frame in 0..frames {
            for channel in 0..writer.channels() {
                let sample = S::default();
                writer.write_forward(channel, frame, sample);
            }
        }
        writer.advance_write_position_by(frames);
        final_lookahead_frames += frames;
    }
}
