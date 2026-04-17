use violet_core::{SampleProcessor, SourceInterpreter};
use violet_core::{audio_input::AudioInput, audio_output::AudioOutput};
use xpans_renderconfig::stereo::{Stereo, StereoMode};

use crate::{BuildInterpreter, BuildProcessor, GetDelaySamples, GetOutputChannels, pan_law};

impl BuildInterpreter<f32> for Stereo {
    type Interpretation = f32;

    fn build_interpreter(
        &self,
    ) -> Box<dyn SourceInterpreter<f32, Interpretation = Self::Interpretation>> {
        match self.mode {
            StereoMode::Directional => Box::new(xpans_stereo::Directional::default()),
            StereoMode::Positional => Box::new(xpans_stereo::Positional::default()),
        }
    }
}

impl GetOutputChannels for Stereo {
    fn get_output_channels(&self) -> usize {
        2
    }
}

impl GetDelaySamples for Stereo {
    fn get_delay_samples(&self, _sample_rate: u32) -> usize {
        0
    }
}

impl<In, Out> BuildProcessor<In, Out> for Stereo
where
    In: AudioInput<Sample = f32>,
    Out: AudioOutput<Sample = f32>,
{
    type Interpretation = f32;

    fn build_processor(
        &self,
    ) -> Box<dyn SampleProcessor<In, Out, Interpretation = Self::Interpretation> + Send> {
        Box::new(xpans_stereo::Processor::new(pan_law(self.pan_law)))
    }
}
