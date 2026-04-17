/*!
A spatial audio rendering engine for the xpans Ecosystem

## Connectors
Violet manages its inputs and output through composable data structures
called Connectors. A Connector tells the renderer how many frames that it has
available for reading or writing, which determines how many frames that Violet
can safely render at a time. Upon rendering, the renderer communicates back to
its connectors the number of frames it has rendered.

## Inputs and outputs
Violet provides several optional inputs and outputs through Cargo features.
Custom inputs and outputs may also be built by implementing their respective
traits.

## The renderer
Rendering can be done frame-by-frame or in chunks. When frames are rendered,
audio samples are read from the audio input, spatial samples are read from the
spatial input, and the rendered audio samples are written to the audio output.

The inputs and outputs can call methods on themselves at different stages of
the rendering process to keep track of state, clean up before or after frames,
etc.
*/
pub use violet_core::*;

pub mod audio_input {
    /*!
    Audio input traits and types

    Extensions from cargo features that add audio inputs are re-exported here.
    */
    pub use violet_core::audio_input::*;

    #[cfg(feature = "interpolation")]
    pub use violet_interpolation as interpolation;

    #[cfg(feature = "audio_decoder")]
    pub use violet_audio_decoder as audio_decoder;
}

pub mod audio_output {
    /*!
    Audio output traits and types

    Extensions from cargo features that add audio outputs are re-exported here.
    */
    pub use violet_core::audio_output::*;

    #[cfg(feature = "audio_encoder")]
    pub use violet_audio_encoder as audio_encoder;
}

pub mod spatial_input {
    /*!
    Spatial input traits and types

    Extensions from cargo features that add spatial inputs are re-exported here.
    */
    pub use violet_core::spatial_input::*;

    #[cfg(feature = "spatial_decoder")]
    pub use violet_spatial_decoder as spatial_decoder;
}

#[cfg(feature = "config")]
pub mod config {
    /*!
    Extension for generating source interpreters and sample processors from render configurations
    */
    pub use violet_config::*;
}
