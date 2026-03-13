use crate::Direction;


#[repr(C, align(4))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Faces {
    pub(crate) up: Direction,
    pub(crate) right: Direction,
    pub(crate) forward: Direction,
}

impl Faces {
    pub(crate) const UNORIENTED: Self = Self {
        up: Direction::PosY,
        right: Direction::PosX,
        forward: Direction::NegZ,
    };
    
    pub const fn new(up: Direction, right: Direction, forward: Direction) -> Option<Self> {
        if up.is_orthogonal_to(right)
        && right.is_orthogonal_to(forward)
        && forward.is_orthogonal_to(up)
        {
            Some(Self { up, right, forward })
        } else {
            None
        }
    }
    
    #[inline(always)]
    pub const fn up(self) -> Direction {
        self.up
    }
    
    #[inline(always)]
    pub const fn right(self) -> Direction {
        self.right
    }
    
    #[inline(always)]
    pub const fn forward(self) -> Direction {
        self.forward
    }
    
    #[inline(always)]
    pub const fn down(self) -> Direction {
        self.up.invert()
    }
    
    #[inline(always)]
    pub const fn left(self) -> Direction {
        self.right.invert()
    }
    
    #[inline(always)]
    pub const fn backward(self) -> Direction {
        self.forward.invert()
    }
}