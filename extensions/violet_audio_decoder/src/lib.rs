/*!
Adds an audio decoder input to xpans Violet

## Supported formats
All formats supported by [Symphonia](https://github.com/pdeljanov/Symphonia)
should work. Integer samples are converted to floating-point through
Symphonia as well.

## Example
```rust, no_run
use std::fs::File;
use violet_audio_decoder::AudioDecoder;

// Open the audio file:
let file = File::open("audio.wav").unwrap();

// Create an audio decoder that decodes to `f32` samples:
let audio_decoder = AudioDecoder::<f32>::new(file);

// You would then use `audio_decoder` as the audio input for the renderer.
```
*/

use symphonia::core::{
    audio::{AudioBuffer, Signal},
    codecs::{Decoder, DecoderOptions},
    conv::FromSample,
    formats::{FormatOptions, FormatReader},
    io::{MediaSource, MediaSourceStream},
    meta::MetadataOptions,
    probe::Hint,
    sample::{Sample, i24, u24},
};

use violet_core::{Connector, audio_input::AudioInput};

pub struct AudioDecoder<T>
where
    T: AudioDecoderSample,
{
    format_reader: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
    current_buffer: AudioBuffer<T>,
    frame_in_buffer: usize,
    ended: bool,
}

impl<T> AudioDecoder<T>
where
    T: AudioDecoderSample,
{
    /// Creates an `AudioDecoder` using a Symphonia media source.
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

        let mut format_reader = probed.format;

        let track = format_reader.default_track().unwrap();

        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &decoder_opts)
            .unwrap();

        let track_id = track.id;

        let current_buffer = read_packet(&mut format_reader, &mut decoder, track_id).unwrap();

        Self {
            format_reader,
            decoder,
            track_id,
            current_buffer,
            frame_in_buffer: 0,
            ended: false,
        }
    }
    /// Returns the sample rate of the audio stream.
    pub fn sample_rate(&self) -> u32 {
        self.decoder.as_ref().codec_params().sample_rate.unwrap()
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

    fn read_next_packet(&mut self) {
        let maybe_buffer = read_packet(&mut self.format_reader, &mut self.decoder, self.track_id);

        if let Some(buffer) = maybe_buffer {
            self.current_buffer = buffer;
            self.frame_in_buffer = 0;
        } else {
            self.ended = true;
        }
    }
}

fn read_packet<T>(
    format_reader: &mut Box<dyn FormatReader>,
    decoder: &mut Box<dyn Decoder>,
    track_id: u32,
) -> Option<AudioBuffer<T>>
where
    T: AudioDecoderSample,
{
    while let Ok(packet) = format_reader.next_packet() {
        if packet.track_id() != track_id {
            continue;
        }

        let unconverted = decoder.decode(&packet).unwrap();
        let mut buf = unconverted.make_equivalent::<T>();
        unconverted.convert(&mut buf);

        return Some(buf);
    }
    None
}

impl<T> Connector for AudioDecoder<T>
where
    T: AudioDecoderSample,
{
    fn channel_count(&self) -> usize {
        self.channels()
    }
    fn sample_rate(&self) -> u32 {
        self.sample_rate()
    }

    fn frames_available(&self) -> Option<usize> {
        if self.ended {
            return None;
        }
        Some(self.current_buffer.frames() - self.frame_in_buffer)
    }

    fn advance(&mut self, frames: usize) {
        self.frame_in_buffer += frames;
        if self.frame_in_buffer >= self.current_buffer.frames() {
            self.read_next_packet()
        }
    }
}

impl<T: AudioDecoderSample> AudioInput for AudioDecoder<T> {
    type Sample = T;

    fn sample(&self, channel: usize, frame: usize) -> Self::Sample {
        let frame = self.frame_in_buffer + frame;
        self.current_buffer.chan(channel)[frame]
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
