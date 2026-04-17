/*!
Provides an audio encoder output for xpans Violet

## Supported Formats
The Violet Audio Encoder only supports WAV encoding. Support for other formats
is planned.
*/
use std::{fs::File, io::BufWriter, ops::AddAssign, sync::mpsc::Sender, thread::JoinHandle};

use hound::{WavSpec, WavWriter};
use wreath::{Reader, RingReader, RingWriter, Writer, ring_buf};

use violet_core::{Connector, audio_output::AudioOutput};

/**
Stores metadata about the audio stream.

Splits into an `AudioEncoder` and `AudioEncoderTask`.
*/
pub struct AudioEncoderInfo {
    file: File,
    sample_rate: u32,
    channels: u16,
    duration: usize,
}

/**
Enum that the audio encoder process sends to inform other processes of its
progress.
*/
pub enum Progress {
    /**
    A sample was written.
    Contains the written sample number with the target duration.
    */
    Sample(usize, usize),
    /// Encoding completed successfully.
    Finished,
    /// Encoding failed.
    Failed,
}

impl AudioEncoderInfo {
    /**
    Creates an `AudioEncoderInfo`.

    `duration` is the *target* duration of the audio stream.
    */
    pub fn new(file: File, sample_rate: u32, channels: u16, duration: usize) -> Self {
        Self {
            file,
            sample_rate,
            channels,
            duration,
        }
    }
    /// Returns the sample rate of the audio stream.
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    /// Returns the number of channels in the audio stream.
    pub fn channels(&self) -> usize {
        usize::from(self.channels)
    }
    /// Returns the target duration in samples of the audio stream.
    pub fn duration(&self) -> u64 {
        self.duration as u64
    }
    /**
    Splits this struct into an `AudioEncoder` and `AudioEncoderTask`.

    The resulting `AudioEncoder` is the output you will give the renderer,
    and the `AudioEncoderTask` allows you to spawn the encoder task that
    can encode the stream on a seperate thread.
    */
    pub fn into_pair(self, write_capacity: usize) -> (AudioEncoder<f32>, AudioEncoderTask) {
        let (ring_reader, ring_writer) = ring_buf(1, write_capacity);
        let audio_output = AudioEncoder {
            writer: ring_writer,
            channels: self.channels,
        };
        let task = AudioEncoderTask {
            reader: ring_reader,
            info: self,
        };
        (audio_output, task)
    }
}

/// Contains data necessary to start the audio encoder process.
pub struct AudioEncoder<T> {
    writer: RingWriter<T>,
    channels: u16,
}

impl<T: Default + Copy + AddAssign> Connector for AudioEncoder<T> {
    fn pre_frame(&mut self, frame: usize) {
        for channel in 0..self.channel_count() {
            let index = interleaved_index(self.channel_count(), frame, channel);
            self.writer.write_forward(index, T::default());
        }
    }
    fn advance(&mut self, frames: usize) {
        self.writer
            .advance_write_position_by(frames * self.channel_count());
    }

    fn frames_available(&self) -> Option<usize> {
        let writes_available = self.writer.real_writes_available();
        if (writes_available == 0) && self.writer.is_closed() {
            return None;
        }
        Some(writes_available / self.channel_count())
    }
}
impl<T: Default + Copy + AddAssign> AudioOutput for AudioEncoder<T> {
    type Sample = T;

    fn set_sample(&mut self, channel: usize, frame: usize, value: Self::Sample) {
        let index = interleaved_index(self.channel_count(), frame, channel);
        let sample = self.writer.mutate_forward(index);
        *sample += value;
    }

    fn channel_count(&self) -> usize {
        self.channels as usize
    }
}
fn interleaved_index(channel_count: usize, frame: usize, channel: usize) -> usize {
    let frame_start = channel_count * frame;
    channel + frame_start
}

/// Contains data necessary to start the audio encoder process.
pub struct AudioEncoderTask {
    reader: RingReader<f32>,
    info: AudioEncoderInfo,
}

impl AudioEncoderTask {
    /// Returns the inner `AudioEncoderInfo`.
    pub fn info(&self) -> &AudioEncoderInfo {
        &self.info
    }
    /// Spawns a standard library thread that runs the audio encoder process.
    pub fn spawn_and_run(
        self,
        block_size: usize,
        cancelled: impl Fn() -> bool + Send + 'static,
        paused: impl Fn() -> bool + Send + 'static,
        progress_sender: Sender<Progress>,
    ) -> JoinHandle<()> {
        std::thread::spawn(move || self.run(block_size, cancelled, paused, progress_sender))
    }
    /**
    Run the audio encoder process on the current thread.

    Running the audio encoder process on the same thread as the renderer
    will likely not give desired results.
    */
    pub fn run(
        self,
        block_size: usize,
        cancelled: impl Fn() -> bool,
        paused: impl Fn() -> bool,
        progress_sender: Sender<Progress>,
    ) {
        output_writer_process(
            self.info,
            self.reader,
            cancelled,
            paused,
            block_size,
            progress_sender,
        );
    }
}

fn output_writer_process(
    info: AudioEncoderInfo,
    reader: RingReader<f32>,
    cancelled: impl Fn() -> bool,
    paused: impl Fn() -> bool,
    block_size: usize,
    progress_sender: Sender<Progress>,
) {
    let file = BufWriter::new(info.file);
    let spec = WavSpec {
        channels: info.channels,
        sample_rate: info.sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut wav_writer = WavWriter::new(file, spec).unwrap();
    let total_samples = info.duration * wav_writer.spec().channels as usize;
    while reader.read_position() < total_samples {
        if cancelled() {
            break;
        }
        if !reader.read_is_available() || paused() {
            continue;
        }
        let read_position = reader.read_position();
        let reads_available = reader.real_reads_available();
        let read_len = reads_available.min(block_size);
        for i in 0..read_len {
            let sample = reader.read_forward(i);
            let _ = wav_writer.write_sample(sample);
            let _ = progress_sender.send(Progress::Sample(read_position + i, total_samples));
        }
        reader.advance_read_position_by(read_len);
    }
    let _ = progress_sender.send(Progress::Finished);
    let _ = wav_writer.finalize();
}
