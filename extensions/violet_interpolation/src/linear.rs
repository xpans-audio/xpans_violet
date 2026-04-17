//! Linear interpolation
use num::{Float, cast::AsPrimitive};

use violet_core::{
    Connector,
    audio_input::{AudioInput, BufferedAudioInput, FractionalAudioInput},
};

/**
Wraps a `BufferedAudioInput` in a `LinearInterpolator`.

This is auto-implemented on all implementors of `BufferedAudioInput`.
*/
pub trait WithLinearInterpolation: Sized + BufferedAudioInput {
    fn with_linear_interpolation(self) -> LinearInterpolator<Self>;
}
impl<T: Sized + BufferedAudioInput> WithLinearInterpolation for T {
    fn with_linear_interpolation(self) -> LinearInterpolator<Self> {
        LinearInterpolator { input: self }
    }
}

/// Applies linear interpolation to the inner input.
pub struct LinearInterpolator<In>
where
    In: BufferedAudioInput,
{
    input: In,
}
impl<In> LinearInterpolator<In>
where
    In: BufferedAudioInput,
{
    pub fn new(input: In) -> Self {
        Self { input }
    }
    pub fn into_inner(self) -> In {
        self.input
    }
}
impl<In: AudioInput> Connector for LinearInterpolator<In>
where
    In: BufferedAudioInput,
{
    fn advance(&mut self, frames: usize) {
        self.input.advance(frames);
    }

    fn frames_available(&self) -> Option<usize> {
        self.input.frames_available()
    }
}
impl<In: AudioInput> AudioInput for LinearInterpolator<In>
where
    In: BufferedAudioInput,
{
    type Sample = In::Sample;

    fn sample(&self, channel: usize, frame: usize) -> Self::Sample {
        self.input.sample(channel, frame)
    }

    fn sample_rate(&self) -> u32 {
        self.input.sample_rate()
    }

    fn channel_count(&self) -> usize {
        self.input.channel_count()
    }
}
impl<In> BufferedAudioInput for LinearInterpolator<In>
where
    In: BufferedAudioInput,
{
    fn buffered_sample(&self, channel: usize, frame: usize, sample: usize) -> Self::Sample {
        self.input.buffered_sample(channel, frame, sample)
    }
}
impl<In> FractionalAudioInput for LinearInterpolator<In>
where
    In: BufferedAudioInput,
    In::Sample: Float + AsPrimitive<usize>,
{
    fn fractional_sample(
        &self,
        channel: usize,
        frame: usize,
        sample: Self::Sample,
    ) -> Self::Sample {
        let floor = sample.floor().as_();
        let ceil = sample.ceil().as_();
        let floor_sample = self.buffered_sample(channel, frame, floor);
        let ceil_sample = self.buffered_sample(channel, frame, ceil);
        let fract = sample.fract();
        lerp(floor_sample, ceil_sample, fract)
    }
}
fn lerp<T: Float>(v0: T, v1: T, t: T) -> T {
    return (T::one() - t) * v0 + t * v1;
}
