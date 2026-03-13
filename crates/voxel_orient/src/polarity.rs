use crate::{Axis, Direction};


/// [Pol] represents the polarity (positive [+] or negative [-]) of a value.
#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Pol {
    /// Negative (-) polarity.
    #[default]
    Neg = 0,
    /// Positive (+) polarity.
    Pos = 1,
}

impl Pol {
    
    #[inline]
    pub const fn x(self) -> Direction {
        match self {
            Pol::Neg => Direction::NegX,
            Pol::Pos => Direction::PosX,
        }
    }
    
    #[inline]
    pub const fn y(self) -> Direction {
        match self {
            Pol::Neg => Direction::NegY,
            Pol::Pos => Direction::PosY,
        }
    }
    
    #[inline]
    pub const fn z(self) -> Direction {
        match self {
            Pol::Neg => Direction::NegZ,
            Pol::Pos => Direction::PosZ,
        }
    }
    
    #[inline]
    pub const fn with_axis(self, axis: Axis) -> Direction {
        Direction::from_polar_axis(self, axis)
    }
    
    #[inline(always)]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
    
    #[inline(always)]
    pub const fn is_neg(self) -> bool {
        matches!(self, Self::Neg)
    }
    
    #[inline(always)]
    pub const fn is_pos(self) -> bool {
        matches!(self, Self::Pos)
    }
    
    #[inline(always)]
    pub const fn invert(self) -> Self {
        match self {
            Self::Neg => Self::Pos,
            Self::Pos => Self::Neg,
        }
    }
}