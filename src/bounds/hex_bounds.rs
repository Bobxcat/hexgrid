use std::{array, mem::MaybeUninit, ptr, slice};

use crate::{ArrayBuilder, bounds::Bounds, coord::AxialCoord};

#[repr(C)]
struct StartOfRowIdxTables<const R: usize> {
    lo: [usize; R],
    mid: usize,
    hi: [usize; R],
}

impl<const R: usize> StartOfRowIdxTables<R> {
    const fn as_slice(&self) -> &[usize] {
        unsafe { slice::from_raw_parts(self as *const Self as *const usize, R * 2 + 1) }
    }
}

/// The size of a hexagon of radius `R` is `3R^2 + 3R + 1`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ConstHexBoundsStore<const R: usize, T> {
    /// 3R^2
    squared_part: [[[T; 3]; R]; R],
    /// 3R
    linear_part: [[T; 3]; R],
    /// 1
    const_part: T,
}

impl<const R: usize, T> ConstHexBoundsStore<R, T> {
    pub fn from_fn(mut f: impl FnMut(AxialCoord) -> T) -> Self {
        // ConstHexBoundsStore::<R,()>::from_copy(()).map(||)
        // Self {
        //     squared_part: [[[]]],
        //     linear_part: todo!(),
        //     const_part: todo!(),
        // }
        todo!()
    }

    pub fn from_init(mut init: impl FnMut() -> T) -> Self {
        Self {
            squared_part: array::from_fn(|_| array::from_fn(|_| array::from_fn(|_| init()))),
            linear_part: array::from_fn(|_| array::from_fn(|_| init())),
            const_part: init(),
        }
    }

    pub const fn from_copy(val: T) -> Self
    where
        T: Copy,
    {
        Self {
            squared_part: [[[val; _]; _]; _],
            linear_part: [[val; _]; _],
            const_part: val,
        }
    }

    pub const fn total_size() -> usize {
        const { 3 * R * (R + 1) + 1 }
    }

    const fn row_size(row: usize) -> usize {
        (const { R * 2 + 1 }) - usize::abs_diff(R, row)
    }

    #[inline]
    const fn start_of_row_idx_tables() -> StartOfRowIdxTables<R> {
        const {
            let table_lo: [usize; R] = {
                let mut builder = ArrayBuilder::new();
                let mut i = 0;
                let mut sum = 0;
                while i < R {
                    builder.push(sum);
                    sum += Self::row_size(i as usize);
                    i += 1;
                }
                builder.build()
            };
            let mid: usize = table_lo[R - 1] + Self::row_size(R - 1);
            let table_hi: [usize; R] = {
                let mut builder = ArrayBuilder::new();
                let mut i = R + 1;
                let mut sum = mid + Self::row_size(R);
                while i <= 2 * R {
                    builder.push(sum);
                    sum += Self::row_size(i as usize);
                    i += 1;
                }
                builder.build()
            };
            StartOfRowIdxTables {
                lo: table_lo,
                mid,
                hi: table_hi,
            }
        }
    }

    const fn start_of_row_idx(row: usize) -> usize {
        Self::start_of_row_idx_tables().as_slice()[row]
    }

    /// Returns `None` if `idx` isn't within this store (which is centered at `(R, R)`)
    #[inline]
    const fn axial_to_idx(coord: AxialCoord) -> Option<usize> {
        if coord.q.is_negative() || coord.r.is_negative() {
            return None;
        }
        let (q, r) = (coord.q as u32, coord.r as u32);
        // Implementing this function incorrectly could lead to UB
        let row = r as usize;
        let idx_in_row = (q - u32::saturating_sub(R as u32, r)) as usize;
        Some(Self::idx_parts_to_idx(row, idx_in_row))
    }

    #[inline]
    const fn idx_parts_to_axial(row: usize, idx_in_row: usize) -> AxialCoord {
        let r = row as i32;
        let q = (idx_in_row + usize::saturating_sub(R as usize, row)) as i32;
        AxialCoord::new(q, r)
    }

    #[inline]
    const fn idx_parts_to_idx(row: usize, idx_in_row: usize) -> usize {
        Self::start_of_row_idx(row) + idx_in_row
    }

    /// The data in this store, in no particular order
    #[inline]
    pub const fn as_slice(&self) -> &[T] {
        let first_elem = ptr::from_ref(self).cast::<T>();
        unsafe { slice::from_raw_parts(first_elem, Self::total_size()) }
    }

    /// The data in this store, in no particular order
    #[inline]
    pub const fn as_slice_mut(&mut self) -> &mut [T] {
        let first_elem = ptr::from_mut(self).cast::<T>();
        unsafe { slice::from_raw_parts_mut(first_elem, Self::total_size()) }
    }

    #[inline]
    pub const fn get(&self, idx: AxialCoord) -> Option<&T> {
        let Some(idx_flat) = Self::axial_to_idx(idx) else {
            return None;
        };
        Some(&self.as_slice()[idx_flat])
    }

    #[inline]
    pub const fn get_mut(&mut self, idx: AxialCoord) -> Option<&mut T> {
        let Some(idx_flat) = Self::axial_to_idx(idx) else {
            return None;
        };
        Some(&mut self.as_slice_mut()[idx_flat])
    }

    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> ConstHexBoundsStore<R, U> {
        ConstHexBoundsStore {
            squared_part: self.squared_part.map(|x| x.map(|y| y.map(|z| f(z)))),
            linear_part: self.linear_part.map(|x| x.map(|y| f(y))),
            const_part: f(self.const_part),
        }
    }

    pub fn map_with_idx<U>(
        self,
        mut f: impl FnMut(AxialCoord, T) -> U,
    ) -> ConstHexBoundsStore<R, U> {
        let mut map_from = self.map(|val| MaybeUninit::new(val));
        let mut map_to = ConstHexBoundsStore::from_init(MaybeUninit::<U>::uninit);
        // let x = ();
        for row in 0..=R * 2 {
            // let start_of_row = Self::start_of_row_idx(row);
            // let q_offset = usize::saturating_sub(R as usize, row);
            for idx_in_row in 0..Self::row_size(row) {
                // let r = row as i32;
                // let q = (idx_in_row + q_offset) as i32;
                // let coord = AxialCoord::new(q, r);
                //
                let coord = Self::idx_parts_to_axial(row, idx_in_row);
                let mapped = f(
                    coord,
                    // SAFETY: each `map_from` value is only read once
                    unsafe { map_from.get_mut(coord).unwrap().assume_init_read() },
                );
                map_to.get_mut(coord).unwrap().write(mapped);
            }
        }
        map_to.map(|uninit|
            // SAFETY: all values are initialized
            unsafe {
                uninit.assume_init()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{AxialCoord, ConstHexBoundsStore};

    #[test]
    fn foo() {
        let mut x = ConstHexBoundsStore::<1, _>::from_copy(10u32);
        println!("{x:#?}");
        *x.get_mut(AxialCoord::new(1, 1)).unwrap() += 1;
        println!("{x:#?}");
        *x.get_mut(AxialCoord::new(0, 1)).unwrap() += 1;
        println!("{x:#?}");
        *x.get_mut(AxialCoord::new(2, 1)).unwrap() += 1;
        println!("{x:#?}");
    }

    #[test]
    fn store_map_with_idx() {
        let mut x = ConstHexBoundsStore::<3, _>::from_copy(10u32);
        let mut x = x.map_with_idx(|coord, v| {
            //
            v + 1 + coord.q.unsigned_abs()
        });
    }
}

pub struct ConstHexagonBounds<const CENTER_Q: i32, const CENTER_R: i32, const RADIUS: usize, T> {
    store: ConstHexBoundsStore<RADIUS, T>,
}

impl<const CENTER_Q: i32, const CENTER_R: i32, const RADIUS: usize, T>
    ConstHexagonBounds<CENTER_Q, CENTER_R, RADIUS, T>
{
    pub const fn size() -> usize {
        3 * RADIUS * (RADIUS + 1) + 1
    }

    pub const fn center() -> AxialCoord {
        AxialCoord {
            q: CENTER_Q,
            r: CENTER_R,
        }
    }

    // pub fn new(x: T) -> Self {
    //     Self {
    //         storage: [x; RADIUS],
    //     }
    // }
}

/// The bounded range is a hexagon,
/// centered on a tile and defined by a maximum
/// distance to the center
pub struct HexagonBounds<T> {
    pub center: AxialCoord,
    /// Radius, not including the center
    pub radius: u32,
    storage: Vec<Vec<T>>,
}

impl<T> HexagonBounds<T> {
    fn row_count(&self) -> u32 {
        self.radius * 2 + 1
    }

    fn row_size(&self, row: u32) -> u32 {
        self.radius * 2 + 1 - (self.radius as i32 - row as i32).unsigned_abs()
    }
}

impl<T> Bounds<T> for HexagonBounds<T> {
    //! https://www.redblobgames.com/grids/hexagons/#map-storage

    /// (row, col)
    type Idx = (u32, u32);

    // fn data_length(&self) -> usize {
    //     // Maybe change it so that `HexGrid` stores a 2d vec and we can specify each inner `Vec`'s length?

    //     (0..self.row_count())
    //         .map(|row| self.row_size(row) as usize)
    //         .sum()
    // }

    // fn coord_to_idx(&self, coord: AxialCoord) -> usize {
    //     let row_size = 10;
    //     let idx_in_row = coord.r.rem_euclid(row_size);
    //     todo!()
    // }

    fn coord_to_idx(&self, coord: AxialCoord) -> Self::Idx {
        // `coord` is adjusted so that the center is at `Axial(radius, radius)`
        let axial = coord - self.center + AxialCoord::new(self.radius as i32, self.radius as i32);
        (
            axial.r as u32,
            (axial.q as u32).wrapping_sub(u32::saturating_sub(self.radius, axial.r as u32)),
        )
    }

    fn get_by_idx(&self, idx: Self::Idx) -> Option<&T> {
        self.storage.get(idx.0 as usize)?.get(idx.1 as usize)
    }
}
