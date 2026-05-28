//! Lagrange interpolation
use std::ops::{AddAssign, MulAssign, Range};

use num::{Float, Zero, cast::AsPrimitive};

use violet_core::{
    Connector,
    audio_input::{AudioInput, BufferedAudioInput, FractionalAudioInput},
};

/**
Wraps a `BufferedAudioInput` in a `LagrangeInterpolator`.

This is auto-implemented on all implementors of `BufferedAudioInput`.
*/
pub trait WithLagrangeInterpolation: Sized + BufferedAudioInput {
    /**
    `order` is the order of the lagrange polynomial to use for interpolation.

    For example, passing in `3` will make the interpolator use third order
    lagrange polynomial interpolation, which means the interpolator will use
    a total of four points.
    */
    fn with_lagrange_interpolation(self, order: usize) -> LagrangeInterpolator<Self>;
}
impl<T: Sized + BufferedAudioInput> WithLagrangeInterpolation for T {
    fn with_lagrange_interpolation(self, order: usize) -> LagrangeInterpolator<Self> {
        LagrangeInterpolator { input: self, order }
    }
}

/// Applies lagrange interpolation to the inner input.
pub struct LagrangeInterpolator<In>
where
    In: BufferedAudioInput,
{
    input: In,
    order: usize,
}
impl<In> LagrangeInterpolator<In>
where
    In: BufferedAudioInput,
{
    /**
    `order` is the order of the lagrange polynomial to use for interpolation.

    For example, passing in `3` will make the interpolator use third order
    lagrange polynomial interpolation, which means the interpolator will use
    a total of four points.
    */
    pub fn new(input: In, order: usize) -> Self {
        Self { input, order }
    }
    pub fn into_inner(self) -> In {
        self.input
    }
}
impl<In: AudioInput> Connector for LagrangeInterpolator<In>
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
impl<In: AudioInput> AudioInput for LagrangeInterpolator<In>
where
    In: BufferedAudioInput,
{
    type Sample = In::Sample;

    fn sample(&self, channel: usize, frame: usize) -> Self::Sample {
        self.input.sample(channel, frame)
    }
}
impl<In> BufferedAudioInput for LagrangeInterpolator<In>
where
    In: BufferedAudioInput,
{
    fn buffered_sample(&self, channel: usize, frame: usize, sample: isize) -> Self::Sample {
        self.input.buffered_sample(channel, frame, sample)
    }
    /*
    For `max_delay_length` and `max_lookahead_length`:

    No need to subtract the order or points since getting
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
impl<In> FractionalAudioInput for LagrangeInterpolator<In>
where
    In: BufferedAudioInput,
    In::Sample: Float + AsPrimitive<usize> + AsPrimitive<isize> + Zero + MulAssign + AddAssign,
    isize: AsPrimitive<Self::Sample>,
{
    fn fractional_sample(
        &self,
        channel: usize,
        frame: usize,
        sample: Self::Sample,
    ) -> Self::Sample {
        let index = sample;
        let floor: isize = index.floor().as_();

        let [start, end] = index_range(floor, self.order);

        let mut sum = Self::Sample::zero();

        for point in start..end {
            let sample = self.buffered_sample(channel, frame, point);
            sum += sample * lagrange([start, end], point, -index)
        }

        sum
    }
}

fn lagrange<T>(range: [isize; 2], point: isize, x: T) -> T
where
    T: Float + 'static + MulAssign,
    isize: AsPrimitive<T>,
{
    let [first, last] = get_exclusive_ranges(range, point);

    let mut numerator = T::one();
    let mut denominator = T::one();
    for i in first.chain(last) {
        numerator *= x - i.as_();
        denominator *= point.as_() - i.as_();
    }

    numerator / denominator
}
/// Gets two ranges, one with all values before `point`
/// and one with all values after `point`. Both exclude `point`.
fn get_exclusive_ranges(range: [isize; 2], point: isize) -> [Range<isize>; 2] {
    let first = range[0]..point;
    let last = point + 1..range[1];
    [first, last]
}

/// Gets the index range for lagrange interpolation.
///
/// Instead of returning actual ranges, we use arrays since
/// `Range` does not implement the `Copy` trait.
fn index_range(floor: isize, order: usize) -> [isize; 2] {
    let order = order.cast_signed();

    // `>> 1` divides by `2`.
    let order_a = (order >> 1) + 1;
    let order_b = (order + 1) >> 1;

    let start = -floor - order_a;
    let end = -floor + order_b;

    [start, end]
}

#[cfg(test)]
#[test]
fn index_range_works_zero() {
    // First order:
    let range = index_range(0, 1);
    assert_eq!(range[0], -1);
    assert_eq!(range[1], 1);

    // Second order:
    let range = index_range(0, 2);
    assert_eq!(range[0], -2);
    assert_eq!(range[1], 1);

    // Third order:
    let range = index_range(0, 3);
    assert_eq!(range[0], -2);
    assert_eq!(range[1], 2);
}

#[cfg(test)]
#[test]
fn index_range_works_offset() {
    // First order:
    let range = index_range(3, 1);
    assert_eq!(range[0], -4);
    assert_eq!(range[1], -2);

    // Second order:
    let range = index_range(3, 2);
    assert_eq!(range[0], -5);
    assert_eq!(range[1], -2);

    // Third order:
    let range = index_range(3, 3);
    assert_eq!(range[0], -5);
    assert_eq!(range[1], -1);
}
