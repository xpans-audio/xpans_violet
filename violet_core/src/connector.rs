/**
An interface for all inputs and outputs to communicate to the renderer and
keep track of its state.
*/
pub trait Connector {
    /**
    The number of frames that this connector can support rendering at the
    time this method is called.

    If this connector is an input, this is the number of audio or spatial
    samples per audio source that this connector has available for reading.

    If this connector is an output, this is the number of audio samples
    per channel that this connector has the capacity to receive.
    */
    fn frames_available(&self) -> Option<usize>;
    /// The sample rate of the connector.
    fn sample_rate(&self) -> u32;
    /**
    The number of channels the connector has.

    For inputs, this should be equal to the number of audio sources within the
    scene. For outputs, this should be equal to the sample processor's output
    channels.
    */
    fn channel_count(&self) -> usize;
    /**
    This method is called on the connector whenever the renderer has
    rendered a chunk of frames.
    This is useful for tracking the exact number of frames rendered by the
    renderer.
    */
    fn advance(&mut self, frames: usize);
    /**
    This method is called on the connector before a frame is rendered.
    It provides the relative frame that is about to be rendered.

    `frame` is the *relative* frame within the current chunk of frames
    being rendered at the time, or rather, the frame since the last time
    `advance()` was called.
    */
    #[allow(unused)]
    fn pre_frame(&mut self, frame: usize) {}
    /**
    This method is called on the connector after a frame is rendered.
    It provides the relative frame that was just rendered.

    `frame` is the *relative* frame within the current chunk of frames
    being rendered at the time, or rather, the frame since the last time
    `advance()` was called.
    */
    #[allow(unused)]
    fn post_frame(&mut self, frame: usize) {}
}
impl<T> Connector for Box<T>
where
    T: ?Sized + Connector,
{
    fn pre_frame(&mut self, frame: usize) {
        self.as_mut().pre_frame(frame);
    }
    fn post_frame(&mut self, frame: usize) {
        self.as_mut().post_frame(frame);
    }
    fn sample_rate(&self) -> u32 {
        self.as_ref().sample_rate()
    }
    fn channel_count(&self) -> usize {
        self.as_ref().channel_count()
    }
    fn advance(&mut self, frames: usize) {
        self.as_mut().advance(frames);
    }
    fn frames_available(&self) -> Option<usize> {
        self.as_ref().frames_available()
    }
}
