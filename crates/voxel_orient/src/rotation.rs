// Copyright © 2026 Ada F. <https://github.com/ErisianArchitect>



/*---------------------------------------------------------------------------------------**
||Angles increase in a counter-clockwise direction.                                      ||
||To get the angle in degress, multiply `angle.rem_euclid(4)` or `(angle & 0b11)` by 90. ||
||So an angle of 1 would be 90 degress, 2 would be 180, and 3 would be 270               ||
**---------------------------------------------------------------------------------------*/

use paste::paste;
use lolevel::{
    cache_padded::CachePadded,
};
use crate::{
    direction::Direction, faces::Faces, orientation::Orientation, wrap_angle
};

#[cfg(feature = "glam")]
use glam::*;

// verified (2026-1-5)
/// Lookup table for pre-wrapped rotation indices for each byte value.
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
#[must_use]
#[inline(always)]
pub const fn wrap_rotation_u8(rotation: u8) -> u8 {
    CACHED_WRAP_U8_ARRAY.value[rotation as usize]
}

#[repr(C, align(8))]
struct DirectionTable {
    table: [Direction; 6],
    pad: u16,
}

impl DirectionTable {
    #[must_use]
    #[inline(always)]
    pub const fn new(table: [Direction; 6]) -> Self {
        Self {
            table,
            pad: 0,
        }
    }

    #[must_use]
    #[inline(always)]
    pub const fn get(&self, dir: Direction) -> Direction {
        self.table[dir as usize]
    }
}

// Verified (2026-1-4)
// This Rot enum is used for niche optimization and other performance optimizations.
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
    /// The identity rotation (unrotated).
    pub const IDENTITY: Self = Self::PosY0;
    /// The rotation value with the maximum discriminant value.
    pub const MAX: Self = Self::NegZ3;

    /// Convert to the raw discriminant value as a [u8].
    #[must_use]
    #[inline(always)]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    /// Create [Rot] from the raw discriminant value.
    /// `value` is expected to be in the range `0..24`.
    #[must_use]
    #[inline(always)]
    pub const unsafe fn from_u8_unchecked(value: u8) -> Self {
        unsafe { ::core::mem::transmute(value) }
    }

    /// Attempt to create a [Rot] from a raw discriminat value.
    #[must_use]
    #[inline(always)]
    pub const fn from_u8(value: u8) -> Option<Self> {
        if value >= 24 {
            return None;
        }
        Some(unsafe { Self::from_u8_unchecked(value) })
    }

    /// Create a [Rot] from a discriminant that has been wrapped by the number of variants.
    /// `value % 24`
    #[must_use]
    #[inline(always)]
    pub const fn from_u8_wrapping(value: u8) -> Self {
        let wrapped = wrap_rotation_u8(value);
        unsafe { Self::from_u8_unchecked(wrapped) }
    }
}

/// Represents the 
#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rotation(pub(crate) Rot);
const _: () = isit::assert_u8_niche::<Rotation>();

macro_rules! rotate_coord_fns {
    ($(
        $type:ty
    ),*$(,)?) => {
        $(
            paste!{
                // verified (2026-1-5)
                #[must_use]
                pub const fn [<rotate_coord_ $type>](self, (x, y, z): ($type, $type, $type)) -> ($type, $type, $type) {
                    use Rot::*;
                    #[repr(u8)]
                    #[derive(Clone, Copy)]
                    enum XYZ {
                        X = 0,
                        Y = 1,
                        Z = 2,
                    }
                    use XYZ::{X, Y, Z};
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

/// ```rust, ignore
/// pad_table![]
/// ```
macro_rules! pad_table {
    ($($elem:ident),*$(,)?) => {
        [$(Direction::$elem,)* Direction::PosY, Direction::PosY, Direction::PosY, Direction::PosY, Direction::PosY, Direction::PosY, Direction::PosY, Direction::PosY]
    }
}

/// Pads a table of 6 directions into a table of 8 directions (where the extras are PosY).
/// ```rust, ignore
/// pad_direction_table[PosY, PosX, PosZ, NegY, NegX, NegZ]
/// ```
macro_rules! pad_direction_table {
    ($pos_y:ident, $pos_x:ident, $pos_z:ident, $neg_y:ident, $neg_x:ident, $neg_z:ident) => {
        [Direction::$pos_y, Direction::$pos_x, Direction::$pos_z, Direction::$neg_y, Direction::$neg_x, Direction::$neg_z, Direction::PosY, Direction::PosY]
    }
}

impl Rotation {
    #[cfg(feature = "glam")]
    const QUATS: [Quat; 24] = [
        Quat::from_xyzw(0.0, 0.0, 0.0, 1.0),
        Quat::from_xyzw(0.0, 0.70710677, 0.0, 0.70710677),
        Quat::from_xyzw(0.0, 1.0, 0.0, 0.0),
        Quat::from_xyzw(0.0, 0.70710677, 0.0, -0.70710677),
        Quat::from_xyzw(0.5, 0.5, -0.5, 0.5),
        Quat::from_xyzw(0.70710677, 0.70710677, 0.0, 0.0),
        Quat::from_xyzw(0.5, 0.5, 0.5, -0.5),
        Quat::from_xyzw(0.0, 0.0, 0.70710677, -0.70710677),
        Quat::from_xyzw(0.70710677, 0.0, 0.0, 0.70710677),
        Quat::from_xyzw(0.5, 0.5, 0.5, 0.5),
        Quat::from_xyzw(0.0, 0.70710677, 0.70710677, 0.0),
        Quat::from_xyzw(0.5, -0.5, -0.5, 0.5),
        Quat::from_xyzw(1.0, 0.0, 0.0, 0.0),
        Quat::from_xyzw(0.70710677, 0.0, 0.70710677, 0.0),
        Quat::from_xyzw(0.0, 0.0, 1.0, 0.0),
        Quat::from_xyzw(0.70710677, 0.0, -0.70710677, 0.0),
        Quat::from_xyzw(0.5, -0.5, 0.5, 0.5),
        Quat::from_xyzw(0.0, 0.0, 0.70710677, 0.70710677),
        Quat::from_xyzw(0.5, -0.5, -0.5, -0.5),
        Quat::from_xyzw(0.70710677, -0.70710677, 0.0, 0.0),
        Quat::from_xyzw(0.0, 0.70710677, -0.70710677, 0.0),
        Quat::from_xyzw(0.5, 0.5, -0.5, -0.5),
        Quat::from_xyzw(0.70710677, 0.0, 0.0, -0.70710677),
        Quat::from_xyzw(0.5, -0.5, 0.5, -0.5),
    ];

    pub const MIN: Self = Self(Rot::IDENTITY);
    // only 24 valid rotations (6 sides * 4 angles)
    pub const MAX: Self = Self(Rot::MAX);
    
    pub(crate) const ANGLE_MASK: u8 = 0b00000011;
    pub(crate) const ANGLE_MASK_I32: i32 = Self::ANGLE_MASK as i32;
    /// ((up << UP_SHIFT) & UP_MASK) | (angle & ANGLE_MASK)
    pub(crate) const UP_SHIFT: u32 = 2;
    
    pub const IDENTITY: Rotation = Rotation::new(Direction::PosY, 0);
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

    const REFACE_DIRECTIONS: [DirectionTable; 24] = {
        use Direction::*;
        [
            DirectionTable::new([PosY, PosX, PosZ, NegY, NegX, NegZ]),
            DirectionTable::new([PosY, NegZ, PosX, NegY, PosZ, NegX]),
            DirectionTable::new([PosY, NegX, NegZ, NegY, PosX, PosZ]),
            DirectionTable::new([PosY, PosZ, NegX, NegY, NegZ, PosX]),
            DirectionTable::new([PosX, NegZ, NegY, NegX, PosZ, PosY]),
            DirectionTable::new([PosX, PosY, NegZ, NegX, NegY, PosZ]),
            DirectionTable::new([PosX, PosZ, PosY, NegX, NegZ, NegY]),
            DirectionTable::new([PosX, NegY, PosZ, NegX, PosY, NegZ]),
            DirectionTable::new([PosZ, PosX, NegY, NegZ, NegX, PosY]),
            DirectionTable::new([PosZ, PosY, PosX, NegZ, NegY, NegX]),
            DirectionTable::new([PosZ, NegX, PosY, NegZ, PosX, NegY]),
            DirectionTable::new([PosZ, NegY, NegX, NegZ, PosY, PosX]),
            DirectionTable::new([NegY, PosX, NegZ, PosY, NegX, PosZ]),
            DirectionTable::new([NegY, PosZ, PosX, PosY, NegZ, NegX]),
            DirectionTable::new([NegY, NegX, PosZ, PosY, PosX, NegZ]),
            DirectionTable::new([NegY, NegZ, NegX, PosY, PosZ, PosX]),
            DirectionTable::new([NegX, PosZ, NegY, PosX, NegZ, PosY]),
            DirectionTable::new([NegX, PosY, PosZ, PosX, NegY, NegZ]),
            DirectionTable::new([NegX, NegZ, PosY, PosX, PosZ, NegY]),
            DirectionTable::new([NegX, NegY, NegZ, PosX, PosY, PosZ]),
            DirectionTable::new([NegZ, NegX, NegY, PosZ, PosX, PosY]),
            DirectionTable::new([NegZ, PosY, NegX, PosZ, NegY, PosX]),
            DirectionTable::new([NegZ, PosX, PosY, PosZ, NegX, NegY]),
            DirectionTable::new([NegZ, NegY, PosX, PosZ, PosY, NegX]),
        ]
    };

    const SOURCE_DIRECTIONS: [DirectionTable; 24] = {
        use Direction::*;
        [
            DirectionTable::new([PosY, PosX, PosZ, NegY, NegX, NegZ]),
            DirectionTable::new([PosY, PosZ, NegX, NegY, NegZ, PosX]),
            DirectionTable::new([PosY, NegX, NegZ, NegY, PosX, PosZ]),
            DirectionTable::new([PosY, NegZ, PosX, NegY, PosZ, NegX]),
            DirectionTable::new([NegZ, PosY, NegX, PosZ, NegY, PosX]),
            DirectionTable::new([PosX, PosY, NegZ, NegX, NegY, PosZ]),
            DirectionTable::new([PosZ, PosY, PosX, NegZ, NegY, NegX]),
            DirectionTable::new([NegX, PosY, PosZ, PosX, NegY, NegZ]),
            DirectionTable::new([NegZ, PosX, PosY, PosZ, NegX, NegY]),
            DirectionTable::new([PosX, PosZ, PosY, NegX, NegZ, NegY]),
            DirectionTable::new([PosZ, NegX, PosY, NegZ, PosX, NegY]),
            DirectionTable::new([NegX, NegZ, PosY, PosX, PosZ, NegY]),
            DirectionTable::new([NegY, PosX, NegZ, PosY, NegX, PosZ]),
            DirectionTable::new([NegY, PosZ, PosX, PosY, NegZ, NegX]),
            DirectionTable::new([NegY, NegX, PosZ, PosY, PosX, NegZ]),
            DirectionTable::new([NegY, NegZ, NegX, PosY, PosZ, PosX]),
            DirectionTable::new([NegZ, NegY, PosX, PosZ, PosY, NegX]),
            DirectionTable::new([PosX, NegY, PosZ, NegX, PosY, NegZ]),
            DirectionTable::new([PosZ, NegY, NegX, NegZ, PosY, PosX]),
            DirectionTable::new([NegX, NegY, NegZ, PosX, PosY, PosZ]),
            DirectionTable::new([NegZ, NegX, NegY, PosZ, PosX, PosY]),
            DirectionTable::new([PosX, NegZ, NegY, NegX, PosZ, PosY]),
            DirectionTable::new([PosZ, PosX, NegY, NegZ, NegX, PosY]),
            DirectionTable::new([NegX, PosZ, NegY, PosX, NegZ, PosY]),
        ]
    };

    // verified (2025-12-28)
    #[must_use]
    #[inline]
    pub const fn face_rotation(face: Direction, angle: i32) -> Self {
        Self::FACE_ROTATIONS[face as usize][wrap_angle(angle) as usize]
    }
    
    // verified (2025-12-28)
    #[must_use]
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
    
    #[must_use]
    #[inline]
    pub const unsafe fn from_u8_unchecked(value: u8) -> Self {
        Self(unsafe { Rot::from_u8_unchecked(value) })
    }
    
    #[must_use]
    #[inline]
    pub const fn from_u8(value: u8) -> Option<Self> {
        // Only first 24 values are valid rotations.
        if value > Self::MAX.0 as u8 {
            return None;
        }
        // SAFETY: Guard clause ensures that u8 is valid value.
        Some(unsafe { Self::from_u8_unchecked(value) })
    }
    
    #[must_use]
    #[inline]
    pub const fn from_u8_wrapping(value: u8) -> Self {
        unsafe { Self::from_u8_unchecked(wrap_rotation_u8(value)) }
    }
    
    #[must_use]
    #[inline]
    pub const fn new(up: Direction, angle: i32) -> Self {
        let up = up.rotation_discriminant();
        let angle = wrap_angle(angle) as u8;
        unsafe { Self::from_u8_unchecked(angle | (up << Self::UP_SHIFT)) }
    }
    
    #[must_use]
    #[inline]
    pub const fn from_up(up: Direction) -> Self {
        Self::new(up, 0)
    }
    
    #[must_use]
    #[inline(always)]
    pub const fn as_u8(self) -> u8 {
        self.0 as u8
    }
    
    /// Creates a new [Rotation] with [Direction::NegX] as the up direction.
    #[must_use]
    #[inline]
    pub const fn neg_x(angle: i32) -> Self {
        Self::new(Direction::NegX, angle)
    }
    
    /// Creates a new [Rotation] with [Direction::NegY] as the up direction.
    #[must_use]
    #[inline]
    pub const fn neg_y(angle: i32) -> Self {
        Self::new(Direction::NegY, angle)
    }
    
    /// Creates a new [Rotation] with [Direction::NegZ] as the up direction.
    #[must_use]
    #[inline]
    pub const fn neg_z(angle: i32) -> Self {
        Self::new(Direction::NegZ, angle)
    }
    
    /// Creates a new [Rotation] with [Direction::PosX] as the up direction.
    #[must_use]
    #[inline]
    pub const fn pos_x(angle: i32) -> Self {
        Self::new(Direction::PosX, angle)
    }
    
    /// Creates a new [Rotation] with [Direction::PosY] as the up direction.
    #[must_use]
    #[inline]
    pub const fn pos_y(angle: i32) -> Self {
        Self::new(Direction::PosY, angle)
    }
    
    #[must_use]
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
    #[must_use]
    pub const fn angles(self) -> [Self; 4] {
        let angle1 = self;
        let angle2 = angle1.reorient(angle1);
        let angle3 = angle2.reorient(angle1);
        [
            Self::IDENTITY,
            angle1,
            angle2,
            angle3,
        ]
    }
    
    // verified (2025-12-28)
    /// A helper function to create 3 rotations for a corner rotation group.
    /// The first rotation is unrotated, the second rotation is the target rotation,
    /// and the third rotation is the target rotation applied to itself.
    #[must_use]
    pub const fn corner_angles(self) -> [Self; 3] {
        let angle1 = self;
        let angle2 = angle1.reorient(angle1);
        [
            Self::IDENTITY,
            angle1,
            angle2
        ]
    }

    // verified (2025-12-28)
    #[must_use]
    #[inline]
    pub const fn with_flip(self, flip: super::Flip) -> Orientation {
        Orientation::new(self, flip)
    }

    // verified (2025-12-28)
    #[must_use]
    #[inline]
    pub const fn orientation(self) -> Orientation {
        self.with_flip(super::Flip::NONE)
    }
    
    #[must_use]
    #[inline(always)]
    pub const fn from_up_and_forward(up: Direction, forward: Direction) -> Option<Rotation> {
        // verified (2026-1-9)
        const UP_AND_FORWARD_MATRIX: CachePadded<[[Option<Rotation>; 8]; 6]> = {
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
            let mut matrix = CachePadded { value: [[None; 8]; 6] };
            let mut up_i = 0usize;
            while up_i < 6 {
                let up = Direction::INDEX_ORDER[up_i];
                let mut fwd_i = 0usize;
                while fwd_i < 6 {
                    let fwd = Direction::INDEX_ORDER[fwd_i];
                    matrix.value[up_i][fwd_i] = from_up_and_forward_slow(up, fwd);
                    fwd_i += 1;
                }
                up_i += 1;
            }
            matrix
        };
        UP_AND_FORWARD_MATRIX.value[up.rotation_discriminant() as usize][forward.rotation_discriminant() as usize]
    }
    
    #[must_use]
    #[inline(always)]
    pub const fn faces(self) -> Faces {
        const UP_FORWARD_RIGHT_TABLE: CachePadded<[Faces; 24]> = {
            let mut table = CachePadded::new([Faces::IDENTITY; 24]);
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
    #[must_use]
    #[inline]
    pub const fn cycle(self, offset: i32) -> Self {
        let index = self.0 as i64;
        // Don't use wrapping_add here, as tempting as it seems. It would be incorrect because 2**32 is not a multiple of 24.
        let new_index = (index + offset as i64).rem_euclid(24) as u8;
        unsafe { Self::from_u8_unchecked(new_index) }
    }

    #[must_use]
    #[inline]
    pub const fn angle(self) -> i32 {
        (self.0 as u8 & Self::ANGLE_MASK) as i32
    }

    #[inline]
    pub fn set_up(&mut self, up: Direction) {
        self.0 = unsafe {
            Rot::from_u8_unchecked((self.0 as u8 & Self::ANGLE_MASK) | (up.rotation_discriminant() << Self::UP_SHIFT))
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
    
    #[must_use]
    #[inline]
    pub fn iter() -> RotationIterator {
        RotationIterator::START
    }
    
    // verified (2026-1-5)
    /// Get the rotated `PosY` direction.
    #[must_use]
    pub const fn up(self) -> Direction {
        Self::REFACE_DIRECTIONS[self.0 as usize].get(Direction::PosY)
    }

    // verified (2026-1-5)
    /// Get the rotated `NegY` direction.
    #[must_use]
    pub const fn down(self) -> Direction {
        Self::REFACE_DIRECTIONS[self.0 as usize].get(Direction::NegY)
    }

    // verified (2026-1-5)
    /// Get the rotated `NegX` direction.
    #[must_use]
    pub const fn left(self) -> Direction {
        Self::REFACE_DIRECTIONS[self.0 as usize].get(Direction::NegX)
    }

    // verified (2026-1-5)
    /// Get the rotated `PosX` direction.
    #[must_use]
    pub const fn right(self) -> Direction {
        Self::REFACE_DIRECTIONS[self.0 as usize].get(Direction::PosX)
    }

    // verified (2026-1-5)
    /// Get the rotated `NegZ` direction.
    #[must_use]
    pub const fn forward(self) -> Direction {
        Self::REFACE_DIRECTIONS[self.0 as usize].get(Direction::NegZ)
    }

    // verified (2026-1-5)
    /// Get the rotated `PosZ` direction.
    #[must_use]
    pub const fn backward(self) -> Direction {
        Self::REFACE_DIRECTIONS[self.0 as usize].get(Direction::PosZ)
    }
    
    rotate_coord_fns!(i8, i16, i32, i64, i128, isize, f32, f64);

    // verified (2025-12-30)
    /// Rotates `coord`.
    #[must_use]
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

    #[must_use]
    #[inline(always)]
    pub const fn difference(self, rotation: Self)  -> Self {
        const DIFFERENCE_TABLE: [[Rotation; 32]; 24] = {
            let mut table = [[Rotation::IDENTITY; 32]; 24];
            let mut lhs_i = 0;
            while lhs_i < 24 {
                let mut rhs_i = 0;
                while rhs_i < 24 {
                    let lhs = unsafe { Rotation::from_u8_unchecked(lhs_i) };
                    let rhs = unsafe { Rotation::from_u8_unchecked(rhs_i) };
                    table[lhs_i as usize][rhs_i as usize]
                        = lhs.invert().reorient(rhs);
                    rhs_i += 1;
                }
                lhs_i += 1;
            }
            table
        };
        DIFFERENCE_TABLE[self.0 as usize][rotation.0 as usize]
    }
    
    // verified (2025-12-28): reface and source_face are symmetrical.
    /// Rotates direction.
    #[must_use]
    pub const fn reface(self, direction: Direction) -> Direction {
        // Self::REFACE_TABLE[direction as usize][self.0 as usize]
        Self::REFACE_DIRECTIONS[self.0 as usize].get(direction)
    }

    // verified (2025-12-28): source_face and reface are symmetrical.
    /// Tells which [Direction] rotated to `destination`.
    #[must_use]
    pub const fn source_face(self, destination: Direction) -> Direction {
        Self::SOURCE_DIRECTIONS[self.0 as usize].get(destination)
    }

    // verified (2025-12-28)
    // double verified (2025-12-29)
    /// Gets the angle of the face oriented to `world_face`.
    #[must_use]
    pub fn face_angle(self, world_face: Direction) -> u8 {
        use Direction::*;
        use Rot::*;
        match (self.0, world_face) {
            (NegX0, NegX) => 0,
            (NegX0, NegY) => 1,
            (NegX0, NegZ) => 3,
            (NegX0, PosX) => 2,
            (NegX0, PosY) => 1,
            (NegX0, PosZ) => 1,
            (NegY0, NegX) => 2,
            (NegY0, NegY) => 0,
            (NegY0, NegZ) => 2,
            (NegY0, PosX) => 2,
            (NegY0, PosY) => 0,
            (NegY0, PosZ) => 2,
            (NegZ0, NegX) => 1,
            (NegZ0, NegY) => 2,
            (NegZ0, NegZ) => 0,
            (NegZ0, PosX) => 3,
            (NegZ0, PosY) => 0,
            (NegZ0, PosZ) => 2,
            (PosX0, NegX) => 2,
            (PosX0, NegY) => 3,
            (PosX0, NegZ) => 1,
            (PosX0, PosX) => 0,
            (PosX0, PosY) => 3,
            (PosX0, PosZ) => 3,
            (PosY0, NegX) => 0,
            (PosY0, NegY) => 0,
            (PosY0, NegZ) => 0,
            (PosY0, PosX) => 0,
            (PosY0, PosY) => 0,
            (PosY0, PosZ) => 0,
            (PosZ0, NegX) => 3,
            (PosZ0, NegY) => 0,
            (PosZ0, NegZ) => 2,
            (PosZ0, PosX) => 1,
            (PosZ0, PosY) => 2,
            (PosZ0, PosZ) => 0,
            (NegX1, NegX) => 1,
            (NegX1, NegY) => 1,
            (NegX1, NegZ) => 3,
            (NegX1, PosX) => 1,
            (NegX1, PosY) => 1,
            (NegX1, PosZ) => 1,
            (NegY1, NegX) => 2,
            (NegY1, NegY) => 1,
            (NegY1, NegZ) => 2,
            (NegY1, PosX) => 2,
            (NegY1, PosY) => 3,
            (NegY1, PosZ) => 2,
            (NegZ1, NegX) => 1,
            (NegZ1, NegY) => 2,
            (NegZ1, NegZ) => 1,
            (NegZ1, PosX) => 3,
            (NegZ1, PosY) => 0,
            (NegZ1, PosZ) => 1,
            (PosX1, NegX) => 1,
            (PosX1, NegY) => 3,
            (PosX1, NegZ) => 1,
            (PosX1, PosX) => 1,
            (PosX1, PosY) => 3,
            (PosX1, PosZ) => 3,
            (PosY1, NegX) => 0,
            (PosY1, NegY) => 3,
            (PosY1, NegZ) => 0,
            (PosY1, PosX) => 0,
            (PosY1, PosY) => 1,
            (PosY1, PosZ) => 0,
            (PosZ1, NegX) => 3,
            (PosZ1, NegY) => 0,
            (PosZ1, NegZ) => 1,
            (PosZ1, PosX) => 1,
            (PosZ1, PosY) => 2,
            (PosZ1, PosZ) => 1,
            (NegX2, NegX) => 2,
            (NegX2, NegY) => 1,
            (NegX2, NegZ) => 3,
            (NegX2, PosX) => 0,
            (NegX2, PosY) => 1,
            (NegX2, PosZ) => 1,
            (NegY2, NegX) => 2,
            (NegY2, NegY) => 2,
            (NegY2, NegZ) => 2,
            (NegY2, PosX) => 2,
            (NegY2, PosY) => 2,
            (NegY2, PosZ) => 2,
            (NegZ2, NegX) => 1,
            (NegZ2, NegY) => 2,
            (NegZ2, NegZ) => 2,
            (NegZ2, PosX) => 3,
            (NegZ2, PosY) => 0,
            (NegZ2, PosZ) => 0,
            (PosX2, NegX) => 0,
            (PosX2, NegY) => 3,
            (PosX2, NegZ) => 1,
            (PosX2, PosX) => 2,
            (PosX2, PosY) => 3,
            (PosX2, PosZ) => 3,
            (PosY2, NegX) => 0,
            (PosY2, NegY) => 2,
            (PosY2, NegZ) => 0,
            (PosY2, PosX) => 0,
            (PosY2, PosY) => 2,
            (PosY2, PosZ) => 0,
            (PosZ2, NegX) => 3,
            (PosZ2, NegY) => 0,
            (PosZ2, NegZ) => 0,
            (PosZ2, PosX) => 1,
            (PosZ2, PosY) => 2,
            (PosZ2, PosZ) => 2,
            (NegX3, NegX) => 3,
            (NegX3, NegY) => 1,
            (NegX3, NegZ) => 3,
            (NegX3, PosX) => 3,
            (NegX3, PosY) => 1,
            (NegX3, PosZ) => 1,
            (NegY3, NegX) => 2,
            (NegY3, NegY) => 3,
            (NegY3, NegZ) => 2,
            (NegY3, PosX) => 2,
            (NegY3, PosY) => 1,
            (NegY3, PosZ) => 2,
            (NegZ3, NegX) => 1,
            (NegZ3, NegY) => 2,
            (NegZ3, NegZ) => 3,
            (NegZ3, PosX) => 3,
            (NegZ3, PosY) => 0,
            (NegZ3, PosZ) => 3,
            (PosX3, NegX) => 3,
            (PosX3, NegY) => 3,
            (PosX3, NegZ) => 1,
            (PosX3, PosX) => 3,
            (PosX3, PosY) => 3,
            (PosX3, PosZ) => 3,
            (PosY3, NegX) => 0,
            (PosY3, NegY) => 1,
            (PosY3, NegZ) => 0,
            (PosY3, PosX) => 0,
            (PosY3, PosY) => 3,
            (PosY3, PosZ) => 0,
            (PosZ3, NegX) => 3,
            (PosZ3, NegY) => 0,
            (PosZ3, NegZ) => 3,
            (PosZ3, PosX) => 1,
            (PosZ3, PosY) => 2,
            (PosZ3, PosZ) => 3,
        }
    }

    // verified (2025-12-28)
    /// Rotate a [Rotation] by another [Rotation].
    #[must_use]
    pub const fn reorient(self, rotation: Self) -> Self {
        const REORIENT_TABLE: [[Rotation; 32]; 24] = {
            const fn reorient_slow(lhs: Rotation, rhs: Rotation) -> Rotation {
                // What??? I know I wrote this code, but this is kinda nuts.
                let up = lhs.up();
                let fwd = lhs.forward();
                let rot_up = rhs.reface(up);
                let rot_fwd = rhs.reface(fwd);
                // Pattern matching is used here because it's a const fn and unwrap()
                // won't work.
                let Some(rot) = Rotation::from_up_and_forward(rot_up, rot_fwd) else {
                    unreachable!()
                };
                rot
            }
            let mut table = [[Rotation::IDENTITY; 32]; 24];
            let mut lhs_i = 0;
            while lhs_i < 24 {
                let mut rhs_i = 0;
                while rhs_i < 24 {
                    let lhs = unsafe { Rotation::from_u8_unchecked(lhs_i) };
                    let rhs = unsafe { Rotation::from_u8_unchecked(rhs_i) };
                    table[lhs_i as usize][rhs_i as usize] = reorient_slow(lhs, rhs);
                    rhs_i += 1;
                }
                lhs_i += 1;
            }
            table
        };
        REORIENT_TABLE[self.0 as usize][rotation.0 as usize]
    }

    // verified (2025-12-28)
    /// Rotate a [Rotation] by the inverse of another [Rotation].
    #[must_use]
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

    pub const fn conjugate(self, rotation: Self) -> Self {
        self.invert().reorient(rotation).reorient(self)
    }
    
    // verified (2025-12-28)
    /// Creates a [Rotation] that when rotated by the original will create the base [Rotation].
    #[must_use]
    #[inline]
    pub const fn invert(self) -> Self {
        Self::IDENTITY.deorient(self)
    }

    #[must_use]
    #[inline]
    pub const fn rotate_x(self, angle: i32) -> Self {
        self.reorient(Self::X_ROTATIONS[wrap_angle(angle) as usize])
    }

    #[must_use]
    #[inline]
    pub const fn rotate_y(self, angle: i32) -> Self {
        self.reorient(Self::Y_ROTATIONS[wrap_angle(angle) as usize])
    }

    #[must_use]
    #[inline]
    pub const fn rotate_z(self, angle: i32) -> Self {
        self.reorient(Self::Z_ROTATIONS[wrap_angle(angle) as usize])
    }

    /// Rotate `face` counter-clockwise by `angle`. Use a negative `angle` to rotate clockwise.
    #[must_use]
    #[inline]
    pub const fn rotate_face(self, face: Direction, angle: i32) -> Self {
        let rot = Self::face_rotation(face, angle);
        self.reorient(rot)
    }

    /// Rotate corner clockwise by `angle`.
    #[must_use]
    #[inline]
    pub const fn rotate_corner(self, x: i32, y: i32, z: i32, angle: i32) -> Self {
        let rot = Self::corner_rotation(x, y, z, angle);
        self.reorient(rot)
    }

    #[must_use]
    #[inline(always)]
    pub const fn display(self, short: bool) -> RotationDisplay {
        if short {
            RotationDisplay::Short(RotationShortDisplay(self))
        } else {
            RotationDisplay::Long(RotationLongDisplay(self))
        }
    }

    #[cfg(feature = "glam")]
    #[must_use]
    pub fn to_matrix(self) -> glam::Mat4 {
        let up = self.reface(Direction::PosY).to_vec3a();
        let forward = self.reface(Direction::PosZ).to_vec3a();
        let right = self.reface(Direction::NegX).to_vec3a();
        Mat4::from_cols(
            right.extend(0.0),
            up.extend(0.0),
            forward.extend(0.0),
            Vec4::W,
        )
    }

    #[cfg(feature = "glam")]
    #[inline]
    #[must_use]
    pub const fn to_quat(self) -> Quat {
        Self::QUATS[self.0 as usize]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RotationIterator {
    rotation: u8,
}

impl RotationIterator {
    pub const START: Self = Self { rotation: 0 };
    
    #[must_use]
    #[inline]
    pub const fn start_at(rotation: Rotation) -> Self {
        Self { rotation: rotation.0 as u8 }
    }
    
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self::START
    }
    
    #[must_use]
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

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct RotationShortDisplay(pub Rotation);

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct RotationLongDisplay(pub Rotation);

#[derive(Debug, Clone, Copy)]
pub enum RotationDisplay {
    Short(RotationShortDisplay),
    Long(RotationLongDisplay),
}

impl std::fmt::Display for RotationShortDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let up = self.0.up();
        write!(f,
            "{polarity}{axis}{angle}",
            polarity = up.polarity().display(true),
            axis = up.axis().display(),
            angle = self.0.angle(),
        )
    }
}

impl std::fmt::Display for RotationLongDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let up = self.0.up();
        let angle = self.0.angle();
        write!(f,
            "Rotation(up={polarity}{axis},angle={angle})",
            polarity = up.polarity().display(false),
            axis = up.axis().display(),
            angle = angle,
        )
    }
}

impl std::fmt::Display for RotationDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RotationDisplay::Short(disp) => write!(f, "{disp}"),
            RotationDisplay::Long(disp) => write!(f, "{disp}"),
        }
    }
}

#[cfg(test)]
mod rotation_tests {
    use super::*;

    #[test]
    fn difference_test() {
        for lhs in Rotation::iter() {
            for rhs in Rotation::iter() {
                let difference = lhs.difference(rhs);
                assert_eq!(lhs.reorient(difference), rhs);
            }
        }
    }
}
