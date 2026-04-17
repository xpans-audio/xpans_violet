use std::ops::Mul;

use num::cast::AsPrimitive;
use xpans_render::input::{BufferedInput, FractionalInput, SampleRate};

use crate::audio_input::{AudioInput, BufferedAudioInput, FractionalAudioInput};

/// Holds the audio input and additional state used to read samples from it.
pub struct AudioInputSocket<Input>
where
    Input: AudioInput,
{
    input: Input,
    pub(crate) channel: usize,
    pub(crate) frame: usize,
}

impl<Input> AudioInputSocket<Input>
where
    Input: AudioInput,
{
    pub(crate) fn new(input: Input) -> Self {
        Self {
            input,
            channel: 0,
            frame: 0,
        }
    }
    pub(crate) fn input(&self) -> &Input {
        &self.input
    }
    pub(crate) fn input_mut(&mut self) -> &mut Input {
        &mut self.input
    }
}

impl<Input> xpans_render::input::Input<Input::Sample> for AudioInputSocket<Input>
where
    Input: AudioInput,
{
    fn current_sample(&self) -> Input::Sample {
        self.input.sample(self.channel, self.frame)
    }
}

impl<Input: AudioInput> SampleRate for AudioInputSocket<Input> {
    fn sample_rate(&self) -> u32 {
        self.input().sample_rate() as u32
    }
}

impl<Input> BufferedInput<Input::Sample> for AudioInputSocket<Input>
where
    Input: BufferedAudioInput,
{
    fn integer_sample(&self, index: usize) -> Input::Sample {
        self.input.buffered_sample(self.channel, self.frame, index)
    }
}
impl<Input> FractionalInput<Input::Sample, Input::Sample> for AudioInputSocket<Input>
where
    Input: FractionalAudioInput,
    Input::Sample: Copy + 'static + Mul<Output = Input::Sample>,
    usize: AsPrimitive<Input::Sample>,
{
    fn fractional_sample(&self, index: Input::Sample) -> Input::Sample {
        self.input
            .fractional_sample(self.channel, self.frame, index)
    }
}
