// Copyright © 2026 Ada F. <https://github.com/ErisianArchitect>



//! [Canonicalization](https://en.wikipedia.org/wiki/Canonicalization) is the process of converting data that has
//! multiple representations into a "standard" representation.
//! 
//! There are 48 standard orientations for a voxel cube. These 48
//! orientations are composed of 24 rotations, each with a flipped
//! form (either along the X, Y, or Z axis). Each of these 48
//! standard orientations has 3 alternative representations that
//! have equivalent orientations. Two orientations are considered
//! equivalent if they have the same Up, Right, and Forward
//! directions. There are 4 groups of canonical orientations.
//! `Group0`, `Group1`, `Group2` and `Group3`. These 4 groups have
//! 48 orientations for each of the X, Y, and Z axes. There is some
//! overlap between axes. Each group has a specific [Orientation]
//! for canonicalization based on [Flip] state of the orientation
//! being canonicalized, and the axis of canonicalization. These
//! are Canonicalization Orientations. The Group0 is always
//! [Orientation::IDENTITY].
//!
//! # Axial Group Cayley Table
//!
//! |  Flip-->  | [Flip::NONE] | [Flip::X] | [Flip::Y] | [Flip::XY]    | [Flip::Z] | [Flip::XZ]    | [Flip::YZ]    | [Flip::XYZ]   |
//! |:---------:|:------------:|:---------:|:---------:|:-------------:|:---------:|:-------------:|:-------------:|:-------------:|
//! | [Axis::X] | Group0       | Group0    | Group1    | Group1        | Group2    | Group2        | Group3        | Group3        |
//! | [Axis::Y] | Group0       | Group1    | Group0    | Group1        | Group2    | Group3        | Group2        | Group3        |
//! | [Axis::Z] | Group0       | Group1    | Group2    | Group3        | Group0    | Group1        | Group2        | Group3        |

use core::mem::transmute;

use crate::{Direction, Flip, Orientation, Rotation};

const XY_ORIENT: Orientation = Orientation::new(Rotation::new(Direction::NegY, 2), Flip::XY);
const XZ_ORIENT: Orientation = Orientation::new(Rotation::new(Direction::PosY, 2), Flip::XZ);
const YZ_ORIENT: Orientation = Orientation::new(Rotation::new(Direction::NegY, 0), Flip::YZ);

/// The [CanonicalGroup] represents which group of representations an orientation occupies.
/// 
/// For each orientation within the S3
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CanonicalGroup {
    Group0 = 0,
    Group1 = 1,
    Group2 = 2,
    Group3 = 3,
}

impl CanonicalGroup {
    pub const GROUPS: [Self; 4] = [
        Self::Group0,
        Self::Group1,
        Self::Group2,
        Self::Group3,
    ];

    #[must_use]
    #[inline(always)]
    pub const fn from_u8(group: u8) -> Self {
        debug_assert!(group <= 0b11, "group value out of bounds (0..4)");
        unsafe { transmute(group & 0b11) }
    }

    #[must_use]
    #[inline(always)]
    pub const fn to_u8(self) -> u8 {
        self as u8
    }

    #[must_use]
    #[inline]
    pub const fn orient_x(self) -> Orientation {
        match self {
            CanonicalGroup::Group0 => Orientation::IDENTITY,
            CanonicalGroup::Group1 => XY_ORIENT,
            CanonicalGroup::Group2 => XZ_ORIENT,
            CanonicalGroup::Group3 => YZ_ORIENT,
        }
    }

    #[must_use]
    #[inline]
    pub const fn orient_y(self) -> Orientation {
        match self {
            CanonicalGroup::Group0 => Orientation::IDENTITY,
            CanonicalGroup::Group1 => XY_ORIENT,
            CanonicalGroup::Group2 => YZ_ORIENT,
            CanonicalGroup::Group3 => XZ_ORIENT,
        }
    }

    #[must_use]
    #[inline]
    pub const fn orient_z(self) -> Orientation {
        match self {
            CanonicalGroup::Group0 => Orientation::IDENTITY,
            CanonicalGroup::Group1 => XZ_ORIENT,
            CanonicalGroup::Group2 => YZ_ORIENT,
            CanonicalGroup::Group3 => XY_ORIENT,
        }
    }

    #[must_use]
    #[inline(always)]
    pub const fn cycle(self, count: i32) -> Self {
        Self::GROUPS[((self as i64 + count as i64) % 4) as usize]
    }

    #[must_use]
    #[inline(always)]
    pub const fn eq(self, other: Self) -> bool {
        self as u8 == other as u8
    }

    #[must_use]
    #[inline(always)]
    pub const fn ne(self, other: Self) -> bool {
        self as u8 != other as u8
    }

    #[must_use]
    #[inline(always)]
    pub const fn iter() -> CanonicalGroupIter {
        CanonicalGroupIter {
            group_index: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanonicalGroupIter {
    group_index: u8,
}

impl Iterator for CanonicalGroupIter {
    type Item = CanonicalGroup;

    fn next(&mut self) -> Option<Self::Item> {
        if self.group_index >= 4 {
            None
        } else {
            let group = unsafe { *CanonicalGroup::GROUPS.get_unchecked(self.group_index as usize) };
            self.group_index += 1;
            Some(group)
        }
    }
}
