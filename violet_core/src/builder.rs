use crate::audio_input::AudioInput;
use crate::audio_output::AudioOutput;
use crate::renderer::Renderer;
use crate::sample_processor::SampleProcessor;
use crate::source_interpreter::SourceInterpreter;
use crate::spatial_input::SpatialInput;
use crate::{audio_input_socket::AudioInputSocket, audio_output_socket::AudioOutputSocket};

/// Builds a renderer.
#[derive(Debug)]
pub struct RendererBuilder<Interpreter, Processor, AudioIn, SpatialIn, AudioOut> {
    interpreter: Option<Interpreter>,
    processor: Option<Processor>,
    audio_input: Option<AudioIn>,
    spatial_input: Option<SpatialIn>,
    audio_output: Option<AudioOut>,
}
impl<Interpreter, Processor, AudioIn, SpatialIn, AudioOut> Default
    for RendererBuilder<Interpreter, Processor, AudioIn, SpatialIn, AudioOut>
{
    fn default() -> Self {
        Self {
            interpreter: None,
            processor: None,
            audio_input: None,
            spatial_input: None,
            audio_output: None,
        }
    }
}

impl<Interpreter, Processor, AudioIn, SpatialIn, AudioOut>
    RendererBuilder<Interpreter, Processor, AudioIn, SpatialIn, AudioOut>
where
    AudioIn: AudioInput,
    SpatialIn: SpatialInput<Scalar = AudioIn::Sample>,
    AudioOut: AudioOutput,
    Interpreter: SourceInterpreter<AudioIn::Sample, Interpretation = Processor::Interpretation>,
    Processor: SampleProcessor<AudioIn, AudioOut>,
    Interpreter::Interpretation: Default + Clone,
{
    /// Creates a new `RendererBuilder`.
    pub fn new() -> Self {
        Self::default()
    }
    /**
    Sets the spatial input of the renderer.

    The source count of the spatial input should equal the channel count of
    the audio input.
    */
    pub fn set_spatial_input(mut self, spatial_input: SpatialIn) -> Self {
        self.spatial_input = Some(spatial_input);
        self
    }
    /**
    Sets the audio input of the renderer.

    The channel count of the audio input should equal the source count of
    the spatial input.
    */
    pub fn set_audio_input(mut self, audio_input: AudioIn) -> Self {
        self.audio_input = Some(audio_input);
        self
    }
    /**
    Sets the audio output of the renderer.

    The channel count of the audio output should equal the channel count
    of the sample processor.
    */
    pub fn set_audio_output(mut self, audio_output: AudioOut) -> Self {
        self.audio_output = Some(audio_output);
        self
    }
    /// Sets the source interpreter of the renderer.
    pub fn set_source_interpreter(mut self, interpreter: Interpreter) -> Self {
        self.interpreter = Some(interpreter);
        self
    }
    /**
    Sets the sample processor of the renderer.

    The channel count of the sample processor should equal the channel
    count of the audio output.
    */
    pub fn set_sample_processor(mut self, processor: Processor) -> Self {
        self.processor = Some(processor);
        self
    }
    /**
    Attempts to build the renderer, returning a `RendererBuildError` if
    the renderer fails to build.
    */
    pub fn build(
        self,
    ) -> Result<Renderer<Interpreter, Processor, AudioIn, SpatialIn, AudioOut>, RendererBuildError>
    {
        self.inputs_compatible()?;
        self.output_compatible()?;
        let interpreter = self.interpreter.ok_or(MISSING_INTERPRETER)?;
        let audio_input = self.audio_input.ok_or(MISSING_AUDIO_INPUT)?;
        let processor = self.processor.ok_or(MISSING_PROCESSOR)?;
        let spatial_input = self.spatial_input.ok_or(MISSING_SPATIAL_INPUT)?;
        let audio_output = self.audio_output.ok_or(MISSING_AUDIO_OUTPUT)?;
        let count = audio_input.channel_count();
        let len = interpreter.interpretation_length();
        let interpretations =
            vec![Interpreter::Interpretation::default(); count * len].into_boxed_slice();
        Ok(Renderer {
            interpreter,
            processor,
            audio_input: AudioInputSocket::new(audio_input),
            spatial_input,
            audio_output: AudioOutputSocket::new(audio_output),
            interpretations,
        })
    }
    fn inputs_compatible(&self) -> Result<(), RendererBuildError> {
        let audio_channels = self
            .audio_input
            .as_ref()
            .ok_or(MISSING_AUDIO_INPUT)?
            .channel_count();
        let source_count = self
            .spatial_input
            .as_ref()
            .ok_or(MISSING_SPATIAL_INPUT)?
            .source_count();
        if audio_channels != source_count {
            return Err(RendererBuildError::InputChannelMismatch);
        }
        Ok(())
    }
    fn output_compatible(&self) -> Result<(), RendererBuildError> {
        let processor_samples = self
            .processor
            .as_ref()
            .ok_or(MISSING_PROCESSOR)?
            .output_channels();
        let output_channels = self
            .audio_output
            .as_ref()
            .ok_or(MISSING_AUDIO_OUTPUT)?
            .channel_count();
        if processor_samples != output_channels {
            return Err(RendererBuildError::OutputChannelMismatch);
        }
        Ok(())
    }
}

const MISSING_INTERPRETER: RendererBuildError = RendererBuildError::MissingInterpreter;
const MISSING_PROCESSOR: RendererBuildError = RendererBuildError::MissingProcessor;
const MISSING_AUDIO_INPUT: RendererBuildError = RendererBuildError::MissingAudioInput;
const MISSING_AUDIO_OUTPUT: RendererBuildError = RendererBuildError::MissingAudioOutput;
const MISSING_SPATIAL_INPUT: RendererBuildError = RendererBuildError::MissingSpatialInput;
#[derive(Debug, Clone, Copy)]
pub enum RendererBuildError {
    MissingInterpreter,
    MissingProcessor,
    MissingAudioInput,
    MissingSpatialInput,
    MissingAudioOutput,
    InputChannelMismatch,
    OutputChannelMismatch,
}
