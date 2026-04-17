/*!
Adds an audio decoder input to xpans Violet

## Supported formats
All formats supported by [Symphonia](https://github.com/pdeljanov/Symphonia)
should work. Integer samples are converted to floating-point through
Symphonia as well.

## Example
```rust, no_run
use std::fs::File;
use violet_audio_decoder::AudioDecoderInfo;

let file = File::open("audio.wav").unwrap();
let decoder_info = AudioDecoderInfo::new(file);

// The amount of samples the audio buffer will retain (i.e. for delayed samples)
let read_capacity = 64;

// The amount of samples the decoder process will be able to write ahead of the
// renderer.
let write_capacity = 64;

let (audio_decoder, audio_decoder_task) =
    decoder_info.into_pair::<f32>(read_capacity, write_capacity);

// Run the decoder process on a new thread.
// We use closures that always return false, so this process will not be
// pausable or cancelable.
audio_decoder_task.spawn_and_run(8, || false, || false);

// You would then use `audio_decoder` as the audio input for the renderer.
```
*/

use std::thread::JoinHandle;

use symphonia::core::{
    audio::Signal,
    codecs::{Decoder, DecoderOptions},
    conv::FromSample,
    formats::{FormatOptions, FormatReader},
    io::{MediaSource, MediaSourceStream},
    meta::MetadataOptions,
    probe::Hint,
    sample::{Sample, i24, u24},
};
use wreath::{MultiRingReader, MultiRingWriter, Reader, Writer, multi_ring_buf};

use violet_core::{
    Connector,
    audio_input::{AudioInput, BufferedAudioInput},
};

/// The actual audio input that the renderer gets samples from.
pub struct AudioDecoder<T> {
    reader: MultiRingReader<T>,
    sample_rate: u32,
}

/// Stores metadata about the audio stream.
/// Splits into an `AudioDecoder` and `AudioDecoderTask`.
pub struct AudioDecoderInfo {
    format_reader: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
}

impl AudioDecoderInfo {
    /// Creates an `AudioDecoderInfo` using a Symphonia media source.
    pub fn new(file: impl MediaSource + 'static) -> Self {
        let file = Box::new(file);
        let mss = MediaSourceStream::new(file, Default::default());

        let hint = Hint::new();

        let format_opts: FormatOptions = Default::default();
        let metadata_opts: MetadataOptions = Default::default();
        let decoder_opts: DecoderOptions = Default::default();

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &format_opts, &metadata_opts)
            .unwrap();

        let format = probed.format;

        let track = format.default_track().unwrap();

        let decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &decoder_opts)
            .unwrap();

        let track_id = track.id;
        Self {
            format_reader: format,
            decoder,
            track_id,
        }
    }
    /// Returns the sample rate of the audio stream.
    pub fn sample_rate(&self) -> u32 {
        self.decoder.as_ref().codec_params().sample_rate.unwrap() as u32
    }
    /// Returns the number of channels in the audio stream.
    pub fn channels(&self) -> usize {
        self.decoder
            .as_ref()
            .codec_params()
            .channels
            .unwrap()
            .count()
    }
    /// Returns the duration in samples of the audio stream.
    pub fn duration(&self) -> u64 {
        self.decoder.as_ref().codec_params().n_frames.unwrap()
    }
    /// Splits this struct into an `AudioDecoder` and `AudioDecoderTask`.
    ///
    /// The resulting `AudioDecoder` is the input you will give the renderer,
    /// and the `AudioDecoderTask` allows you to spawn the decoder task that
    /// can decode the stream on a seperate thread.
    pub fn into_pair<T>(
        self,
        read_len: usize,
        write_len: usize,
    ) -> (AudioDecoder<T>, AudioDecoderTask<T>)
    where
        T: Copy + Default,
    {
        let channels = self.decoder.codec_params().channels.unwrap().count();
        let sample_rate = self.decoder.codec_params().sample_rate.unwrap();
        let (reader, writer) = multi_ring_buf(channels, read_len, write_len);
        // Advance positions by read capacity to start with full delay
        // length available:
        reader.advance_read_position_by(read_len);
        writer.advance_write_position_by(read_len);

        let info = self;
        let task = AudioDecoderTask { writer, info };
        let connection = AudioDecoder {
            reader,
            sample_rate,
        };
        (connection, task)
    }
}

/// Contains data necessary to start the audio decoder process.
pub struct AudioDecoderTask<T> {
    writer: MultiRingWriter<T>,
    info: AudioDecoderInfo,
}

impl<T> AudioDecoderTask<T>
where
    T: AudioDecoderSample + 'static,
{
    /// Returns the inner `AudioDecoderInfo`.
    pub fn info(&self) -> &AudioDecoderInfo {
        &self.info
    }
    /// Spawns a standard library thread that runs the audio decoder process.
    pub fn spawn_and_run(
        self,
        block_size: usize,
        cancelled: impl Fn() -> bool + Send + 'static,
        paused: impl Fn() -> bool + Send + 'static,
    ) -> JoinHandle<()> {
        std::thread::spawn(move || self.run(block_size, cancelled, paused))
    }
    /// Run the audio decoder process on the current thread.
    /// Running the audio decoder process on the same thread as the renderer
    /// will likely not give desired results.
    pub fn run(self, block_size: usize, cancelled: impl Fn() -> bool, paused: impl Fn() -> bool) {
        audio_decoder_process(
            self.writer,
            self.info.decoder,
            self.info.track_id,
            self.info.format_reader,
            cancelled,
            paused,
            block_size,
        );
    }
}

impl<T: Default + Copy> Connector for AudioDecoder<T> {
    fn advance(&mut self, frames: usize) {
        self.reader.advance_read_position_by(frames);
    }

    fn frames_available(&self) -> Option<usize> {
        let reads_available = self.reader.real_reads_available();
        if (reads_available == 0) && self.reader.is_closed() {
            return None;
        }
        Some(reads_available)
    }
}
impl<T: Default + Copy> AudioInput for AudioDecoder<T> {
    type Sample = T;

    fn sample(&self, channel: usize, frame: usize) -> Self::Sample {
        self.reader.read_forward(channel, frame)
    }
    fn channel_count(&self) -> usize {
        self.reader.channels()
    }
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}
impl<T: Default + Copy> BufferedAudioInput for AudioDecoder<T> {
    fn buffered_sample(&self, channel: usize, frame: usize, sample: usize) -> Self::Sample {
        let frame = frame.cast_signed();
        let index = frame.saturating_sub_unsigned(sample);
        self.reader.read_relative(channel, index)
    }
}

pub trait AudioDecoderSample
where
    Self: Sample
        + FromSample<f32>
        + FromSample<f64>
        + FromSample<i16>
        + FromSample<i24>
        + FromSample<i32>
        + FromSample<i8>
        + FromSample<u16>
        + FromSample<u24>
        + FromSample<u32>
        + FromSample<u8>,
{
}
impl AudioDecoderSample for f32 {}
impl AudioDecoderSample for f64 {}

fn audio_decoder_process<T>(
    writer: MultiRingWriter<T>,
    mut decoder: Box<dyn Decoder>,
    track_id: u32,
    mut format_reader: Box<dyn FormatReader>,
    cancelled: impl Fn() -> bool,
    _paused: impl Fn() -> bool,
    block_size: usize,
) where
    T: AudioDecoderSample,
{
    let total_channels = decoder.codec_params().channels.unwrap().count();
    let track_id = track_id;
    while let Ok(packet) = format_reader.next_packet() {
        if packet.track_id() != track_id {
            continue;
        }
        if cancelled() {
            break;
        }

        let unconverted = decoder.decode(&packet).unwrap();
        let mut buf = unconverted.make_equivalent::<T>();
        unconverted.convert(&mut buf);

        let mut frame_in_packet = 0;
        let packet_duration = packet.block_dur() as usize;
        while frame_in_packet < packet_duration {
            let frames_left_in_packet = packet_duration - frame_in_packet;
            let ideal_writes = frames_left_in_packet.min(block_size);
            let available_writes = writer.real_writes_available();
            let writes = ideal_writes.min(available_writes);
            for channel in 0..total_channels {
                for i in 0..writes {
                    let sample = buf.chan(channel)[frame_in_packet + i];
                    writer.write_forward(channel, i, sample);
                }
            }
            frame_in_packet += writes;
            writer.advance_write_position_by(writes);
        }
    }
}
