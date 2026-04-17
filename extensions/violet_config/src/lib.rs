/*!
Generates source interpreters and sample processors from render configurations
in xpans Violet.
*/
mod headphones;
mod mono;
mod stereo;

use xpans_common_lr::{Linear, Sine, SquareRoot};

use violet_core::{SampleProcessor, SourceInterpreter};
use violet_core::{audio_input::AudioInput, audio_output::AudioOutput};
use xpans_renderconfig::PanLaw;

pub trait GetOutputChannels {
    fn get_output_channels(&self) -> usize;
}
pub trait GetDelaySamples {
    fn get_delay_samples(&self, sample_rate: u32) -> usize;
}

fn pan_law(pan_law: xpans_renderconfig::PanLaw) -> Box<dyn xpans_common_lr::PanLaw<f32> + Send> {
    match pan_law {
        PanLaw::Linear => Box::new(Linear),
        PanLaw::SquareRoot => Box::new(SquareRoot),
        PanLaw::Sine => Box::new(Sine),
    }
}

pub trait BuildInterpreter<T> {
    type Interpretation;
    fn build_interpreter(
        &self,
    ) -> Box<dyn SourceInterpreter<T, Interpretation = Self::Interpretation>>;
}

pub trait BuildProcessor<In, Out>
where
    In: AudioInput,
    Out: AudioOutput,
{
    type Interpretation;
    fn build_processor(
        &self,
    ) -> Box<dyn SampleProcessor<In, Out, Interpretation = Self::Interpretation> + Send>;
}
