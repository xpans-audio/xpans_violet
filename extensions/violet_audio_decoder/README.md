# xpans Violet Audio Decoder
Provides an audio decoder input for xpans Violet.

[![Crates.io Version](https://img.shields.io/crates/v/violet_audio_decoder)](https://crates.io/crates/violet_audio_decoder)
[![docs.rs](https://img.shields.io/docsrs/violet_audio_decoder)](https://docs.rs/violet_audio_decoder/0.1.0/violet_audio_decoder/)

## Supported formats
All formats supported by [Symphonia](https://github.com/pdeljanov/Symphonia)
should work. Integer samples are converted to floating-point through 
Symphonia as well.

## Example
```rust
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
