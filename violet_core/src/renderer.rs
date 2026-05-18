use crate::audio_input::AudioInput;
use crate::audio_output::AudioOutput;
use crate::spatial_input::SpatialInput;
use crate::{
    SampleProcessor, SourceInterpreter, audio_input_socket::AudioInputSocket,
    audio_output_socket::AudioOutputSocket,
};

/**
Renders spatial audio scenes.

A renderer is constructed using a `RendererBuilder`.
*/
pub struct Renderer<Interpreter, Processor, AudioIn, SpatialIn, AudioOut>
where
    AudioIn: AudioInput,
    SpatialIn: SpatialInput<Scalar = AudioIn::Sample>,
    AudioOut: AudioOutput,
    Interpreter: SourceInterpreter<AudioIn::Sample, Interpretation = Processor::Interpretation>,
    Processor: SampleProcessor<AudioIn, AudioOut>,
{
    pub(crate) interpreter: Interpreter,
    pub(crate) processor: Processor,
    pub(crate) audio_input: AudioInputSocket<AudioIn>,
    pub(crate) spatial_input: SpatialIn,
    pub(crate) audio_output: AudioOutputSocket<AudioOut>,
    pub(crate) interpretations: Box<[Interpreter::Interpretation]>,
}

impl<Interpreter, Processor, AudioIn, SpatialIn, AudioOut>
    Renderer<Interpreter, Processor, AudioIn, SpatialIn, AudioOut>
where
    AudioIn: AudioInput,
    SpatialIn: SpatialInput<Scalar = AudioIn::Sample>,
    AudioOut: AudioOutput,
    Interpreter: SourceInterpreter<AudioIn::Sample, Interpretation = Processor::Interpretation>,
    Processor: SampleProcessor<AudioIn, AudioOut>,
{
    fn interpret_sources(&mut self, frame: usize) {
        let length = self.interpreter.interpretation_length();
        for (source, result) in self.interpretations.chunks_exact_mut(length).enumerate() {
            let source = self.spatial_input.source(source, frame);
            self.interpreter.interpret_source(&source, result);
        }
    }
    fn process_samples(&mut self, frame: usize) {
        self.set_frame(frame);
        let length = self.interpreter.interpretation_length();
        for (channel, interpretation) in self.interpretations.chunks_exact(length).enumerate() {
            self.audio_input.channel = channel;
            self.processor.process_samples(
                interpretation,
                &self.audio_input,
                &mut self.audio_output,
            );
        }
    }
    fn pre_frame(&mut self, frame: usize) {
        self.audio_input.input_mut().pre_frame(frame);
        self.spatial_input.pre_frame(frame);
        self.audio_output.output_mut().pre_frame(frame);
    }
    fn post_frame(&mut self, frame: usize) {
        self.audio_input.input_mut().post_frame(frame);
        self.spatial_input.post_frame(frame);
        self.audio_output.output_mut().post_frame(frame);
    }
    /// Updates the frame number stored in the input and output sockets.
    fn set_frame(&mut self, frame: usize) {
        self.audio_input.frame = frame;
        self.audio_output.frame = frame;
    }
    /**
    Tries to render a single frame, returning `None` if that frame was not
    available.
    */
    pub fn render_frame(&mut self) -> Option<()> {
        self.render_frames(1)
            .and_then(|frames| (frames == 1).then_some(()))
    }
    /**
    Tries to render the given amount of frames and returns the amount of
    frames actually rendered.

    If more frames are requested than there are available, this method will
    only render the available frames.
    */
    pub fn render_frames(&mut self, frames: usize) -> Option<usize> {
        let frames = frames.min(self.frames_available()?);
        self.render_frames_unchecked(frames);
        Some(frames)
    }
    /**
    Renders the given amount of frames without checking if that amount of
    frames are available for rendering.

    Rendering more frames than are available can have wildly unpredictable
    results depending on the renderer's inputs and outputs.
    */
    fn render_frames_unchecked(&mut self, frames: usize) {
        for frame in 0..frames {
            self.pre_frame(frame);
            self.interpret_sources(frame);
            self.process_samples(frame);
            self.post_frame(frame);
        }
        self.advance(frames);
    }
    /**
    Renders the frames that are available for rendering, returning `None`
    if no frames were available.
    */
    pub fn render_available_frames(&mut self) -> Option<usize> {
        let frames = self.frames_available()?;
        self.render_frames_unchecked(frames);
        Some(frames)
    }
    /// Returns the number of audio sources in the rendering scene.
    pub fn source_count(&self) -> usize {
        self.audio_input().channel_count()
    }
    /**
    Returns the amount of frames available for rendering. Returns `None` if
    this renderer has finished rendering and will have no more frames
    available.
    */
    pub fn frames_available(&self) -> Option<usize> {
        let audio_in = self.audio_input().frames_available()?;
        let spatial_in = self.spatial_input.frames_available()?;
        let audio_out = self.audio_output().frames_available()?;
        Some(audio_in.min(spatial_in).min(audio_out))
    }
    /// Advances all connectors.
    fn advance(&mut self, frames: usize) {
        self.audio_input_mut().advance(frames);
        self.spatial_input.advance(frames);
        self.audio_output_mut().advance(frames);
    }
    /// Returns the channel count of the audio output/sample processor.
    pub fn output_channels(&self) -> usize {
        self.audio_output().channel_count()
    }

    /// Returns an immutable reference to the audio input.
    pub fn audio_input(&self) -> &AudioIn {
        self.audio_input.input()
    }
    /// Returns a mutable reference to the audio input.
    pub fn audio_input_mut(&mut self) -> &mut AudioIn {
        self.audio_input.input_mut()
    }
    /// Returns an immutable reference to the sample processor.
    pub fn sample_processor(&self) -> &Processor {
        &self.processor
    }
    /// Returns a mutable reference to the sample processor.
    pub fn sample_processor_mut(&mut self) -> &mut Processor {
        &mut self.processor
    }
    /// Returns an immutable reference to the source interpreter.
    pub fn interpreter(&self) -> &Interpreter {
        &self.interpreter
    }
    /// Returns a mutable reference to the source interpreter.
    pub fn interpreter_mut(&mut self) -> &mut Interpreter {
        &mut self.interpreter
    }
    /// Returns an immutable reference to the spatial input.
    pub fn spatial_input(&self) -> &SpatialIn {
        &self.spatial_input
    }
    /// Returns a mutable reference to the spatial input.
    pub fn spatial_input_mut(&mut self) -> &mut SpatialIn {
        &mut self.spatial_input
    }
    /// Returns an immutable reference to the audio output.
    pub fn audio_output(&self) -> &AudioOut {
        self.audio_output.output()
    }
    /// Returns a mutable reference to the audio output.
    pub fn audio_output_mut(&mut self) -> &mut AudioOut {
        self.audio_output.output_mut()
    }
    /// Returns an immutable reference to the inner slice of calculated interpretations.
    pub fn interpretations(&self) -> &[Interpreter::Interpretation] {
        &self.interpretations
    }
    /// Returns a mutable reference to the inner slice of calculated interpretations.
    pub fn interpretations_mut(&mut self) -> &mut Box<[Interpreter::Interpretation]> {
        &mut self.interpretations
    }
}
