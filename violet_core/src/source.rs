use xpans::{Extent, Position};
use xpans_spe::{SetExtent, SetPosition};

/// Spatial data for an audio source.
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct Source<T> {
    pub pos_x: T,
    pub pos_y: T,
    pub pos_z: T,
    pub ext_x: T,
    pub ext_y: T,
    pub ext_z: T,
}

impl<T: Copy> Position<T> for Source<T> {
    fn pos_x(&self) -> T {
        self.pos_x
    }

    fn pos_y(&self) -> T {
        self.pos_y
    }

    fn pos_z(&self) -> T {
        self.pos_z
    }
}

impl<T: Copy> Extent<T> for Source<T> {
    fn ext_x(&self) -> T {
        self.ext_x
    }

    fn ext_y(&self) -> T {
        self.ext_y
    }

    fn ext_z(&self) -> T {
        self.ext_z
    }
}

impl<V> SetPosition<V> for Source<V> {
    fn set_pos_x(&mut self, x: V) {
        self.pos_x = x
    }

    fn set_pos_y(&mut self, y: V) {
        self.pos_y = y
    }

    fn set_pos_z(&mut self, z: V) {
        self.pos_z = z
    }
}

impl<V> SetExtent<V> for Source<V> {
    fn set_ext_x(&mut self, x: V) {
        self.ext_x = x
    }

    fn set_ext_y(&mut self, y: V) {
        self.ext_y = y
    }

    fn set_ext_z(&mut self, z: V) {
        self.ext_z = z
    }
}
