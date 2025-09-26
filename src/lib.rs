//! A library for representing hex grids

pub mod bounds;
pub mod coord;
use std::{array, marker::PhantomData, mem::MaybeUninit, num::NonZeroU32, ptr, slice};

use crate::{
    bounds::Bounds,
    coord::{AxialCoord, HecsCoord, HexCoord, HexCoordBase},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HexFace {
    Right,
    UpRight,
    UpLeft,
    Left,
    DownLeft,
    DownRight,
}

impl HexFace {
    /// Returns the index of `self` within `Self::variants()`
    #[inline]
    const fn to_idx(self) -> usize {
        use HexFace::*;
        match self {
            Right => 0,
            UpRight => 1,
            UpLeft => 2,
            Left => 3,
            DownLeft => 4,
            DownRight => 5,
        }
    }

    #[inline]
    const fn from_idx(idx: usize) -> Self {
        Self::variants()[idx]
    }

    /// All variants in counter-clockwise order, starting with
    #[inline]
    pub const fn variants() -> [Self; 6] {
        use HexFace::*;
        const { [Right, UpRight, UpLeft, Left, DownLeft, DownRight] }
    }

    /// Positive is a counter-clockwise rotation, negative is a clockwise rotation
    #[inline]
    pub const fn rotate(self, n: i32) -> Self {
        Self::from_idx((self.to_idx() as i32 + n).rem_euclid(6) as usize)
    }
}

pub trait Wrapping<T> {
    //
}

struct ArrayBuilder<const N: usize, T> {
    buf: [MaybeUninit<T>; N],
    len: usize,
}

impl<const N: usize, T> ArrayBuilder<N, T> {
    const fn new() -> Self {
        Self {
            buf: [const { MaybeUninit::uninit() }; _],
            len: 0,
        }
    }

    const fn push(&mut self, val: T) {
        assert!(self.len < N);
        self.buf[self.len].write(val);
        self.len += 1;
    }

    const fn build(self) -> [T; N] {
        if self.len == N {
            unsafe { ptr::read(&self.buf as *const [MaybeUninit<T>; N] as *const [T; N]) }
        } else {
            panic!("Failed to build `ArrayBuilder`")
        }
    }
}

/// The bounded range is a rectangle,
/// with a width and a height
pub struct RectangleBounds<T> {
    pub width: u32,
    pub height: u32,
    bot_left: HecsCoord,
    top_right: HecsCoord,
    store: Vec<T>,
}

impl<T> Bounds<T> for RectangleBounds<T> {
    type Idx = usize;

    fn coord_to_idx(&self, coord: AxialCoord) -> Self::Idx {
        // coord.to_hecs()
        todo!()
    }

    fn get_by_idx(&self, idx: Self::Idx) -> Option<&T> {
        todo!()
    }
}

// pub struct HexGrid<T, B: Bounds> {
//     bounds: B,
//     _p: PhantomData<T>,
// }

// impl<T, B: Bounds> HexGrid<T, B> {
//     pub fn new(bounds: B) -> Self {
//         Self {
//             bounds,
//             _p: PhantomData,
//         }
//     }
// }
