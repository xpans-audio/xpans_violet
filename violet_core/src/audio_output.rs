//! Audio output traits
use crate::connector::Connector;

/// Receives rendered audio samples from the renderer.
pub trait AudioOutput: Connector {
    type Sample;
    /**
    Sets the current sample for a particular audio channel.

    `frame` is the *relative* frame within the current chunk of frames
    being rendered at the time, or rather, the frame since the last time
    `advance()` was called.
    */
    fn set_sample(&mut self, channel: usize, frame: usize, value: Self::Sample);

    /// The sample rate of the audio output.
    fn sample_rate(&self) -> u32;

    /// The number of output channels this output supports.
    fn channel_count(&self) -> usize;
}
impl<T> AudioOutput for Box<T>
where
    T: ?Sized + AudioOutput,
{
    type Sample = T::Sample;

    fn set_sample(&mut self, channel: usize, frame: usize, value: Self::Sample) {
        self.as_mut().set_sample(channel, frame, value);
    }

    fn sample_rate(&self) -> u32 {
        self.as_ref().sample_rate()
    }

    fn channel_count(&self) -> usize {
        self.as_ref().channel_count()
    }
}
