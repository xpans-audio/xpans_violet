use xpans_render::process::ProcessSamples;

use crate::audio_input::AudioInput;
use crate::audio_output::AudioOutput;
use crate::{audio_input_socket::AudioInputSocket, audio_output_socket::AudioOutputSocket};

/// Uses source interpretations to transform audio samples.
pub trait SampleProcessor<In, Out>
where
    In: AudioInput,
    Out: AudioOutput,
    Self: ProcessSamples<AudioInputSocket<In>, AudioOutputSocket<Out>>,
{
}
impl<T, In, Out> SampleProcessor<In, Out> for T
where
    In: AudioInput,
    Out: AudioOutput,
    T: ?Sized + ProcessSamples<AudioInputSocket<In>, AudioOutputSocket<Out>>,
{
}
