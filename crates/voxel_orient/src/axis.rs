// Last Reviewed: (2025-12-28)

use crate::{direction::Direction, polarity::Pol};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Axis {
    X = 0,
    Y = 1,
    Z = 2
}

impl Axis {
    #[inline]
    pub const fn pos(self) -> Direction {
        match self {
            Axis::X => Direction::PosX,
            Axis::Y => Direction::PosY,
            Axis::Z => Direction::PosZ,
        }
    }

    #[inline]
    pub const fn neg(self) -> Direction {
        match self {
            Axis::X => Direction::NegX,
            Axis::Y => Direction::NegY,
            Axis::Z => Direction::NegZ,
        }
    }
    
    #[inline]
    pub const fn with_polarity(self, pol: Pol) -> Direction {
        Direction::from_polar_axis(pol, self)
    }
}