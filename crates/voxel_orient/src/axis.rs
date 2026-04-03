// Last Reviewed: (2025-12-28)

use crate::{Flip, direction::Direction, polarity::Pol};

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

    #[must_use]
    #[inline(always)]
    pub const fn as_flip_if(self, flipped: bool) -> Flip {
        const FLIPS: [Flip; 6] = [
            Flip::NONE, Flip::NONE, Flip::NONE,
            Flip::X, Flip::Y, Flip::Z,
        ];
        FLIPS[(self as usize) << (flipped as u32)]
    }

    #[must_use]
    #[inline(always)]
    pub const fn as_flip(self) -> Flip {
        unsafe {
            core::mem::transmute(1u8 << self as u32)
        }
    }
    
    #[inline]
    pub const fn with_polarity(self, pol: Pol) -> Direction {
        Direction::from_polar_axis(pol, self)
    }

    #[must_use]
    #[inline(always)]
    pub const fn display(self) -> AxisDisplay {
        AxisDisplay(self)
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct AxisDisplay(pub Axis);

impl std::fmt::Display for AxisDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "{}",
            match self.0 {
                Axis::X => "X",
                Axis::Y => "Y",
                Axis::Z => "Z",
            }
        )
    }
}