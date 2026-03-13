//! This library is used for orienting voxel blocks in a voxel library, but it can also be used for other purposes involving cube-orientations.
//! Each orientation type (`Orientation`, `Rotation`, and `Flip`) are implementing using specialized enums as their backing.
//! This allows them to take advantage of the niche optimization, meaning that `Option<Orientation>` is 1 byte, and so is `Option<Option<Orientation>>`.
//! There are over 50 niches that `Orientation` can take advantage of, and even more for `Rotation` and `Flip`.

/*====================================================================================*\
||I find that sometimes, using big lookup tables is the best solution to your problem.||
||All of the functionality in this library can be done logically, but I think it will ||
||probably execute faster with the lookup tables and such. The logic to construct     ||
||these tables was fairly complicated.                                                ||
\*====================================================================================*/

/*
Goals:
- Switch from using u8 to using specialized enums for niche optimization.
Changelog:
[Nothing here yet]
*/

pub mod axis;
pub mod direction;
pub mod faces;
pub mod flip;
pub mod orient_table;
pub mod orientation_enum;
pub mod orientation;
pub mod polarity;
pub mod rotation;

pub use axis::Axis;
pub use direction::Direction;
pub use flip::Flip;
pub use orientation::Orientation;
pub use rotation::Rotation;
use vcore::lowlevel::cache_padded::CachePadded;

// this code feels like cheating.

// verified (2025-12-28)
// This packing format should remain consistent, and should be considered permanent.
// Field
// Flip    : 0..3 (3 bits)
//      X: 0
//      Y: 1
//      Z: 2
// Rotation: 3..8 (5 bits)
//      angle: 3..5 (2 bits)
//      up   : 5..8 (3 bits)
#[inline(always)]
pub const fn pack_flip_and_rotation(flip: Flip, rotation: Rotation) -> u8 {
    flip.0 as u8 | ((rotation.0 as u8) << 3)
}

#[inline(always)]
pub const fn unpack_flip_and_rotation(packed: u8) -> (Flip, Rotation) {
    let rotation = (packed >> 3) % 24;
    (
        Flip::from_u8_wrapping(packed),
        unsafe { Rotation::from_u8_unchecked(rotation) },
    )
}

// verified (2025-12-28)
/// Wrap a cube face angle within a safe range (0..4).
/// For cube orientations, faces can have 4 angles (up = up, up = left, up = down, up = right).
#[inline(always)]
pub const fn wrap_angle(angle: i32) -> i32 {
    angle & Rotation::ANGLE_MASK_I32
}

// This should be cache aligned on the majority of systems.
/// A simple array wrapper that aligns the array to 64 bytes, which
/// is the most typical cache line size on modern (circa 2026) hardware.
#[repr(C, align(64))]
pub struct CacheAlignedArray<T: 'static + Sized, const LEN: usize> {
    pub array: CachePadded<[T; LEN]>,
}

impl<T, const LEN: usize> CacheAlignedArray<T, LEN> {
    #[must_use]
    #[inline(always)]
    pub const fn new(array: [T; LEN]) -> Self {
        Self { array: CachePadded { value: array } }
    }
}

impl<T, const LEN: usize> ::core::ops::Deref for CacheAlignedArray<T, LEN> {
    type Target = [T];
    
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.array.value
    }
}

impl<T, const LEN: usize> ::core::ops::DerefMut for CacheAlignedArray<T, LEN> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.array.value
    }
}

impl<T, const LEN: usize, I: ::core::slice::SliceIndex<[T], Output = T>> ::core::ops::Index<I> for CacheAlignedArray<T, LEN> {
    type Output = T;
    #[inline(always)]
    fn index(&self, index: I) -> &Self::Output {
        &self.array[index]
    }
}

impl<T, const LEN: usize, I: ::core::slice::SliceIndex<[T], Output = T>> ::core::ops::IndexMut<I> for CacheAlignedArray<T, LEN> {
    #[inline(always)]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.array[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    #[ignore]
    fn quick_testing_sandbox() {
        /*============================================================*\
        ||This is a little scratchpad space to test snippets of code. ||
        ||Make sure to delete code after you are done testing.        ||
        \*============================================================*/
    }
    
    #[test]
    fn orientation_test() {
        for dir in Direction::iter() {
            let orient = Orientation::new(Rotation::new(dir, 1), Flip::NONE);
            
            let orient_face = orient.rotation().up();
            assert_eq!(orient.forward(), orient_face.left(), "forward: {dir}");
            
            assert_eq!(
                orient.reface(Direction::FORWARD),
                orient_face.left(),
                "{dir}: Forward -> Left"
            );
            assert_eq!(
                orient.reface(Direction::LEFT),
                orient_face.down(),
                "{dir}: Left -> Down"
            );
            assert_eq!(
                orient.reface(Direction::BACKWARD),
                orient_face.right(),
                "{dir}: Back -> Right"
            );
            assert_eq!(
                orient.reface(Direction::RIGHT),
                orient_face.up(),
                "{dir}: Right -> Up"
            );
        }
    }
    
    /// verifies [Rotation::reface] function. By extension, also verifies [Rotation::up], [Rotation::down], [Rotation::left], [Rotation::right], [Rotation::forward], and [Rotation::backward].
    #[test]
    fn orientation_query_test() {
        // Since all of the functions used in this function are verified, this is
        // another way to rotate faces. This is used to verify the `reface` function.
        fn rotate_world(up: Direction, angle: i32, world: Direction) -> Direction {
            match world {
                Direction::NegX => up.left_at_angle(angle),
                Direction::NegY => up.invert(),
                Direction::NegZ => up.up_at_angle(angle),
                Direction::PosX => up.right_at_angle(angle),
                Direction::PosY => up,
                Direction::PosZ => up.down_at_angle(angle),
            }
        }
        for angle in 0..4 {
            for up in Direction::iter() {
                for world in Direction::iter() {
                    let rotation = Rotation::new(up, angle);
                    let rot_world = rotation.reface(world);
                    let rot_world_alt = rotate_world(up, angle, world);
                    assert_eq!(rot_world, rot_world_alt, "(rot: [up: {up}, angle: {angle}], world: {world})");
                }
            }
        }
    }
    
    // verifies `source_face` function as well as symmetry between `reface` and `source_face`.
    #[test]
    fn reface_sourceface_symmetry_test() {
        let start_time = std::time::Instant::now();
        for angle in 0..4 {
            for up in Direction::iter() {
                let rotation = Rotation::new(up, angle);
                for world in Direction::iter() {
                    let refaced = rotation.reface(world);
                    let source = rotation.source_face(refaced);
                    assert_eq!(source, world);
                }
            }
        }
        let elapsed = start_time.elapsed();
        println!("Elapsed Time: {elapsed:.3?}");
    }
    
    #[test]
    fn transform_coord_test() {
        for flip in 0..8 {
            let flip = Flip(unsafe { flip::FlipState::from_u8_unchecked(flip) });
            for angle in 0..4 {
                for up in Direction::iter() {
                    let orientation = Orientation::new(Rotation::new(up, angle), flip);
                    for face in Direction::iter() {
                        let reface = orientation.reface(face);
                        let face_coord = face.to_ituple();
                        let reface_coord = reface.to_ituple();
                        let trans_face_coord = orientation.transform_i32(face_coord);
                        assert_eq!(reface_coord, trans_face_coord, "{orientation} -> {face}");
                    }
                }
            }
        }
    }
}