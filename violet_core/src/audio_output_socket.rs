use xpans_render::output::Output;

use crate::audio_output::AudioOutput;

/// Holds the audio output and additional state used to write samples to it.
pub struct AudioOutputSocket<AudioOut>
where
    AudioOut: AudioOutput,
{
    output: AudioOut,
    pub(crate) frame: usize,
}

impl<AudioOut> AudioOutputSocket<AudioOut>
where
    AudioOut: AudioOutput,
{
    pub(crate) fn new(output: AudioOut) -> Self {
        Self { output, frame: 0 }
    }
    pub(crate) fn output(&self) -> &AudioOut {
        &self.output
    }
    pub(crate) fn output_mut(&mut self) -> &mut AudioOut {
        &mut self.output
    }
}
impl<AudioOut> Output<AudioOut::Sample> for AudioOutputSocket<AudioOut>
where
    AudioOut: AudioOutput,
{
    fn set_channel(&mut self, channel: usize, sample: AudioOut::Sample) {
        self.output.set_sample(channel, self.frame, sample);
    }
}
