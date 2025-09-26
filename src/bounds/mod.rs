use crate::coord::AxialCoord;

mod hex_bounds;

pub trait Bounds<T> {
    type Idx: Sized + Copy;
    fn coord_to_idx(&self, coord: AxialCoord) -> Self::Idx;
    fn get_by_idx(&self, idx: Self::Idx) -> Option<&T>;
}
