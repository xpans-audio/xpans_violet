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
    /// The sample rate of the audio input.
    fn sample_rate(&self) -> u32;
    /**
    The number of audio channels the audio input is providing.
    This should be equal to the number of audio sources within the scene.
    */
    fn channel_count(&self) -> usize;
}
impl<T> AudioInput for Box<T>
where
    T: ?Sized + AudioInput,
{
    type Sample = T::Sample;

    fn sample(&self, channel: usize, frame: usize) -> Self::Sample {
        self.as_ref().sample(channel, frame)
    }

    fn sample_rate(&self) -> u32 {
        self.as_ref().sample_rate()
    }

    fn channel_count(&self) -> usize {
        self.as_ref().channel_count()
    }
}

/**
An audio input that can retrieve previous audio samples using an
unsigned index.
*/
pub trait BufferedAudioInput: AudioInput {
    /**
    Gets a previous audio sample.

    `sample` is the number of samples back.
    For example, a `sample` of `0` will get the current sample,
    a sample of `1` will get the sample before, etc.

    `frame` is the *relative* frame within the current chunk of frames
    being rendered at the time, or rather, the frame since the last time
    `advance()` was called.
    */
    fn buffered_sample(&self, channel: usize, frame: usize, sample: usize) -> Self::Sample;
}
impl<T> BufferedAudioInput for Box<T>
where
    T: ?Sized + BufferedAudioInput,
{
    fn buffered_sample(&self, channel: usize, frame: usize, sample: usize) -> Self::Sample {
        self.as_ref().buffered_sample(channel, frame, sample)
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
