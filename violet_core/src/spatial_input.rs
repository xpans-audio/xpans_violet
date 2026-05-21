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
}
impl<T> SpatialInput for Box<T>
where
    T: ?Sized + SpatialInput,
{
    type Scalar = T::Scalar;

    fn source(&self, source: usize, frame: usize) -> Source<Self::Scalar> {
        self.as_ref().source(source, frame)
    }
}
