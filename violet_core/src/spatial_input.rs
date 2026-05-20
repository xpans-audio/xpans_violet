//! Spatial input traits
use crate::{Connector, Source};

/// Provides spatial samples to the renderer.
pub trait SpatialInput: Connector {
    type Scalar;
    /**
    Gets the current spatial data of a particular audio source.

    `frame` is the *relative* frame within the current chunk of frames
    being rendered at the time, or rather, the frame since the last time
    `advance()` was called.
    */
    fn source(&self, source: usize, frame: usize) -> Source<Self::Scalar>;
    /// The sample rate of the spatial input.
    fn sample_rate(&self) -> u32;
    /// The number of audio sources the spatial input is providing.
    fn source_count(&self) -> usize;
}
impl<T> SpatialInput for Box<T>
where
    T: ?Sized + SpatialInput,
{
    type Scalar = T::Scalar;

    fn source(&self, source: usize, frame: usize) -> Source<Self::Scalar> {
        self.as_ref().source(source, frame)
    }

    fn sample_rate(&self) -> u32 {
        self.as_ref().sample_rate()
    }

    fn source_count(&self) -> usize {
        self.as_ref().source_count()
    }
}
