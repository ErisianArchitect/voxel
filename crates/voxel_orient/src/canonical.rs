use core::{
    mem::{
        transmute,
    },
};

use crate::{Direction, Flip, Orientation, Rotation};

const XY_ORIENT: Orientation = Orientation::new(Rotation::new(Direction::NegY, 2), Flip::XY);
const XZ_ORIENT: Orientation = Orientation::new(Rotation::new(Direction::PosY, 2), Flip::XZ);
const YZ_ORIENT: Orientation = Orientation::new(Rotation::new(Direction::NegY, 0), Flip::YZ);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CanonicalGroup {
    Group0 = 0,
    Group1 = 1,
    Group2 = 2,
    Group3 = 3,
}

impl CanonicalGroup {
    #[must_use]
    #[inline(always)]
    pub const fn from_u8(group: u8) -> Self {
        unsafe {
            transmute(group & 0b11)
        }
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
            CanonicalGroup::Group0 => Orientation::UNORIENTED,
            CanonicalGroup::Group1 => XY_ORIENT,
            CanonicalGroup::Group2 => XZ_ORIENT,
            CanonicalGroup::Group3 => YZ_ORIENT,
        }
    }

    #[must_use]
    #[inline]
    pub const fn orient_y(self) -> Orientation {
        match self {
            CanonicalGroup::Group0 => Orientation::UNORIENTED,
            CanonicalGroup::Group1 => XY_ORIENT,
            CanonicalGroup::Group2 => YZ_ORIENT,
            CanonicalGroup::Group3 => XZ_ORIENT,
        }
    }

    #[must_use]
    #[inline]
    pub const fn orient_z(self) -> Orientation {
        match self {
            CanonicalGroup::Group0 => Orientation::UNORIENTED,
            CanonicalGroup::Group1 => XZ_ORIENT,
            CanonicalGroup::Group2 => YZ_ORIENT,
            CanonicalGroup::Group3 => XY_ORIENT,
        }
    }
}