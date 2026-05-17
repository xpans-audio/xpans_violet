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
use violet_audio_decoder::AudioDecoder;

// Open the audio file:
let file = File::open("audio.wav").unwrap();

// Create an audio decoder that decodes to `f32` samples:
let audio_decoder = AudioDecoder::<f32>::new(file);

// You would then use `audio_decoder` as the audio input for the renderer.
```
