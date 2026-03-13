/*---------------------------------------------------------------------------------------**
||Angles increase in a counter-clockwise direction.                                      ||
||To get the angle in degress, multiply `angle.rem_euclid(4)` or `(angle & 0b11)` by 90. ||
||So an angle of 1 would be 90 degress, 2 would be 180, and 3 would be 270               ||
**---------------------------------------------------------------------------------------*/

use paste::paste;
use vcore::lowlevel::cache_padded::CachePadded;
use crate::{
    direction::Direction, faces::Faces, orientation::Orientation, wrap_angle
};

// verified (2026-1-5)
pub(crate) const CACHED_WRAP_U8_ARRAY: CachePadded<[u8; 256]> = {
    const ROTATIONS_COUNT_U8: u8 = 24;
    let mut arr: CachePadded<[u8; 256]> = CachePadded::new([0u8; 256]);
    let mut index: usize = 0;
    while index < 256 {
        let answer: u8 = index as u8 % ROTATIONS_COUNT_U8;
        arr.value[index] = answer;
        index += 1usize;
    }
    arr
};

// hopefully speeds this operation up by using a (hopefully) cached table.
#[inline(always)]
pub const fn wrap_rotation_u8(rotation: u8) -> u8 {
    CACHED_WRAP_U8_ARRAY.value[rotation as usize]
}

// Verified (2026-1-4)
#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rot {
    #[default]
    PosY0 = 0,
    PosY1 = 1,
    PosY2 = 2,
    PosY3 = 3,
    PosX0 = 4,
    PosX1 = 5,
    PosX2 = 6,
    PosX3 = 7,
    PosZ0 = 8,
    PosZ1 = 9,
    PosZ2 = 10,
    PosZ3 = 11,
    NegY0 = 12,
    NegY1 = 13,
    NegY2 = 14,
    NegY3 = 15,
    NegX0 = 16,
    NegX1 = 17,
    NegX2 = 18,
    NegX3 = 19,
    NegZ0 = 20,
    NegZ1 = 21,
    NegZ2 = 22,
    NegZ3 = 23,
}

impl Rot {
    pub const UNROTATED: Self = Self::PosY0;
    pub const MAX: Self = Self::PosZ3;
    #[inline(always)]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
    
    /// `value` is expected to be in the range `0..24`.
    #[inline(always)]
    pub const unsafe fn from_u8_unchecked(value: u8) -> Self {
        unsafe { ::core::mem::transmute(value) }
    }
    
    #[inline(always)]
    pub const fn from_u8(value: u8) -> Option<Self> {
        if value >= 24 {
            return None;
        }
        Some(unsafe { Self::from_u8_unchecked(value) })
    }
    
    /// `value % 24`
    #[inline(always)]
    pub const fn from_u8_wrapping(value: u8) -> Self {
        let wrapped = wrap_rotation_u8(value);
        unsafe { Self::from_u8_unchecked(wrapped) }
    }
}

#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rotation(pub(crate) Rot);

// const assertions.
const _: () = {
    let bare_size = size_of::<Rotation>();
    let opt_size = size_of::<Option<Rotation>>();
    if bare_size > 1 {
        panic!("Size is greater than 1.");
    }
    if bare_size != opt_size {
        panic!("Niche was not used.");
    }
};

macro_rules! rotate_coord_fns {
    ($(
        $type:ty
    ),*$(,)?) => {
        $(
            paste!{
                // verified (2026-1-5)
                pub const fn [<rotate_coord_ $type>](self, (x, y, z): ($type, $type, $type)) -> ($type, $type, $type) {
                    use Rot::*;
                    match self.0 {
                        PosY0 /* (0, PosY) */ => (x, y, z), // Default rotation, no change.
                        PosY1 /* (1, PosY) */ => (z, y, -x),
                        PosY2 /* (2, PosY) */ => (-x, y, -z),
                        PosY3 /* (3, PosY) */ => (-z, y, x),
                        PosX0 /* (0, PosX) */ => (y, -z, -x),
                        PosX1 /* (1, PosX) */ => (y, x, -z),
                        PosX2 /* (2, PosX) */ => (y, z, x),
                        PosX3 /* (3, PosX) */ => (y, -x, z),
                        PosZ0 /* (0, PosZ) */ => (x, -z, y),
                        PosZ1 /* (1, PosZ) */ => (z, x, y),
                        PosZ2 /* (2, PosZ) */ => (-x, z, y),
                        PosZ3 /* (3, PosZ) */ => (-z, -x, y),
                        NegY0 /* (0, NegY) */ => (x, -y, -z),
                        NegY1 /* (1, NegY) */ => (z, -y, x),
                        NegY2 /* (2, NegY) */ => (-x, -y, z),
                        NegY3 /* (3, NegY) */ => (-z, -y, -x),
                        NegX0 /* (0, NegX) */ => (-y, -z, x),
                        NegX1 /* (1, NegX) */ => (-y, x, z),
                        NegX2 /* (2, NegX) */ => (-y, z, -x),
                        NegX3 /* (3, NegX) */ => (-y, -x, -z),
                        NegZ0 /* (0, NegZ) */ => (-x, -z, -y),
                        NegZ1 /* (1, NegZ) */ => (-z, x, -y),
                        NegZ2 /* (2, NegZ) */ => (x, z, -y),
                        NegZ3 /* (3, NegZ) */ => (z, -x, -y),
                    }
                }
            }
        )*
    };
}

impl Rotation {
    pub const MIN: Self = Self(Rot::UNROTATED);
    // only 24 valid rotations (6 sides * 4 angles)
    pub const MAX: Self = Self(Rot::MAX);
    
    pub(crate) const ANGLE_MASK: u8 = 0b00000011;
    pub(crate) const ANGLE_MASK_I32: i32 = Self::ANGLE_MASK as i32;
    /// ((up << UP_SHIFT) & UP_MASK) | (angle & ANGLE_MASK)
    pub(crate) const UP_SHIFT: u32 = 2;
    
    pub const UNROTATED: Rotation = Rotation::new(Direction::PosY, 0);
    pub const ROTATE_X: Rotation = Rotation::new(Direction::PosZ, 0);
    pub const ROTATE_X_CCW: Rotation = Self::ROTATE_X;
    pub const ROTATE_X_CW: Rotation = Self::ROTATE_X.invert();
    pub const ROTATE_Y: Rotation = Rotation::new(Direction::PosY, 1);
    pub const ROTATE_Y_CCW: Rotation = Self::ROTATE_Y;
    pub const ROTATE_Y_CW: Rotation = Self::ROTATE_Y.invert();
    pub const ROTATE_Z: Rotation = Rotation::new(Direction::NegX, 1);
    pub const ROTATE_Z_CCW: Rotation = Self::ROTATE_Z;
    pub const ROTATE_Z_CW: Rotation = Self::ROTATE_Z.invert();
    pub const X_ROTATIONS: [Rotation; 4] = Self::ROTATE_X.angles();
    pub const Y_ROTATIONS: [Rotation; 4] = Self::ROTATE_Y.angles();
    pub const Z_ROTATIONS: [Rotation; 4] = Self::ROTATE_Z.angles();// verified

    // verified (2025-12-28)
    pub const CORNER_ROTATIONS_MATRIX: [[[[Rotation; 3]; 2]; 2]; 2] = [
        [
            [Rotation::new(Direction::PosX, 2).corner_angles(), Rotation::new(Direction::PosZ, 3).corner_angles()],
            [Rotation::new(Direction::NegZ, 1).corner_angles(), Rotation::new(Direction::NegX, 0).corner_angles()],
        ],
        [
            [Rotation::new(Direction::NegZ, 3).corner_angles(), Rotation::new(Direction::PosX, 0).corner_angles()],
            [Rotation::new(Direction::NegX, 2).corner_angles(), Rotation::new(Direction::PosZ, 1).corner_angles()],
        ],
    ];

    // verified (2025-12-28)
    pub const FACE_ROTATIONS: [[Rotation; 4]; 6] = [
        Self::Y_ROTATIONS, // PosY
        Self::X_ROTATIONS, // PosX
        Self::Z_ROTATIONS, // PosZ
        Self::ROTATE_Y.invert().angles(), // NegY
        Self::ROTATE_X.invert().angles(), // NegX
        Self::ROTATE_Z.invert().angles(), // NegZ
    ];

    // verified (2025-12-28)
    #[inline]
    pub const fn face_rotation(face: Direction, angle: i32) -> Self {
        Self::FACE_ROTATIONS[face.rotation_discriminant() as usize][wrap_angle(angle) as usize]
    }
    
    // verified (2025-12-28)
    pub const fn corner_rotation(x: i32, y: i32, z: i32, angle: i32) -> Rotation {
        let x = if x <= 0 {
            0
        } else {
            1
        } as usize;
        let y = if y <= 0 {
            0
        } else {
            1
        } as usize;
        let z = if z <= 0 {
            0
        } else {
            1
        } as usize;
        let angle = wrap_angle(angle) as usize;
        Self::CORNER_ROTATIONS_MATRIX[y][z][x][angle]
    }
    
    #[inline]
    pub const unsafe fn from_u8_unchecked(value: u8) -> Self {
        Self(unsafe { Rot::from_u8_unchecked(value) })
    }
    
    #[inline]
    pub const fn from_u8(value: u8) -> Option<Self> {
        // Only first 24 values are valid rotations.
        if value > Self::MAX.0 as u8 {
            return None;
        }
        // SAFETY: Guard clause ensures that u8 is valid value.
        Some(unsafe { Self::from_u8_unchecked(value) })
    }
    
    #[inline]
    pub const fn from_u8_wrapping(value: u8) -> Self {
        unsafe { Self::from_u8_unchecked(wrap_rotation_u8(value)) }
    }
    
    #[inline]
    pub const fn new(up: Direction, angle: i32) -> Self {
        let up = up.rotation_discriminant();
        let angle = wrap_angle(angle) as u8;
        unsafe { Self::from_u8_unchecked(angle | (up << Self::UP_SHIFT)) }
    }
    
    #[inline]
    pub const fn from_up(up: Direction) -> Self {
        Self::new(up, 0)
    }
    
    #[inline(always)]
    pub const fn as_u8(self) -> u8 {
        self.0 as u8
    }
    
    /// Creates a new [Rotation] with [Direction::NegX] as the up direction.
    #[inline]
    pub const fn neg_x(angle: i32) -> Self {
        Self::new(Direction::NegX, angle)
    }
    
    /// Creates a new [Rotation] with [Direction::NegY] as the up direction.
    #[inline]
    pub const fn neg_y(angle: i32) -> Self {
        Self::new(Direction::NegY, angle)
    }
    
    /// Creates a new [Rotation] with [Direction::NegZ] as the up direction.
    #[inline]
    pub const fn neg_z(angle: i32) -> Self {
        Self::new(Direction::NegZ, angle)
    }
    
    /// Creates a new [Rotation] with [Direction::PosX] as the up direction.
    #[inline]
    pub const fn pos_x(angle: i32) -> Self {
        Self::new(Direction::PosX, angle)
    }
    
    /// Creates a new [Rotation] with [Direction::PosY] as the up direction.
    #[inline]
    pub const fn pos_y(angle: i32) -> Self {
        Self::new(Direction::PosY, angle)
    }
    
    /// Creates a new [Rotation] with [Direction::PosZ] as the up direction.
    #[inline]
    pub const fn pos_z(angle: i32) -> Self {
        Self::new(Direction::PosZ, angle)
    }
    
    // verified (2025-12-28)
    /// A helper function to create 4 rotations for a rotation group.  
    /// A rotation group is a series of "contiguous" rotations. That is, the rotations are logically sequential.
    /// An example would be rotations around an axis, or around a face, where there are 4 rotations possible.
    /// The first rotation is unrotated, the second rotation is the target rotation, the
    /// third rotation is the target rotation applied twice, and
    /// the fourth rotation is the target rotation applied three times.
    pub const fn angles(self) -> [Self; 4] {
        let angle1 = self;
        let angle2 = angle1.reorient(angle1);
        let angle3 = angle2.reorient(angle1);
        [
            Self::UNROTATED,
            angle1,
            angle2,
            angle3,
        ]
    }
    
    // verified (2025-12-28)
    /// A helper function to create 3 rotations for a corner rotation group.
    /// The first rotation is unrotated, the second rotation is the target rotation,
    /// and the third rotation is the target rotation applied to itself.
    pub const fn corner_angles(self) -> [Self; 3] {
        let angle1 = self;
        let angle2 = angle1.reorient(angle1);
        [
            Self::UNROTATED,
            angle1,
            angle2
        ]
    }

    // verified (2025-12-28)
    #[inline]
    pub const fn with_flip(self, flip: super::Flip) -> Orientation {
        Orientation::new(self, flip)
    }

    // verified (2025-12-28)
    #[inline]
    pub const fn orientation(self) -> Orientation {
        self.with_flip(super::Flip::NONE)
    }
    
    #[inline(always)]
    pub const fn from_up_and_forward(up: Direction, forward: Direction) -> Option<Rotation> {
        // verified (2026-1-9)
        const UP_AND_FORWARD_MATRIX: vcore::lowlevel::align::Align64<[[Option<Rotation>; 8]; 6]> = {
            const fn from_up_and_forward_slow(up: Direction, forward: Direction) -> Option<Rotation> {
                use Direction::*;
                Some(Rotation::new(up, match (up, forward) {
                    (PosY, PosX) => 3,
                    (PosY, PosZ) => 2,
                    (PosY, NegX) => 1,
                    (PosY, NegZ) => 0,
                    (PosX, PosY) => 0,
                    (PosX, PosZ) => 1,
                    (PosX, NegY) => 2,
                    (PosX, NegZ) => 3,
                    (PosZ, PosY) => 0,
                    (PosZ, PosX) => 3,
                    (PosZ, NegY) => 2,
                    (PosZ, NegX) => 1,
                    (NegY, PosX) => 3,
                    (NegY, PosZ) => 0,
                    (NegY, NegX) => 1,
                    (NegY, NegZ) => 2,
                    (NegX, PosY) => 0,
                    (NegX, PosZ) => 3,
                    (NegX, NegY) => 2,
                    (NegX, NegZ) => 1,
                    (NegZ, PosY) => 0,
                    (NegZ, PosX) => 1,
                    (NegZ, NegY) => 2,
                    (NegZ, NegX) => 3,
                    _ => return None,
                }))
            }
            let mut matrix = vcore::lowlevel::align::Align64([[None; 8]; 6]);
            let mut up_i = 0usize;
            while up_i < 6 {
                let up = Direction::INDEX_ORDER[up_i];
                let mut fwd_i = 0usize;
                while fwd_i < 6 {
                    let fwd = Direction::INDEX_ORDER[fwd_i];
                    matrix.0[up_i][fwd_i] = from_up_and_forward_slow(up, fwd);
                    fwd_i += 1;
                }
                up_i += 1;
            }
            matrix
        };
        UP_AND_FORWARD_MATRIX.0[up.rotation_discriminant() as usize][forward.rotation_discriminant() as usize]
    }
    
    #[inline(always)]
    pub const fn faces(self) -> Faces {
        const UP_FORWARD_RIGHT_TABLE: CachePadded<[Faces; 24]> = {
            let mut table = CachePadded::new([Faces::UNORIENTED; 24]);
            let mut rot_i = 0u8;
            while rot_i < 24 {
                let rot = unsafe { Rotation::from_u8_unchecked(rot_i) };
                table.value[rot_i as usize] = Faces {
                    up: rot.up(),
                    right: rot.right(),
                    forward: rot.forward(),
                };
                rot_i += 1;
            }
            table
        };
        UP_FORWARD_RIGHT_TABLE.value[self.0 as usize]
    }

    // Yes, this method works. I checked.
    /// Cycle through rotations (24 in total).
    #[inline]
    #[must_use]
    pub const fn cycle(self, offset: i32) -> Self {
        let index = self.0 as i64;
        // Don't use wrapping_add here, as tempting as it seems. It would be incorrect because 2**32 is not a multiple of 24.
        let new_index = (index + offset as i64).rem_euclid(24) as u8;
        unsafe { Self::from_u8_unchecked(new_index) }
    }

    #[inline]
    pub const fn angle(self) -> i32 {
        (self.0 as u8 & Self::ANGLE_MASK) as i32
    }

    #[inline]
    pub fn set_up(&mut self, up: Direction) {
        const ANGLE_ISOLATE_MASK: u8 = 0b00000011;
        self.0 = unsafe {
            Rot::from_u8_unchecked((self.0 as u8 & ANGLE_ISOLATE_MASK) | (up.rotation_discriminant() << Self::UP_SHIFT))
        };
    }

    // verified (2025-12-28)
    #[inline]
    pub fn set_angle(&mut self, angle: i32) {
        const UP_ISOLATE_MASK: u8 = 0b11111100;
        self.0 = unsafe {
            Rot::from_u8_unchecked((self.0 as u8 & UP_ISOLATE_MASK) | wrap_angle(angle) as u8)
        };
    }
    
    #[inline]
    pub fn iter() -> RotationIterator {
        RotationIterator::START
    }
    
    // verified (2026-1-5)
    pub const fn up(self) -> Direction {
        use Direction::*;
        use Rot::*;
        match self.0 {
            PosY0 => PosY,
            PosY1 => PosY,
            PosY2 => PosY,
            PosY3 => PosY,
            PosX0 => PosX,
            PosX1 => PosX,
            PosX2 => PosX,
            PosX3 => PosX,
            PosZ0 => PosZ,
            PosZ1 => PosZ,
            PosZ2 => PosZ,
            PosZ3 => PosZ,
            NegY0 => NegY,
            NegY1 => NegY,
            NegY2 => NegY,
            NegY3 => NegY,
            NegX0 => NegX,
            NegX1 => NegX,
            NegX2 => NegX,
            NegX3 => NegX,
            NegZ0 => NegZ,
            NegZ1 => NegZ,
            NegZ2 => NegZ,
            NegZ3 => NegZ,
        }
    }

    // verified (2026-1-5)
    pub const fn down(self) -> Direction {
        use Direction::*;
        use Rot::*;
        match self.0 {
            PosY0 => NegY,
            PosY1 => NegY,
            PosY2 => NegY,
            PosY3 => NegY,
            PosX0 => NegX,
            PosX1 => NegX,
            PosX2 => NegX,
            PosX3 => NegX,
            PosZ0 => NegZ,
            PosZ1 => NegZ,
            PosZ2 => NegZ,
            PosZ3 => NegZ,
            NegY0 => PosY,
            NegY1 => PosY,
            NegY2 => PosY,
            NegY3 => PosY,
            NegX0 => PosX,
            NegX1 => PosX,
            NegX2 => PosX,
            NegX3 => PosX,
            NegZ0 => PosZ,
            NegZ1 => PosZ,
            NegZ2 => PosZ,
            NegZ3 => PosZ,
        }
    }

    // verified (2026-1-5)
    pub const fn left(self) -> Direction {
        use Direction::*;
        use Rot::*;
        match self.0 {
            PosY0 => NegX,
            PosY1 => PosZ,
            PosY2 => PosX,
            PosY3 => NegZ,
            PosX0 => PosZ,
            PosX1 => NegY,
            PosX2 => NegZ,
            PosX3 => PosY,
            PosZ0 => NegX,
            PosZ1 => NegY,
            PosZ2 => PosX,
            PosZ3 => PosY,
            NegY0 => NegX,
            NegY1 => NegZ,
            NegY2 => PosX,
            NegY3 => PosZ,
            NegX0 => NegZ,
            NegX1 => NegY,
            NegX2 => PosZ,
            NegX3 => PosY,
            NegZ0 => PosX,
            NegZ1 => NegY,
            NegZ2 => NegX,
            NegZ3 => PosY,
        }
    }

    // verified (2026-1-5)
    pub const fn right(self) -> Direction {
        use Direction::*;
        use Rot::*;
        match self.0 {
            PosY0 => PosX,
            PosY1 => NegZ,
            PosY2 => NegX,
            PosY3 => PosZ,
            PosX0 => NegZ,
            PosX1 => PosY,
            PosX2 => PosZ,
            PosX3 => NegY,
            PosZ0 => PosX,
            PosZ1 => PosY,
            PosZ2 => NegX,
            PosZ3 => NegY,
            NegY0 => PosX,
            NegY1 => PosZ,
            NegY2 => NegX,
            NegY3 => NegZ,
            NegX0 => PosZ,
            NegX1 => PosY,
            NegX2 => NegZ,
            NegX3 => NegY,
            NegZ0 => NegX,
            NegZ1 => PosY,
            NegZ2 => PosX,
            NegZ3 => NegY,
        }
    }

    // verified (2026-1-5)
    pub const fn forward(self) -> Direction {
        use Direction::*;
        use Rot::*;
        match self.0 {
            PosY0 => NegZ,
            PosY1 => NegX,
            PosY2 => PosZ,
            PosY3 => PosX,
            PosX0 => PosY,
            PosX1 => PosZ,
            PosX2 => NegY,
            PosX3 => NegZ,
            PosZ0 => PosY,
            PosZ1 => NegX,
            PosZ2 => NegY,
            PosZ3 => PosX,
            NegY0 => PosZ,
            NegY1 => NegX,
            NegY2 => NegZ,
            NegY3 => PosX,
            NegX0 => PosY,
            NegX1 => NegZ,
            NegX2 => NegY,
            NegX3 => PosZ,
            NegZ0 => PosY,
            NegZ1 => PosX,
            NegZ2 => NegY,
            NegZ3 => NegX,
        }
    }

    // verified (2026-1-5)
    pub const fn backward(self) -> Direction {
        use Direction::*;
        use Rot::*;
        match self.0 {
            PosY0 => PosZ,
            PosY1 => PosX,
            PosY2 => NegZ,
            PosY3 => NegX,
            PosX0 => NegY,
            PosX1 => NegZ,
            PosX2 => PosY,
            PosX3 => PosZ,
            PosZ0 => NegY,
            PosZ1 => PosX,
            PosZ2 => PosY,
            PosZ3 => NegX,
            NegY0 => NegZ,
            NegY1 => PosX,
            NegY2 => PosZ,
            NegY3 => NegX,
            NegX0 => NegY,
            NegX1 => PosZ,
            NegX2 => PosY,
            NegX3 => NegZ,
            NegZ0 => NegY,
            NegZ1 => NegX,
            NegZ2 => PosY,
            NegZ3 => PosX,
        }        
    }
    
    rotate_coord_fns!(i8, i16, i32, i64, i128, isize, f32, f64);

    // verified (2025-12-30)
    /// Rotates `coord`.
    pub fn rotate_coord<T: Copy + std::ops::Neg<Output = T>, C: Into<(T, T, T)> + From<(T, T, T)>>(self, coord: C) -> C {
        let (x, y, z): (T, T, T) = coord.into();
        C::from(match self.0 {
            Rot::PosY0 /* (0, PosY) */ => (x, y, z), // Default rotation, no change.
            Rot::PosY1 /* (1, PosY) */ => (z, y, -x),
            Rot::PosY2 /* (2, PosY) */ => (-x, y, -z),
            Rot::PosY3 /* (3, PosY) */ => (-z, y, x),
            Rot::PosX0 /* (0, PosX) */ => (y, -z, -x),
            Rot::PosX1 /* (1, PosX) */ => (y, x, -z),
            Rot::PosX2 /* (2, PosX) */ => (y, z, x),
            Rot::PosX3 /* (3, PosX) */ => (y, -x, z),
            Rot::PosZ0 /* (0, PosZ) */ => (x, -z, y),
            Rot::PosZ1 /* (1, PosZ) */ => (z, x, y),
            Rot::PosZ2 /* (2, PosZ) */ => (-x, z, y),
            Rot::PosZ3 /* (3, PosZ) */ => (-z, -x, y),
            Rot::NegY0 /* (0, NegY) */ => (x, -y, -z),
            Rot::NegY1 /* (1, NegY) */ => (z, -y, x),
            Rot::NegY2 /* (2, NegY) */ => (-x, -y, z),
            Rot::NegY3 /* (3, NegY) */ => (-z, -y, -x),
            Rot::NegX0 /* (0, NegX) */ => (-y, -z, x),
            Rot::NegX1 /* (1, NegX) */ => (-y, x, z),
            Rot::NegX2 /* (2, NegX) */ => (-y, z, -x),
            Rot::NegX3 /* (3, NegX) */ => (-y, -x, -z),
            Rot::NegZ0 /* (0, NegZ) */ => (-x, -z, -y),
            Rot::NegZ1 /* (1, NegZ) */ => (-z, x, -y),
            Rot::NegZ2 /* (2, NegZ) */ => (x, z, -y),
            Rot::NegZ3 /* (3, NegZ) */ => (z, -x, -y),
        })
    }
    
    // verified (2025-12-28): reface and source_face are symmetrical.
    /// Rotates direction.
    pub const fn reface(self, direction: Direction) -> Direction {
        match direction {
            Direction::NegX => self.left(),
            Direction::NegY => self.down(),
            Direction::NegZ => self.forward(),
            Direction::PosX => self.right(),
            Direction::PosY => self.up(),
            Direction::PosZ => self.backward(),
        }
    }

    // verified (2025-12-28): source_face and reface are symmetrical.
    /// Tells which [Direction] rotated to `destination`.
    pub const fn source_face(self, destination: Direction) -> Direction {
        // This code was bootstrap generated. I wrote a naive solution,
        // then generated this code with the naive solution.
        // Besides maybe if you rearrange the order of matching,
        // this should theoretically be the optimal solution.
        // This CAN be optimized by flattening the key space.
        // By combining the angle, up, and destination into a single index,
        // this could become an O(1) lookup into a table.
        use Direction::*;
        match ((self.angle(), self.up()), destination) {
            ((0, PosY), PosY) => PosY,
            ((0, PosY), PosX) => PosX,
            ((0, PosY), PosZ) => PosZ,
            ((0, PosY), NegY) => NegY,
            ((0, PosY), NegX) => NegX,
            ((0, PosY), NegZ) => NegZ,
            ((0, PosX), PosY) => NegZ,
            ((0, PosX), PosX) => PosY,
            ((0, PosX), PosZ) => NegX,
            ((0, PosX), NegY) => PosZ,
            ((0, PosX), NegX) => NegY,
            ((0, PosX), NegZ) => PosX,
            ((0, PosZ), PosY) => NegZ,
            ((0, PosZ), PosX) => PosX,
            ((0, PosZ), PosZ) => PosY,
            ((0, PosZ), NegY) => PosZ,
            ((0, PosZ), NegX) => NegX,
            ((0, PosZ), NegZ) => NegY,
            ((0, NegY), PosY) => NegY,
            ((0, NegY), PosX) => PosX,
            ((0, NegY), PosZ) => NegZ,
            ((0, NegY), NegY) => PosY,
            ((0, NegY), NegX) => NegX,
            ((0, NegY), NegZ) => PosZ,
            ((0, NegX), PosY) => NegZ,
            ((0, NegX), PosX) => NegY,
            ((0, NegX), PosZ) => PosX,
            ((0, NegX), NegY) => PosZ,
            ((0, NegX), NegX) => PosY,
            ((0, NegX), NegZ) => NegX,
            ((0, NegZ), PosY) => NegZ,
            ((0, NegZ), PosX) => NegX,
            ((0, NegZ), PosZ) => NegY,
            ((0, NegZ), NegY) => PosZ,
            ((0, NegZ), NegX) => PosX,
            ((0, NegZ), NegZ) => PosY,
            ((1, PosY), PosY) => PosY,
            ((1, PosY), PosX) => PosZ,
            ((1, PosY), PosZ) => NegX,
            ((1, PosY), NegY) => NegY,
            ((1, PosY), NegX) => NegZ,
            ((1, PosY), NegZ) => PosX,
            ((1, PosX), PosY) => PosX,
            ((1, PosX), PosX) => PosY,
            ((1, PosX), PosZ) => NegZ,
            ((1, PosX), NegY) => NegX,
            ((1, PosX), NegX) => NegY,
            ((1, PosX), NegZ) => PosZ,
            ((1, PosZ), PosY) => PosX,
            ((1, PosZ), PosX) => PosZ,
            ((1, PosZ), PosZ) => PosY,
            ((1, PosZ), NegY) => NegX,
            ((1, PosZ), NegX) => NegZ,
            ((1, PosZ), NegZ) => NegY,
            ((1, NegY), PosY) => NegY,
            ((1, NegY), PosX) => PosZ,
            ((1, NegY), PosZ) => PosX,
            ((1, NegY), NegY) => PosY,
            ((1, NegY), NegX) => NegZ,
            ((1, NegY), NegZ) => NegX,
            ((1, NegX), PosY) => PosX,
            ((1, NegX), PosX) => NegY,
            ((1, NegX), PosZ) => PosZ,
            ((1, NegX), NegY) => NegX,
            ((1, NegX), NegX) => PosY,
            ((1, NegX), NegZ) => NegZ,
            ((1, NegZ), PosY) => PosX,
            ((1, NegZ), PosX) => NegZ,
            ((1, NegZ), PosZ) => NegY,
            ((1, NegZ), NegY) => NegX,
            ((1, NegZ), NegX) => PosZ,
            ((1, NegZ), NegZ) => PosY,
            ((2, PosY), PosY) => PosY,
            ((2, PosY), PosX) => NegX,
            ((2, PosY), PosZ) => NegZ,
            ((2, PosY), NegY) => NegY,
            ((2, PosY), NegX) => PosX,
            ((2, PosY), NegZ) => PosZ,
            ((2, PosX), PosY) => PosZ,
            ((2, PosX), PosX) => PosY,
            ((2, PosX), PosZ) => PosX,
            ((2, PosX), NegY) => NegZ,
            ((2, PosX), NegX) => NegY,
            ((2, PosX), NegZ) => NegX,
            ((2, PosZ), PosY) => PosZ,
            ((2, PosZ), PosX) => NegX,
            ((2, PosZ), PosZ) => PosY,
            ((2, PosZ), NegY) => NegZ,
            ((2, PosZ), NegX) => PosX,
            ((2, PosZ), NegZ) => NegY,
            ((2, NegY), PosY) => NegY,
            ((2, NegY), PosX) => NegX,
            ((2, NegY), PosZ) => PosZ,
            ((2, NegY), NegY) => PosY,
            ((2, NegY), NegX) => PosX,
            ((2, NegY), NegZ) => NegZ,
            ((2, NegX), PosY) => PosZ,
            ((2, NegX), PosX) => NegY,
            ((2, NegX), PosZ) => NegX,
            ((2, NegX), NegY) => NegZ,
            ((2, NegX), NegX) => PosY,
            ((2, NegX), NegZ) => PosX,
            ((2, NegZ), PosY) => PosZ,
            ((2, NegZ), PosX) => PosX,
            ((2, NegZ), PosZ) => NegY,
            ((2, NegZ), NegY) => NegZ,
            ((2, NegZ), NegX) => NegX,
            ((2, NegZ), NegZ) => PosY,
            ((3, PosY), PosY) => PosY,
            ((3, PosY), PosX) => NegZ,
            ((3, PosY), PosZ) => PosX,
            ((3, PosY), NegY) => NegY,
            ((3, PosY), NegX) => PosZ,
            ((3, PosY), NegZ) => NegX,
            ((3, PosX), PosY) => NegX,
            ((3, PosX), PosX) => PosY,
            ((3, PosX), PosZ) => PosZ,
            ((3, PosX), NegY) => PosX,
            ((3, PosX), NegX) => NegY,
            ((3, PosX), NegZ) => NegZ,
            ((3, PosZ), PosY) => NegX,
            ((3, PosZ), PosX) => NegZ,
            ((3, PosZ), PosZ) => PosY,
            ((3, PosZ), NegY) => PosX,
            ((3, PosZ), NegX) => PosZ,
            ((3, PosZ), NegZ) => NegY,
            ((3, NegY), PosY) => NegY,
            ((3, NegY), PosX) => NegZ,
            ((3, NegY), PosZ) => NegX,
            ((3, NegY), NegY) => PosY,
            ((3, NegY), NegX) => PosZ,
            ((3, NegY), NegZ) => PosX,
            ((3, NegX), PosY) => NegX,
            ((3, NegX), PosX) => NegY,
            ((3, NegX), PosZ) => NegZ,
            ((3, NegX), NegY) => PosX,
            ((3, NegX), NegX) => PosY,
            ((3, NegX), NegZ) => PosZ,
            ((3, NegZ), PosY) => NegX,
            ((3, NegZ), PosX) => PosZ,
            ((3, NegZ), PosZ) => NegY,
            ((3, NegZ), NegY) => PosX,
            ((3, NegZ), NegX) => NegZ,
            ((3, NegZ), NegZ) => PosY,
            _ => unreachable!(),
        }
    }

    // verified (2025-12-28)
    // double verified (2025-12-29)
    /// Gets the angle of the face oriented to `world_face`.
    pub fn face_angle(self, world_face: Direction) -> u8 {
        use Direction::*;
        match (self.angle(), self.up(), world_face) {
            (0, NegX, NegX) => 0,
            (0, NegX, NegY) => 1,
            (0, NegX, NegZ) => 3,
            (0, NegX, PosX) => 2,
            (0, NegX, PosY) => 1,
            (0, NegX, PosZ) => 1,
            (0, NegY, NegX) => 2,
            (0, NegY, NegY) => 0,
            (0, NegY, NegZ) => 2,
            (0, NegY, PosX) => 2,
            (0, NegY, PosY) => 0,
            (0, NegY, PosZ) => 2,
            (0, NegZ, NegX) => 1,
            (0, NegZ, NegY) => 2,
            (0, NegZ, NegZ) => 0,
            (0, NegZ, PosX) => 3,
            (0, NegZ, PosY) => 0,
            (0, NegZ, PosZ) => 2,
            (0, PosX, NegX) => 2,
            (0, PosX, NegY) => 3,
            (0, PosX, NegZ) => 1,
            (0, PosX, PosX) => 0,
            (0, PosX, PosY) => 3,
            (0, PosX, PosZ) => 3,
            (0, PosY, NegX) => 0,
            (0, PosY, NegY) => 0,
            (0, PosY, NegZ) => 0,
            (0, PosY, PosX) => 0,
            (0, PosY, PosY) => 0,
            (0, PosY, PosZ) => 0,
            (0, PosZ, NegX) => 3,
            (0, PosZ, NegY) => 0,
            (0, PosZ, NegZ) => 2,
            (0, PosZ, PosX) => 1,
            (0, PosZ, PosY) => 2,
            (0, PosZ, PosZ) => 0,
            (1, NegX, NegX) => 1,
            (1, NegX, NegY) => 1,
            (1, NegX, NegZ) => 3,
            (1, NegX, PosX) => 1,
            (1, NegX, PosY) => 1,
            (1, NegX, PosZ) => 1,
            (1, NegY, NegX) => 2,
            (1, NegY, NegY) => 1,
            (1, NegY, NegZ) => 2,
            (1, NegY, PosX) => 2,
            (1, NegY, PosY) => 3,
            (1, NegY, PosZ) => 2,
            (1, NegZ, NegX) => 1,
            (1, NegZ, NegY) => 2,
            (1, NegZ, NegZ) => 1,
            (1, NegZ, PosX) => 3,
            (1, NegZ, PosY) => 0,
            (1, NegZ, PosZ) => 1,
            (1, PosX, NegX) => 1,
            (1, PosX, NegY) => 3,
            (1, PosX, NegZ) => 1,
            (1, PosX, PosX) => 1,
            (1, PosX, PosY) => 3,
            (1, PosX, PosZ) => 3,
            (1, PosY, NegX) => 0,
            (1, PosY, NegY) => 3,
            (1, PosY, NegZ) => 0,
            (1, PosY, PosX) => 0,
            (1, PosY, PosY) => 1,
            (1, PosY, PosZ) => 0,
            (1, PosZ, NegX) => 3,
            (1, PosZ, NegY) => 0,
            (1, PosZ, NegZ) => 1,
            (1, PosZ, PosX) => 1,
            (1, PosZ, PosY) => 2,
            (1, PosZ, PosZ) => 1,
            (2, NegX, NegX) => 2,
            (2, NegX, NegY) => 1,
            (2, NegX, NegZ) => 3,
            (2, NegX, PosX) => 0,
            (2, NegX, PosY) => 1,
            (2, NegX, PosZ) => 1,
            (2, NegY, NegX) => 2,
            (2, NegY, NegY) => 2,
            (2, NegY, NegZ) => 2,
            (2, NegY, PosX) => 2,
            (2, NegY, PosY) => 2,
            (2, NegY, PosZ) => 2,
            (2, NegZ, NegX) => 1,
            (2, NegZ, NegY) => 2,
            (2, NegZ, NegZ) => 2,
            (2, NegZ, PosX) => 3,
            (2, NegZ, PosY) => 0,
            (2, NegZ, PosZ) => 0,
            (2, PosX, NegX) => 0,
            (2, PosX, NegY) => 3,
            (2, PosX, NegZ) => 1,
            (2, PosX, PosX) => 2,
            (2, PosX, PosY) => 3,
            (2, PosX, PosZ) => 3,
            (2, PosY, NegX) => 0,
            (2, PosY, NegY) => 2,
            (2, PosY, NegZ) => 0,
            (2, PosY, PosX) => 0,
            (2, PosY, PosY) => 2,
            (2, PosY, PosZ) => 0,
            (2, PosZ, NegX) => 3,
            (2, PosZ, NegY) => 0,
            (2, PosZ, NegZ) => 0,
            (2, PosZ, PosX) => 1,
            (2, PosZ, PosY) => 2,
            (2, PosZ, PosZ) => 2,
            (3, NegX, NegX) => 3,
            (3, NegX, NegY) => 1,
            (3, NegX, NegZ) => 3,
            (3, NegX, PosX) => 3,
            (3, NegX, PosY) => 1,
            (3, NegX, PosZ) => 1,
            (3, NegY, NegX) => 2,
            (3, NegY, NegY) => 3,
            (3, NegY, NegZ) => 2,
            (3, NegY, PosX) => 2,
            (3, NegY, PosY) => 1,
            (3, NegY, PosZ) => 2,
            (3, NegZ, NegX) => 1,
            (3, NegZ, NegY) => 2,
            (3, NegZ, NegZ) => 3,
            (3, NegZ, PosX) => 3,
            (3, NegZ, PosY) => 0,
            (3, NegZ, PosZ) => 3,
            (3, PosX, NegX) => 3,
            (3, PosX, NegY) => 3,
            (3, PosX, NegZ) => 1,
            (3, PosX, PosX) => 3,
            (3, PosX, PosY) => 3,
            (3, PosX, PosZ) => 3,
            (3, PosY, NegX) => 0,
            (3, PosY, NegY) => 1,
            (3, PosY, NegZ) => 0,
            (3, PosY, PosX) => 0,
            (3, PosY, PosY) => 3,
            (3, PosY, PosZ) => 0,
            (3, PosZ, NegX) => 3,
            (3, PosZ, NegY) => 0,
            (3, PosZ, NegZ) => 3,
            (3, PosZ, PosX) => 1,
            (3, PosZ, PosY) => 2,
            (3, PosZ, PosZ) => 3,
            _ => unreachable!(),
        }
    }

    // verified (2025-12-28)
    /// Rotate a [Rotation] by another [Rotation].
    pub const fn reorient(self, rotation: Self) -> Self {
        // What??? I know I wrote this code, but this is kinda nuts.
        let up = self.up();
        let fwd = self.forward();
        let rot_up = rotation.reface(up);
        let rot_fwd = rotation.reface(fwd);
        // Pattern matching is used here because it's a const fn and unwrap()
        // won't work.
        let Some(rot) = Self::from_up_and_forward(rot_up, rot_fwd) else {
            unreachable!()
        };
        rot
    }

    // verified (2025-12-28)
    /// Rotate a [Rotation] by the inverse of another [Rotation].
    pub const fn deorient(self, rotation: Self) -> Self {
        let up = self.up();
        let fwd = self.forward();
        let rot_up = rotation.source_face(up);
        let rot_fwd = rotation.source_face(fwd);
        // Pattern matching is used here because it's a const fn and unwrap()
        // won't work.
        let Some(rot) = Self::from_up_and_forward(rot_up, rot_fwd) else {
            unreachable!()
        };
        rot
    }
    
    // verified (2025-12-28)
    /// Creates a [Rotation] that when rotated by the original will create the base [Rotation].
    #[inline]
    pub const fn invert(self) -> Self {
        Self::UNROTATED.deorient(self)
    }

    #[inline]
    pub const fn rotate_x(self, angle: i32) -> Self {
        self.reorient(Self::X_ROTATIONS[wrap_angle(angle) as usize])
    }

    #[inline]
    pub const fn rotate_y(self, angle: i32) -> Self {
        self.reorient(Self::Y_ROTATIONS[wrap_angle(angle) as usize])
    }

    #[inline]
    pub const fn rotate_z(self, angle: i32) -> Self {
        self.reorient(Self::Z_ROTATIONS[wrap_angle(angle) as usize])
    }

    /// Rotate `face` counter-clockwise by `angle`. Use a negative `angle` to rotate clockwise.
    #[inline]
    pub const fn rotate_face(self, face: Direction, angle: i32) -> Self {
        let rot = Self::face_rotation(face, angle);
        self.reorient(rot)
    }

    /// Rotate corner clockwise by `angle`.
    #[inline]
    pub const fn rotate_corner(self, x: i32, y: i32, z: i32, angle: i32) -> Self {
        let rot = Self::corner_rotation(x, y, z, angle);
        self.reorient(rot)
    }

    // #[inline]
    // pub fn to_matrix(self) -> glam::Mat4 {
    //     let up = self.reface(Direction::PosY).to_vec3();
    //     let forward = self.reface(Direction::PosZ).to_vec3();
    //     let right = self.reface(Direction::NegX).to_vec3();
    //     glam::Mat4::from_cols(
    //         right.extend(0.0),
    //         up.extend(0.0),
    //         forward.extend(0.0),
    //         glam::Vec3::ZERO.extend(1.0),
    //     )
    // }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RotationIterator {
    rotation: u8,
}

impl RotationIterator {
    pub const START: Self = Self { rotation: 0 };
    
    #[inline]
    pub const fn start_at(rotation: Rotation) -> Self {
        Self { rotation: rotation.0 as u8 }
    }
    
    #[inline]
    pub const fn new() -> Self {
        Self::START
    }
    
    pub const fn current(self) -> Option<Rotation> {
        if self.rotation == 24 {
            return None;
        }
        Some(unsafe { Rotation::from_u8_unchecked(self.rotation) })
    }
}

impl Iterator for RotationIterator {
    type Item = Rotation;
    
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        const MAX: u8 = 24;
        let remain = (MAX - self.rotation) as usize;
        (remain, Some(remain))
    }
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.rotation >= 24 {
            return None;
        }
        let result = Some(unsafe { Rotation::from_u8_unchecked(self.rotation) });
        self.rotation += 1;
        result
    }
}

impl From<Direction> for Rotation {
    #[inline]
    fn from(value: Direction) -> Self {
        Self::new(value, 0)
    }
}

impl std::fmt::Display for Rotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Rotation(up={},forward={},angle={})", self.up(), self.forward(), self.angle())
    }
}