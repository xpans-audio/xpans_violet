use violet_core::{audio_input::AudioInput, audio_output::AudioOutput};
use xpans_renderconfig::mono::Mono;

use crate::{BuildInterpreter, BuildProcessor, GetDelaySamples, GetOutputChannels};

impl GetDelaySamples for Mono {
    fn get_delay_samples(&self, _sample_rate: u32) -> usize {
        0
    }
}
impl GetOutputChannels for Mono {
    fn get_output_channels(&self) -> usize {
        self.channels
    }
}

impl<T> BuildInterpreter<T> for Mono {
    type Interpretation = ();

    fn build_interpreter(
        &self,
    ) -> Box<dyn violet_core::SourceInterpreter<T, Interpretation = Self::Interpretation>> {
        Box::new(xpans_mono::Interpreter)
    }
}

impl<In, Out> BuildProcessor<In, Out> for Mono
where
    In: AudioInput<Sample = f32>,
    Out: AudioOutput<Sample = f32>,
{
    type Interpretation = ();

    fn build_processor(
        &self,
    ) -> Box<dyn violet_core::SampleProcessor<In, Out, Interpretation = Self::Interpretation> + Send>
    {
        let processor = xpans_mono::Processor::new(self.channels);
        Box::new(processor)
    }
}
