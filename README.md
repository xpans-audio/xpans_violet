# xpans Violet
A spatial audio rendering engine for the xpans Ecosystem

[![Crates.io Version](https://img.shields.io/crates/v/xpans_violet)](https://crates.io/crates/xpans_violet)
[![docs.rs](https://img.shields.io/docsrs/xpans_violet)](https://docs.rs/xpans_violet/0.1.0/xpans_violet/)

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
