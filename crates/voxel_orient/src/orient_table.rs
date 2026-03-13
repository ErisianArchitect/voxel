use crate::{CacheAlignedArray, Orientation, direction::Direction, flip::Flip, rotation::Rotation};
use paste::paste;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum AxisMap {
    PosX,
    PosY,
    NegX,
    NegY,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum AxisMapper {
    PosXPosY,
    NegXPosY,
    PosXNegY,
    NegXNegY,
    PosYPosX,
    NegYPosX,
    PosYNegX,
    NegYNegX,
}

impl AxisMapper {
    // verified (2026-1-5)
    pub(crate) const fn from_pair(x: AxisMap, y: AxisMap) -> Self {
        use AxisMap::*;
        match (x, y) {
            (NegX, PosY) => Self::NegXPosY,
            (PosX, PosY) => Self::PosXPosY,
            (PosX, NegY) => Self::PosXNegY,
            (NegX, NegY) => Self::NegXNegY,
            (PosY, PosX) => Self::PosYPosX,
            (NegY, PosX) => Self::NegYPosX,
            (PosY, NegX) => Self::PosYNegX,
            (NegY, NegX) => Self::NegYNegX,
            (PosX, PosX) => panic!("Invalid Pair (PosX, PosX)"),
            (PosX, NegX) => panic!("Invalid Pair (PosX, NegX)"),
            (PosY, PosY) => panic!("Invalid Pair (PosY, PosY)"),
            (PosY, NegY) => panic!("Invalid Pair (PosY, NegY)"),
            (NegX, PosX) => panic!("Invalid Pair (NegX, PosX)"),
            (NegX, NegX) => panic!("Invalid Pair (NegX, NegX)"),
            (NegY, PosY) => panic!("Invalid Pair (NegY, PosY)"),
            (NegY, NegY) => panic!("Invalid Pair (NegY, NegY)"),
        }
    }
    
    // verified (2026-1-5)
    // This code is used in the generation of the table.
    // It's a bit redundant, but I don't feel like regenerating the table.
    #[allow(unused)]
    pub const fn x(self) -> AxisMap {
        match self {
            AxisMapper::PosXPosY => AxisMap::PosX,
            AxisMapper::NegXPosY => AxisMap::NegX,
            AxisMapper::PosXNegY => AxisMap::PosX,
            AxisMapper::NegXNegY => AxisMap::NegX,
            AxisMapper::PosYPosX => AxisMap::PosY,
            AxisMapper::NegYPosX => AxisMap::NegY,
            AxisMapper::PosYNegX => AxisMap::PosY,
            AxisMapper::NegYNegX => AxisMap::NegY,
        }
    }
    
    // verified (2026-1-5)
    // This code is used in the generation of the table.
    // It's a bit redundant, but I don't feel like regenerating the table.
    #[allow(unused)]
    pub const fn y(self) -> AxisMap {
        match self {
            AxisMapper::PosXPosY => AxisMap::PosY,
            AxisMapper::NegXPosY => AxisMap::PosY,
            AxisMapper::PosXNegY => AxisMap::NegY,
            AxisMapper::NegXNegY => AxisMap::NegY,
            AxisMapper::PosYPosX => AxisMap::PosX,
            AxisMapper::NegYPosX => AxisMap::PosX,
            AxisMapper::PosYNegX => AxisMap::NegX,
            AxisMapper::NegYNegX => AxisMap::NegX,
        }
    }
    
    // verified (2026-1-5)
    // This code is used in the generation of the table.
    // It's a bit redundant, but I don't feel like regenerating the table.
    #[allow(unused)]
    pub const fn to_pair(self) -> (AxisMap, AxisMap) {
        match self {
            AxisMapper::PosXPosY => (AxisMap::PosX, AxisMap::PosY),
            AxisMapper::NegXPosY => (AxisMap::NegX, AxisMap::PosY),
            AxisMapper::PosXNegY => (AxisMap::PosX, AxisMap::NegY),
            AxisMapper::NegXNegY => (AxisMap::NegX, AxisMap::NegY),
            AxisMapper::PosYPosX => (AxisMap::PosY, AxisMap::PosX),
            AxisMapper::NegYPosX => (AxisMap::NegY, AxisMap::PosX),
            AxisMapper::PosYNegX => (AxisMap::PosY, AxisMap::NegX),
            AxisMapper::NegYNegX => (AxisMap::NegY, AxisMap::NegX),
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct CoordMap {
    pub mapper: AxisMapper,
}

macro_rules! coord_map_impl {
    ($(
        $type:ty
    ),*$(,)?) => {
        $(
            paste!{
                // verified (2026-1-5)
                #[inline]
                pub const fn [<map_ $type>](self, (x, y): ($type, $type)) -> ($type, $type) {
                    match self.mapper {
                        AxisMapper::PosXPosY => (x, y),
                        AxisMapper::NegXPosY => (-x, y),
                        AxisMapper::PosXNegY => (x, -y),
                        AxisMapper::NegXNegY => (-x, -y),
                        AxisMapper::PosYPosX => (y, x),
                        AxisMapper::NegYPosX => (-y, x),
                        AxisMapper::PosYNegX => (y, -x),
                        AxisMapper::NegYNegX => (-y, -x),
                    }
                }
                
                // TODO: Use this code
                // pub const fn [<map_many_ $type>](self, array: &mut [($type, $type)]) {
                //     match self.mapper {
                //         AxisMapper::PosXPosY => (/* No change necessary */),
                //         AxisMapper::NegXPosY => {
                //             let mut index = 0usize;
                //             while index < array.len() {
                //                 let (x, y) = array[index];
                //                 array[index] = (-x, y);
                //                 index += 1;
                //             }
                //         },
                //         AxisMapper::PosXNegY => {
                //             let mut index = 0usize;
                //             while index < array.len() {
                //                 let (x, y) = array[index];
                //                 array[index] = (x, -y);
                //                 index += 1;
                //             }
                //         },
                //         AxisMapper::NegXNegY => {
                //             let mut index = 0usize;
                //             while index < array.len() {
                //                 let (x, y) = array[index];
                //                 array[index] = (-x, -y);
                //                 index += 1;
                //             }
                //         },
                //         AxisMapper::PosYPosX => {
                //             let mut index = 0usize;
                //             while index < array.len() {
                //                 let (x, y) = array[index];
                //                 array[index] = (y, x);
                //                 index += 1;
                //             }
                //         },
                //         AxisMapper::NegYPosX => {
                //             let mut index = 0usize;
                //             while index < array.len() {
                //                 let (x, y) = array[index];
                //                 array[index] = (-y, x);
                //                 index += 1;
                //             }
                //         },
                //         AxisMapper::PosYNegX => {
                //             let mut index = 0usize;
                //             while index < array.len() {
                //                 let (x, y) = array[index];
                //                 array[index] = (y, -x);
                //                 index += 1;
                //             }
                //         },
                //         AxisMapper::NegYNegX => {
                //             let mut index = 0usize;
                //             while index < array.len() {
                //                 let (x, y) = array[index];
                //                 array[index] = (-y, -x);
                //                 index += 1;
                //             }
                //         },
                //     }
                // }
                
                // TODO: Use this code
                // #[inline]
                // pub const fn [<mapper_ $type>](self) -> fn(($type, $type)) -> ($type, $type) {
                //     match self.mapper {
                //         AxisMapper::PosXPosY => |(x, y): ($type, $type)| (x, y),
                //         AxisMapper::NegXPosY => |(x, y): ($type, $type)| (-x, y),
                //         AxisMapper::PosXNegY => |(x, y): ($type, $type)| (x, -y),
                //         AxisMapper::NegXNegY => |(x, y): ($type, $type)| (-x, -y),
                //         AxisMapper::PosYPosX => |(x, y): ($type, $type)| (y, x),
                //         AxisMapper::NegYPosX => |(x, y): ($type, $type)| (-y, x),
                //         AxisMapper::PosYNegX => |(x, y): ($type, $type)| (y, -x),
                //         AxisMapper::NegYNegX => |(x, y): ($type, $type)| (-y, -x),
                //     }
                // }
            }
        )*
    };
}

impl CoordMap {
    pub const DEFAULT: Self = Self { mapper: AxisMapper::PosXPosY };
    pub(crate) const fn new(x: AxisMap, y: AxisMap) -> Self {
        Self { mapper: AxisMapper::from_pair(x, y) }
    }
    
    // verified (2026-1-5)
    #[allow(unused)]
    pub const fn x(self) -> AxisMap {
        match self.mapper {
            AxisMapper::PosXPosY => AxisMap::PosX,
            AxisMapper::NegXPosY => AxisMap::NegX,
            AxisMapper::PosXNegY => AxisMap::PosX,
            AxisMapper::NegXNegY => AxisMap::NegX,
            AxisMapper::PosYPosX => AxisMap::PosY,
            AxisMapper::NegYPosX => AxisMap::NegY,
            AxisMapper::PosYNegX => AxisMap::PosY,
            AxisMapper::NegYNegX => AxisMap::NegY,
        }
    }
    
    // verified (2026-1-5)
    #[allow(unused)]
    pub const fn y(self) -> AxisMap {
        match self.mapper {
            AxisMapper::PosXPosY => AxisMap::PosY,
            AxisMapper::NegXPosY => AxisMap::PosY,
            AxisMapper::PosXNegY => AxisMap::NegY,
            AxisMapper::NegXNegY => AxisMap::NegY,
            AxisMapper::PosYPosX => AxisMap::PosX,
            AxisMapper::NegYPosX => AxisMap::PosX,
            AxisMapper::PosYNegX => AxisMap::NegX,
            AxisMapper::NegYNegX => AxisMap::NegX,
        }
    }
    
    coord_map_impl!(
        i8,
        i16,
        i32,
        i64,
        i128,
        isize,
        f32,
        f64,
    );
}

#[repr(transparent)]
pub(crate) struct CoordMapTable {
    pub(crate) table: CacheAlignedArray<CoordMap, 1152>,
}

impl CoordMapTable {
    #[inline]
    pub(crate) const fn new(table: CacheAlignedArray<CoordMap, 1152>) -> Self {
        Self { table }
    }
    
    // verified (2026-1-5)
    #[inline(always)]
    pub(crate) const fn table_index(rotation: Rotation, flip: Flip, face: Direction) -> usize {
        let flip = flip.0 as usize;
        let rot = rotation.0 as usize;
        let face = face.rotation_discriminant() as usize;
        flip * 144 + rot * 6 + face
    }
    
    // verified (2026-1-5)
    #[inline(always)]
    pub(crate) const fn get(&self, rotation: Rotation, flip: Flip, face: Direction) -> CoordMap {
        self.table.array.value[Self::table_index(rotation, flip, face)]
    }
}
// MAP_FACE_COORD_TABLE and SOURCE_FACE_COORD_TABLE are used for mapping UV coordinates.
// verified (2026-1-5)
pub(crate) const MAP_FACE_COORD_TABLE: CoordMapTable = {
    const fn map_face_coord_naive(orientation: Orientation, face: Direction) -> CoordMap {
        // First I will attempt a naive implementation, then I will use the naive implementation to generate code
        // for a more optimized implementation.
        // First get the source face
        let source_face = orientation.source_face(face);
        // next, get the up, right, down, and left for the source face and arg face.
        let face_up = face.up();
        let face_right = face.right();
        let src_up = source_face.up();
        let src_right = source_face.right();
        let src_down = source_face.down();
        let src_left = source_face.left();
        // Next, reface the src_dir faces
        let rsrc_up = orientation.reface(src_up);
        let rsrc_right = orientation.reface(src_right);
        let rsrc_down = orientation.reface(src_down);
        let rsrc_left = orientation.reface(src_left);
        // Now match up the faces
        // x_map and y_map must use right and up faces because the polarity is independent.
        let x_map = if face_right as u8 == rsrc_right as u8 { // PosX :facing: PosX, x maps to PosX (no change).
            AxisMap::PosX
        } else if face_right as u8 == rsrc_up as u8 { // PosX :facing: PosY, 1 turn counter-clockwise, NegY in place of PosX
            AxisMap::NegY
        } else if face_right as u8 == rsrc_left as u8 { // PosX :facing: NegX, x maps to NegX
            AxisMap::NegX
        } else { // PosX facing NegY, 1 clockwise turn, PosY is now in place of PosX
            AxisMap::PosY
        };
        
        let y_map = if face_up as u8 == rsrc_up as u8 {
            AxisMap::PosY
        } else if face_up as u8 == rsrc_left as u8 {
            AxisMap::PosX
        } else if face_up as u8 == rsrc_down as u8 {
            AxisMap::NegY
        } else {
            AxisMap::NegX
        };
        CoordMap::new(x_map, y_map)
    }
    let mut arr = CacheAlignedArray::new([CoordMap::DEFAULT; 1152]);
    let mut index = 0usize;
    let mut flip_i = 0u8;
    // IMPORTANT: The order of the loops is important.
    // :::::::::: Do not modify anything unless you are
    // :::::::::: absolutely certain that you know what
    // :::::::::: you are doing.
    while flip_i < 8 {
        let mut rot_i = 0u8;
        while rot_i < 24 {
            let mut dir_i = 0usize;
            while dir_i < Direction::INDEX_ORDER.len() {
                let orientation = Orientation::new(
                    unsafe { Rotation::from_u8_unchecked(rot_i) },
                    unsafe { Flip::from_u8_unchecked(flip_i) },
                );
                let face = Direction::INDEX_ORDER[dir_i];
                arr.array.value[index] = map_face_coord_naive(orientation, face);
                index += 1;
                dir_i += 1;
            }
            rot_i += 1;
        }
        flip_i += 1;
    }
    CoordMapTable::new(arr)
};

// verified (2026-1-5)
pub(crate) const SOURCE_FACE_COORD_TABLE: CoordMapTable = {
    const fn source_face_coord_naive(orientation: Orientation, face: Direction) -> CoordMap {
        // First I will attempt a naive implementation, then I will use the naive implementation to generate code
        // for a more optimized implementation.
        // First get the source face
        let source_face = orientation.source_face(face);
        // next, get the up, right, down, and left for the source face and arg face.
        let src_up = source_face.up();
        let src_right = source_face.right();
        let face_up = face.up();
        let face_right = face.right();
        let face_down = face.down();
        let face_left = face.left();
        // Next, reface the src_dir faces
        let rsrc_up = orientation.reface(src_up);
        let rsrc_right = orientation.reface(src_right);
        // Now match up the faces
        let x_map = if rsrc_right as u8 == face_right as u8 {
            AxisMap::PosX
        } else if rsrc_right as u8 == face_down as u8 {
            AxisMap::PosY
        } else if rsrc_right as u8 == face_left as u8 {
            AxisMap::NegX
        } else {
            AxisMap::NegY
        };
        let y_map = if rsrc_up as u8 == face_up as u8 {
            AxisMap::PosY
        } else if rsrc_up as u8 == face_right as u8 {
            AxisMap::NegX
        } else if rsrc_up as u8 == face_down as u8 {
            AxisMap::NegY
        } else {
            AxisMap::PosX
        };
        CoordMap::new(x_map, y_map)
    }
    let mut arr = CacheAlignedArray::new([CoordMap::DEFAULT; 1152]);
    let mut index = 0usize;
    let mut flip_i = 0u8;
    // IMPORTANT: The order of the loops is important.
    // :::::::::: Do not modify anything unless you are
    // :::::::::: absolutely certain that you know what
    // :::::::::: you are doing.
    while flip_i < 8 {
        let mut rot_i = 0u8;
        while rot_i < 24 {
            let mut dir_i = 0usize;
            while dir_i < Direction::INDEX_ORDER.len() {
                let orientation = Orientation::new(
                    unsafe { Rotation::from_u8_unchecked(rot_i) },
                    unsafe { Flip::from_u8_unchecked(flip_i) },
                );
                let face = Direction::INDEX_ORDER[dir_i];
                arr.array.value[index] = source_face_coord_naive(orientation, face);
                index += 1;
                dir_i += 1;
            }
            rot_i += 1;
        }
        flip_i += 1;
    }
    CoordMapTable::new(arr)
};