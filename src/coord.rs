use std::ops::{Add, Mul, Neg, Sub};

use crate::HexFace;

pub trait HexCoordBase:
    Clone
    + Add<Output = Self>
    + Neg<Output = Self>
    + Sub<Output = Self>
    + Mul<i32, Output = Self>
    + Sized
{
    fn from_axial(axial: AxialCoord) -> Self;
    fn to_axial(self) -> AxialCoord;

    fn from_hecs(hecs: HecsCoord) -> Self {
        Self::from_axial(hecs.to_axial())
    }
    fn to_hecs(self) -> HecsCoord {
        HecsCoord::from_axial(self.to_axial())
    }

    fn from_offset(offset: OffsetCoord) -> Self {
        Self::from_axial(offset.to_axial())
    }
    fn to_offset(self) -> OffsetCoord {
        OffsetCoord::from_axial(self.to_axial())
    }

    fn distance(self, other: Self) -> i32 {
        self.to_axial().distance(other.to_axial())
    }
}

macro_rules! derive_ops_for_coord {
    ($coord_type:ty) => {
        impl Add for $coord_type {
            type Output = Self;

            #[inline]
            fn add(self, rhs: Self) -> Self::Output {
                self.add(rhs)
            }
        }

        impl Neg for $coord_type {
            type Output = Self;

            #[inline]
            fn neg(self) -> Self::Output {
                self.neg()
            }
        }

        impl Sub for $coord_type {
            type Output = Self;

            #[inline]
            fn sub(self, rhs: Self) -> Self::Output {
                self.sub(rhs)
            }
        }

        impl Mul<i32> for $coord_type {
            type Output = Self;

            #[inline]
            fn mul(self, rhs: i32) -> Self::Output {
                self.scale(rhs)
            }
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AxialCoord {
    /// In the right direction
    pub q: i32,
    /// In the down-right direction
    pub r: i32,
}

impl AxialCoord {
    #[inline]
    pub const fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    #[inline]
    pub const fn add(self, rhs: Self) -> Self {
        Self::new(self.q + rhs.q, self.r + rhs.r)
    }

    /// Returns `self` reflected across the origin
    #[inline]
    pub const fn neg(self) -> Self {
        Self::new(-self.q, -self.r)
    }

    #[inline]
    pub const fn sub(self, rhs: Self) -> Self {
        self.add(rhs.neg())
    }

    #[inline]
    pub const fn scale(self, scale: i32) -> Self {
        Self::new(self.q * scale, self.r * scale)
    }

    #[inline]
    pub const fn direction_offset(self, dir: HexFace) -> Self {
        match dir {
            HexFace::Right => Self::new(1, 0),
            HexFace::UpRight => Self::new(1, -1),
            HexFace::UpLeft => Self::new(0, -1),
            HexFace::Left => Self::new(-1, 0),
            HexFace::DownLeft => Self::new(-1, 1),
            HexFace::DownRight => Self::new(0, 1),
        }
    }

    #[inline]
    pub const fn neighbor(self, dir: HexFace) -> Self {
        self.add(self.direction_offset(dir))
    }

    /// Manhattan distance
    pub const fn distance(self, other: Self) -> i32 {
        (i32::abs(self.q - other.q)
            + i32::abs(self.q + self.r - other.q - other.r)
            + i32::abs(self.r - other.r))
            / 2
    }
}

derive_ops_for_coord!(AxialCoord);
impl HexCoordBase for AxialCoord {
    #[inline]
    fn from_axial(axial: AxialCoord) -> Self {
        axial
    }

    #[inline]
    fn to_axial(self) -> AxialCoord {
        self
    }
}

/// Hexagonal Efficient Coordinate System (HECS)
///
/// This is based off of the wikipedia article: https://en.wikipedia.org/wiki/Hexagonal_Efficient_Coordinate_System
///
/// HECS is very similar to Offset, and converting between the two is very cheap
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HecsCoord {
    /// Parity
    pub a: i32,
    /// Row
    pub r: i32,
    /// Column
    pub c: i32,
}

impl HecsCoord {
    #[inline]
    pub const fn new(a: i32, r: i32, c: i32) -> Self {
        Self { a, r, c }
    }

    #[inline]
    pub const fn add(self, rhs: Self) -> Self {
        Self::new(
            self.a ^ rhs.a,
            self.r + rhs.r + self.a & rhs.a,
            self.c + rhs.c + self.a & rhs.a,
        )
    }

    #[inline]
    pub const fn neg(self) -> Self {
        Self::new(self.a, -self.r - self.a, -self.c - self.a)
    }

    #[inline]
    pub const fn sub(self, rhs: Self) -> Self {
        self.add(rhs.neg())
    }

    #[inline]
    pub const fn scale(self, scale: i32) -> Self {
        let scale_is_neg = scale.is_negative();
        let scale = scale.overflowing_abs().0;
        let scaled = Self::new(
            (self.a * scale) % 2,
            scale * self.r + self.a * (scale / 2),
            scale * self.c + self.a * (scale / 2),
        );
        match scale_is_neg {
            true => scaled.neg(),
            false => scaled,
        }
    }

    #[inline]
    pub const fn neighbor(self, dir: HexFace) -> Self {
        let a = self.a;
        let nota = 1 - self.a;
        let r = self.r;
        let c = self.c;
        match dir {
            HexFace::Right => Self::new(a, r, c + 1),
            HexFace::UpRight => Self::new(nota, r - nota, c + a),
            HexFace::UpLeft => Self::new(nota, r - nota, c - nota),
            HexFace::Left => Self::new(a, r, c - 1),
            HexFace::DownLeft => Self::new(nota, r + a, c - nota),
            HexFace::DownRight => Self::new(nota, r + a, c + a),
        }
    }

    #[inline]
    pub const fn from_offset(offset: OffsetCoord) -> Self {
        let odd_row = offset.row & 1;
        Self {
            a: odd_row,
            r: (offset.row - odd_row) / 2,
            c: offset.col,
        }
    }

    #[inline]
    pub const fn to_offset(self) -> OffsetCoord {
        OffsetCoord::new(self.c, self.r * 2 + self.a)
    }
}

derive_ops_for_coord!(HecsCoord);
impl HexCoordBase for HecsCoord {
    #[inline]
    fn from_axial(axial: AxialCoord) -> Self {
        Self::from_offset(axial.to_offset())
    }

    #[inline]
    fn to_axial(self) -> AxialCoord {
        self.to_offset().to_axial()
    }

    #[inline]
    fn from_offset(offset: OffsetCoord) -> Self {
        Self::from_offset(offset)
    }
    #[inline]
    fn to_offset(self) -> OffsetCoord {
        self.to_offset()
    }
}

/// Each odd numbered row is offset
/// right by half a hexagon
///
/// This is "odd-r"
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OffsetCoord {
    /// Rightwards
    pub col: i32,
    /// Downwards
    pub row: i32,
}

impl OffsetCoord {
    #[inline]
    pub const fn new(col: i32, row: i32) -> Self {
        Self { col, row }
    }

    #[inline]
    pub const fn add(self, rhs: Self) -> Self {
        Self::from_axial(self.to_axial().add(rhs.to_axial()))
    }

    #[inline]
    pub const fn neg(self) -> Self {
        Self::from_axial(self.to_axial().neg())
    }

    #[inline]
    pub const fn sub(self, rhs: Self) -> Self {
        self.add(rhs.neg())
    }

    #[inline]
    pub const fn scale(self, scale: i32) -> Self {
        Self::from_axial(self.to_axial().scale(scale))
    }

    #[inline]
    pub const fn neighbor(self, dir: HexFace) -> Self {
        let parity = self.row & 1;
        let table = const {
            [
                // even rows
                [[1, 0], [0, -1], [-1, -1], [-1, 0], [-1, 1], [0, 1]],
                // odd rows
                [[1, 0], [1, -1], [0, -1], [-1, 0], [0, 1], [1, 1]],
            ]
        };
        let offset = table[parity as usize][dir.to_idx()];
        Self::new(self.col + offset[0], self.row + offset[1])
    }

    #[inline]
    pub const fn from_axial(axial: AxialCoord) -> Self {
        let parity = axial.r & 1;
        Self::new(axial.q + (axial.r - parity) / 2, axial.r)
    }

    #[inline]
    pub const fn to_axial(self) -> AxialCoord {
        let parity = self.row & 1;
        AxialCoord::new(self.col - (self.row - parity) / 2, self.row)
    }
}

derive_ops_for_coord!(OffsetCoord);
impl HexCoordBase for OffsetCoord {
    #[inline]
    fn from_axial(axial: AxialCoord) -> Self {
        Self::from_axial(axial)
    }

    #[inline]
    fn to_axial(self) -> AxialCoord {
        self.to_axial()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HexCoord {
    Axial(AxialCoord),
    Hecs(HecsCoord),
    Offset(OffsetCoord),
}

#[cfg(test)]
mod tests {
    use crate::{
        HexFace,
        coord::{AxialCoord, HecsCoord, HexCoordBase, OffsetCoord},
    };

    #[test]
    fn offset_convert() {
        for i in -10..10 {
            for j in -10..10 {
                let axial = AxialCoord::new(i, j);
                assert_eq!(axial, OffsetCoord::from_axial(axial).to_axial());
                let offset = OffsetCoord::new(i, j);
                assert_eq!(offset, AxialCoord::from_offset(offset).to_offset());
            }
        }
    }

    #[test]
    fn offset_travel() {
        for i in -10..10 {
            for j in -10..10 {
                for dir in HexFace::variants() {
                    let start_axial = AxialCoord::new(i, j);
                    let start_offset = OffsetCoord::from_axial(start_axial);
                    assert_eq!(
                        start_axial.neighbor(dir),
                        start_offset.neighbor(dir).to_axial()
                    );
                }
            }
        }
    }

    #[test]
    fn hecs_convert() {
        for i in -10..10 {
            for j in -10..10 {
                let axial = AxialCoord::new(i, j);
                assert_eq!(axial, HecsCoord::from_axial(axial).to_axial());
            }
        }
    }

    #[test]
    fn hecs_travel() {
        for i in -10..10 {
            for j in -10..10 {
                for dir in HexFace::variants() {
                    let start_axial = AxialCoord::new(i, j);
                    let start_offset = HecsCoord::from_axial(start_axial);
                    assert_eq!(
                        start_axial.neighbor(dir),
                        start_offset.neighbor(dir).to_axial()
                    );
                }
            }
        }
    }
}
