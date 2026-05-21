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
}
impl<T> AudioOutput for Box<T>
where
    T: ?Sized + AudioOutput,
{
    type Sample = T::Sample;

    fn set_sample(&mut self, channel: usize, frame: usize, value: Self::Sample) {
        self.as_mut().set_sample(channel, frame, value);
    }
}
