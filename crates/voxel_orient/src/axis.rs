// Copyright © 2026 Ada F. <https://github.com/ErisianArchitect>



// Last Reviewed: (2025-12-28)

use crate::{Flip, direction::Direction, polarity::Pol};

/// Represents a coordinate axis (either X, Y, or Z).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Axis {
    X = 0,
    Y = 1,
    Z = 2,
}
const _: () = isit::const_assert(Axis::X as u8 == 0);
const _: () = isit::const_assert(Axis::Y as u8 == 1);
const _: () = isit::const_assert(Axis::Z as u8 == 2);

impl Axis {
    /// The direction on this axis in the positive direction.
    #[inline]
    pub const fn pos(self) -> Direction {
        match self {
            Axis::X => Direction::PosX,
            Axis::Y => Direction::PosY,
            Axis::Z => Direction::PosZ,
        }
    }

    /// The direction on this axis in the negative direction.
    #[inline]
    pub const fn neg(self) -> Direction {
        match self {
            Axis::X => Direction::NegX,
            Axis::Y => Direction::NegY,
            Axis::Z => Direction::NegZ,
        }
    }

    /// If `flipped` is `true`, this will return a [Flip] with the corresponding axis inverted.
    #[must_use]
    #[inline(always)]
    pub const fn as_flip_if(self, flipped: bool) -> Flip {
        if flipped { self.as_flip() } else { Flip::NONE }
    }

    /// Returns a [Flip] with the corresponding axis inverted.
    #[must_use]
    #[inline(always)]
    pub const fn as_flip(self) -> Flip {
        const _SAFETY: () = {
            isit::assert_same_size_align::<Flip, u8>();
            isit::const_assert(1 << (Axis::X as u32) == Flip::X.as_u8());
            isit::const_assert(1 << (Axis::Y as u32) == Flip::Y.as_u8());
            isit::const_assert(1 << (Axis::Z as u32) == Flip::Z.as_u8());
        };
        // SAFETY: Flip is represented using the first 3 bits of the byte.
        //         Axis is represented as `X = 0, Y = 1, Z = 2`.
        //         The index of the bits in `Flip` correspond to the discriminant
        //         value of `Axis.` If the program compiles (the above checks
        //         passed), then this code is sound.
        unsafe { core::mem::transmute(1u8 << self as u32) }
    }

    /// Create a [Direction] with the given polarity and the axis of `self`.
    #[inline]
    pub const fn with_polarity(self, pol: Pol) -> Direction {
        Direction::from_polar_axis(pol, self)
    }

    /// Return a representation that can be displayed with a formatter.
    #[must_use]
    #[inline(always)]
    pub const fn display(self) -> AxisDisplay {
        AxisDisplay(self)
    }
}

/// Used for displaying [Axis] as text.
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct AxisDisplay(pub Axis);

impl std::fmt::Display for AxisDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self.0 {
                Axis::X => "X",
                Axis::Y => "Y",
                Axis::Z => "Z",
            }
        )
    }
}
