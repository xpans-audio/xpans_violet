/*!
Adds a spatial decoder input to xpans Violet

## Supported Formats
The only supported format is xpans Spatial Record (XSR), as it is the only
format in the xpans Ecosystem at the time of writing. In the future, more
formats and spatial codecs will be supported.
*/
use std::{marker::PhantomData, thread::JoinHandle};

use wreath::{Reader, RingReader, RingWriter, Writer, ring_buf};
use xpans_xsr::SpatialSampleMap;

use violet_core::{Connector, Source, spatial_input::SpatialInput};

/**
Stores metadata about the spatial stream.
Splits into a `SpatialDecoder` and `SpatialDecoderTask`.
*/
pub struct SpatialDecoderInfo<Scene, T>
where
    Scene: AsRef<SpatialSampleMap<usize, u16, T>>,
{
    scene: Scene,
    source_count: usize,
    duration: usize,
    phantom_data: PhantomData<T>,
}

impl<Scene, T> SpatialDecoderInfo<Scene, T>
where
    T: Copy + Default,
    Scene: AsRef<SpatialSampleMap<usize, u16, T>>,
{
    /**
    Creates a `SpatialDecoderInfo` using a xpans Spatial Record (XSR) map.

    The source count and duration (in samples) of the scene must also be
    given.
    */
    pub fn new(scene: Scene, source_count: usize, duration: usize) -> Self {
        Self {
            scene,
            source_count,
            duration,
            phantom_data: PhantomData,
        }
    }
    /**
    Splits this struct into a `SpatialDecoder` and `SpatialDecoderTask`.

    The resulting `SpatialDecoder` is the input you will give the renderer,
    and the `SpatialDecoderTask` allows you to spawn the decoder task that
    can decode the stream on a seperate thread.
    */
    pub fn into_pair(
        self,
        write_capacity: usize,
    ) -> (SpatialDecoder<T>, SpatialDecoderTask<Scene, T>) {
        let (reader, writer) = ring_buf(1, write_capacity);
        let decoder = SpatialDecoder {
            reader,
            source_count: self.source_count,
        };
        let task = SpatialDecoderTask { writer, info: self };
        (decoder, task)
    }
}

/// Contains data necessary to start the spatial decoder process.
pub struct SpatialDecoderTask<Scene, T>
where
    Scene: AsRef<SpatialSampleMap<usize, u16, T>>,
{
    writer: RingWriter<Source<T>>,
    info: SpatialDecoderInfo<Scene, T>,
}

impl<Scene, T> SpatialDecoderTask<Scene, T>
where
    T: Default + Copy,
    Scene: AsRef<SpatialSampleMap<usize, u16, T>>,
{
    /// Returns the inner `SpatialDecoderInfo`.
    pub fn info(&self) -> &SpatialDecoderInfo<Scene, T> {
        &self.info
    }
    /**
    Run the spatial decoder process on the current thread.

    Running the spatial decoder process on the same thread as the renderer
    will likely not give desired results.
    */
    pub fn run(self, cancelled: impl Fn() -> bool, paused: impl Fn() -> bool) {
        spatial_decoder_process(
            self.info.scene,
            self.info.source_count,
            self.info.duration,
            self.writer,
            cancelled,
            paused,
        );
    }
}

impl<Scene, T> SpatialDecoderTask<Scene, T>
where
    T: Default + Copy + Send + 'static,
    Scene: AsRef<SpatialSampleMap<usize, u16, T>> + Send + 'static,
{
    /// Spawns a standard library thread that runs the spatial decoder process.
    pub fn spawn_and_run(
        self,
        cancelled: impl Fn() -> bool + Send + 'static,
        paused: impl Fn() -> bool + Send + 'static,
    ) -> JoinHandle<()> {
        std::thread::spawn(move || self.run(cancelled, paused))
    }
}

/// The actual spatial input that the renderer gets spatial data from.
pub struct SpatialDecoder<T> {
    reader: RingReader<Source<T>>,
    source_count: usize,
}

impl<T: Default + Copy> Connector for SpatialDecoder<T> {
    fn advance(&mut self, frames: usize) {
        self.reader
            .advance_read_position_by(frames * self.source_count);
    }

    fn frames_available(&self) -> Option<usize> {
        let reads_available = self.reader.real_reads_available() / self.source_count;
        if (reads_available == 0) && self.reader.is_closed() {
            return None;
        }
        Some(reads_available)
    }
}
impl<T: Default + Copy> SpatialInput for SpatialDecoder<T> {
    type Scalar = T;

    fn source(&self, source: usize, frame: usize) -> Source<Self::Scalar> {
        let frame = frame * self.source_count;
        let index = frame + source;
        self.reader.read_forward(index)
    }

    fn source_count(&self) -> usize {
        self.source_count
    }
}

// fn interleaved_index(channel_count: usize, frame: usize, channel: usize) -> usize {
//     let frame_start = channel_count * frame;
//     channel + frame_start
// }

fn spatial_decoder_process<Scene, T>(
    spatial_scene: Scene,
    source_count: usize,
    duration: usize,
    writer: RingWriter<Source<T>>,
    cancelled: impl Fn() -> bool,
    paused: impl Fn() -> bool,
) where
    T: Default + Copy,
    Scene: AsRef<SpatialSampleMap<usize, u16, T>>,
{
    let mut frame = 0usize;
    let mut sources = vec![Source::default(); source_count];
    while frame < duration {
        if cancelled() {
            break;
        }
        if !writer.writes_are_available(source_count) || paused() {
            continue;
        }
        if let Some(events) = spatial_scene.as_ref().get(&frame) {
            for event in events {
                apply_changes(&mut sources[event.id as usize], &event.changes);
            }
        }
        for (i, source) in sources.iter().enumerate() {
            writer.write_forward(i, *source);
        }
        writer.advance_write_position_by(source_count);
        frame += 1;
    }
}

fn apply_changes<T: Copy>(source: &mut Source<T>, changes: &xpans_xsr::Changes<T>) {
    changes.pos_x.map(|v| source.pos_x = v);
    changes.pos_y.map(|v| source.pos_y = v);
    changes.pos_z.map(|v| source.pos_z = v);
    changes.ext_x.map(|v| source.ext_x = v);
    changes.ext_y.map(|v| source.ext_y = v);
    changes.ext_z.map(|v| source.ext_z = v);
}
