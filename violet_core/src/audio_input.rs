//! Audio input traits
use crate::connector::Connector;

/// Provides audio samples to the renderer.
pub trait AudioInput: Connector {
    /// The type of audio samples that this input provides.
    type Sample;
    /**
    Gets the current sample.

    `frame` is the *relative* frame within the current chunk of frames
    being rendered at the time, or rather, the frame since the last time
    `advance()` was called.
    */
    fn sample(&self, channel: usize, frame: usize) -> Self::Sample;
}
impl<T> AudioInput for Box<T>
where
    T: ?Sized + AudioInput,
{
    type Sample = T::Sample;

    fn sample(&self, channel: usize, frame: usize) -> Self::Sample {
        self.as_ref().sample(channel, frame)
    }
}

/**
An audio input that can retrieve previous audio samples using an
unsigned index.
*/
pub trait BufferedAudioInput: AudioInput {
    /**
    Gets a buffered audio sample.

    `sample` is the number of samples relative to the current sample.
    For example, a `sample` of `0` will get the current sample,
    a sample of `1` will get the sample after, `-1` will get the sample
    before, etc.

    The `sample` should not exceed or equal the `max_lookahead_length`,
    or be less than or equal to the negative of `max_delay_length`.

    `frame` is the *relative* frame within the current chunk of frames
    being rendered at the time, or rather, the frame since the last time
    `advance()` was called.
    */
    fn buffered_sample(&self, channel: usize, frame: usize, sample: isize) -> Self::Sample;
    /**
    Gets a previous audio sample.

    `sample` is the number of samples back.
    For example, a `sample` of `0` will get the current sample,
    a sample of `1` will get the sample before, etc.

    The `sample` should not exceed or equal the `max_delay_length`.

    `frame` is the *relative* frame within the current chunk of frames
    being rendered at the time, or rather, the frame since the last time
    `advance()` was called.
    */
    fn delayed_sample(&self, channel: usize, frame: usize, sample: usize) -> Self::Sample {
        let sample = -sample.cast_signed();
        self.buffered_sample(channel, frame, sample)
    }
    /**
    Gets an audio sample that occurs after the rendering frame.

    `sample` is the number of samples ahead.
    For example, a `sample` of `0` will get the current sample,
    a sample of `1` will get the sample after, etc.

    The `sample` should not exceed or equal the `max_lookahead_length`.

    `frame` is the *relative* frame within the current chunk of frames
    being rendered at the time, or rather, the frame since the last time
    `advance()` was called.
    */
    fn lookahead_sample(&self, channel: usize, frame: usize, sample: usize) -> Self::Sample {
        let sample = sample.cast_signed();
        self.buffered_sample(channel, frame, sample)
    }

    /// The maximum delay length in samples supported by this input
    fn max_delay_length(&self) -> usize;

    /// The maximum lookahead length in samples supported by this input
    fn max_lookahead_length(&self) -> usize;
}
impl<T> BufferedAudioInput for Box<T>
where
    T: ?Sized + BufferedAudioInput,
{
    fn buffered_sample(&self, channel: usize, frame: usize, sample: isize) -> Self::Sample {
        self.as_ref().buffered_sample(channel, frame, sample)
    }
    fn delayed_sample(&self, channel: usize, frame: usize, sample: usize) -> Self::Sample {
        self.as_ref().delayed_sample(channel, frame, sample)
    }
    fn lookahead_sample(&self, channel: usize, frame: usize, sample: usize) -> Self::Sample {
        self.as_ref().lookahead_sample(channel, frame, sample)
    }
    fn max_delay_length(&self) -> usize {
        self.as_ref().max_delay_length()
    }
    fn max_lookahead_length(&self) -> usize {
        self.as_ref().max_lookahead_length()
    }
}

/**
An audio input that can retrieve audio samples using a fractional
(typically floating-point) index.

Interpolated inputs will usually implement this.
*/
pub trait FractionalAudioInput: BufferedAudioInput {
    /**
    Gets an audio sample using a fractional index.

    Similar to `BufferedAudioInput`,
    `sample` is the number of samples back.

    `frame` is the *relative* frame within the current chunk of frames
    being rendered at the time, or rather, the frame since the last time
    `advance()` was called.
    */
    fn fractional_sample(&self, channel: usize, frame: usize, sample: Self::Sample)
    -> Self::Sample;
}
impl<T> FractionalAudioInput for Box<T>
where
    T: ?Sized + FractionalAudioInput,
{
    fn fractional_sample(
        &self,
        channel: usize,
        frame: usize,
        sample: Self::Sample,
    ) -> Self::Sample {
        self.as_ref().fractional_sample(channel, frame, sample)
    }
}
