use xpans_render::prelude::InterpretSource;

use crate::Source;

/// Evaluates an audio source's properties into values used by a sample processor.
pub trait SourceInterpreter<T>
where
    Self: InterpretSource<Source<T>>,
{
}
impl<T, S> SourceInterpreter<S> for T where T: ?Sized + InterpretSource<Source<S>> {}
