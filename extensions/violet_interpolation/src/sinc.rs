//! Sinc interpolation
use std::ops::{AddAssign, Range};

use num::{Float, Zero, cast::AsPrimitive, traits::FloatConst};

use violet_core::{
    Connector,
    audio_input::{AudioInput, BufferedAudioInput, FractionalAudioInput},
};

/**
Wraps a `BufferedAudioInput` in a `SincInterpolator`.

This is auto-implemented on all implementors of `BufferedAudioInput`.
*/
pub trait WithSincInterpolation: Sized + BufferedAudioInput {
    /**
    `half_window_length` is the size of the sinc window divided by `2`.
    For example, to get a sinc window of `128`, you would pass in `64`.
    */
    fn with_sinc_interpolation(self, half_window_length: usize) -> SincInterpolator<Self>;
}
impl<T: Sized + BufferedAudioInput> WithSincInterpolation for T {
    fn with_sinc_interpolation(self, half_window_length: usize) -> SincInterpolator<Self> {
        SincInterpolator {
            input: self,
            half_window_length,
        }
    }
}

/// Applies sinc interpolation to the inner input.
pub struct SincInterpolator<In>
where
    In: BufferedAudioInput,
{
    input: In,
    half_window_length: usize,
}
impl<In> SincInterpolator<In>
where
    In: BufferedAudioInput,
{
    /**
    `half_window_length` is the size of the sinc window divided by `2`.
    For example, to get a sinc window of `128`, you would pass in `64`.
    */
    pub fn new(input: In, half_window_length: usize) -> Self {
        Self {
            input,
            half_window_length,
        }
    }
    pub fn into_inner(self) -> In {
        self.input
    }
}
impl<In: AudioInput> Connector for SincInterpolator<In>
where
    In: BufferedAudioInput,
{
    fn sample_rate(&self) -> u32 {
        self.input.sample_rate()
    }

    fn channel_count(&self) -> usize {
        self.input.channel_count()
    }

    fn advance(&mut self, frames: usize) {
        self.input.advance(frames);
    }

    fn frames_available(&self) -> Option<usize> {
        self.input.frames_available()
    }
}
impl<In: AudioInput> AudioInput for SincInterpolator<In>
where
    In: BufferedAudioInput,
{
    type Sample = In::Sample;

    fn sample(&self, channel: usize, frame: usize) -> Self::Sample {
        self.input.sample(channel, frame)
    }
}
impl<In> BufferedAudioInput for SincInterpolator<In>
where
    In: BufferedAudioInput,
{
    fn buffered_sample(&self, channel: usize, frame: usize, sample: isize) -> Self::Sample {
        self.input.buffered_sample(channel, frame, sample)
    }
    /*
    For `max_delay_length` and `max_lookahead_length`:

    No need to subtract the half_window_length since getting
    buffered samples doesn't use interpolation and therefore
    doesn't require space around the desired sample.
    */
    fn max_delay_length(&self) -> usize {
        self.input.max_delay_length()
    }

    fn max_lookahead_length(&self) -> usize {
        self.input.max_lookahead_length()
    }
}
impl<In> FractionalAudioInput for SincInterpolator<In>
where
    In: BufferedAudioInput,
    In::Sample: Float + FloatConst + AsPrimitive<usize> + AsPrimitive<isize> + Zero + AddAssign,
    isize: AsPrimitive<In::Sample>,
{
    fn fractional_sample(
        &self,
        channel: usize,
        frame: usize,
        sample: Self::Sample,
    ) -> Self::Sample {
        let index = sample;
        let floor: isize = index.floor().as_();

        let range = index_range(floor, self.half_window_length);

        let mut sum = Self::Sample::zero();

        for i in range {
            let sample = self.buffered_sample(channel, frame, i);
            sum += sample * sinc(AsPrimitive::<Self::Sample>::as_(i) + index);
        }

        sum
    }
}

/// Gets the index range for sinc interpolation.
fn index_range(floor: isize, half_window_length: usize) -> Range<isize> {
    let half_window_length = half_window_length.cast_signed();

    let start = -floor - half_window_length;
    let end = -floor + half_window_length;

    Range { start, end }
}

/// The normalized sinc function
fn sinc<T: Float + FloatConst>(x: T) -> T {
    if x == T::zero() {
        return T::one();
    }
    let pi_times_x = T::PI() * x;
    pi_times_x.sin() / pi_times_x
}

#[cfg(test)]
#[test]
fn index_range_works_zero() {
    let range = index_range(0, 4);
    assert_eq!(range.start, -4);
    assert_eq!(range.end, 4);
}

#[cfg(test)]
#[test]
fn index_range_works_offset() {
    let range = index_range(4, 4);
    assert_eq!(range.start, -8);
    assert_eq!(range.end, 0);
}
