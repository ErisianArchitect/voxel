// Copyright © 2026 Ada F. <https://github.com/ErisianArchitect>



#[cfg(feature = "glam")]
use crate::CacheAlignedArray;
use crate::{
    Axis, canonical::CanonicalGroup, direction::Direction, flip::Flip, orient_table,
    orientation_enum::Orient, pack_flip_and_rotation, polarity::Pol, rotation::Rotation,
    wrap_angle,
};
use byteset::ByteSet;
use lolevel::{cache_padded::CachePadded};
use paste::paste;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeconstructedOrientation {
    pub flip_x: bool,
    pub flip_y: bool,
    pub flip_z: bool,
    pub angle: u8,
    pub up: Direction,
}

impl DeconstructedOrientation {
    #[must_use]
    #[inline]
    pub const fn construct(self) -> Orientation {
        Orientation::new(
            Rotation::new(self.up, self.angle as i32),
            Flip::new(self.flip_x, self.flip_y, self.flip_z),
        )
    }
}

// ::::: of niche optimization.
// Field     : Bit Range
// Flip      : 0..3
//      X    : 0
//      Y    : 1
//      Z    : 2
// Rotation  : 3..8
//      angle: 3..5
//      up   : 5..8
#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Orientation(pub(crate) Orient);
const _: () = isit::assert_u8_niche::<Orientation>();
const _: () = isit::assert_u8_niche::<Option<Orientation>>();
const _: () = isit::assert_u8_niche::<Option<Option<Orientation>>>();
const _: () = isit::assert_u8_niche::<Option<Option<Option<Orientation>>>>();

macro_rules! map_coord_impls {
    ($(
        $type:ty
    ),*$(,)?) => {
        $(
            paste!{
                /// This method can tell you where on the target face a source UV is.
                /// To get the most benefit out of this, it is advised that you center your coords around (0, 0).
                /// So if you're trying to map a coord within a rect of size (16, 16), you would subtract 8 from the
                /// x and y of the coord, then pass that offset coord to this function, then add 8 back to the x and y
                /// to get your final coord.
                #[must_use]
                #[inline]
                pub const fn [<map_face_coord_ $type>](self, face: Direction, uv: ($type, $type)) -> ($type, $type) {
                    let coordmap = orient_table::MAP_FACE_COORD_TABLE.get(self.rotation(), self.flip(), face);
                    coordmap.[<map_ $type>](uv)
                }

                /// This method can tell you where on the source face a target UV is.
                /// To get the most benefit out of this, it is advised that you center your coords around (0, 0).
                /// So if you're trying to map a coord within a rect of size (16, 16), you would subtract 8 from the
                /// x and y of the coord, then pass that offset coord to this function, then add 8 back to the x and y
                /// to get your final coord.
                #[must_use]
                #[inline]
                pub const fn [<source_face_coord_ $type>](self, face: Direction, uv: ($type, $type)) -> ($type, $type) {
                    let coordmap = orient_table::SOURCE_FACE_COORD_TABLE.get(self.rotation(), self.flip(), face);
                    coordmap.[<map_ $type>](uv)
                }
            }
        )*
    };
}

macro_rules! transform_impls {
    ($(
        $type:ty
    ),*$(,)?) => {
        $(
            paste!{
                #[must_use]
                #[inline]
                pub const fn [<transform_ $type>](self, point: ($type, $type, $type)) -> ($type, $type, $type) {
                    let rotated = self.rotation().[<rotate_coord_ $type>](point);
                    self.flip().[<flip_coord_ $type>](rotated)
                }
            }
        )*
    };
}

macro_rules! orient_cycle_calc_body {
    ($lhs:ident, $rhs: ident, $function:ident) => {{
        let mut dest = [Orientation::IDENTITY; 4];
        let mut cycler = $lhs;
        let mut count = 0u8;
        // TODO: I'm pretty sure this is wrong.
        while count < 4 {
            dest[count as usize] = cycler;
            cycler = cycler.$function($rhs);
            if cycler.eq($lhs) {
                break;
            }
            count += 1;
        }
        (count, dest)
    }};
}

macro_rules! orient_cycle_body {
    ($function:ident($lhs:ident, $rhs:ident, $cycle:ident)) => {{
        if $rhs.eq(Orientation::IDENTITY) {
            return $lhs;
        }
        let (cycle_count, orientations) = $lhs.$function($rhs);
        let cycle = $cycle.rem_euclid(cycle_count as i32);
        orientations[cycle as usize]
    }};
}

macro_rules! canonical_table {
    ($table:ident[$function:ident]) => {{
        const fn canonicalize_slow(orient: Orientation) -> Orientation {
            let new_orient = Orientation::$table[orient.$function() as usize].reorient(orient);
            if !new_orient.is_equivalent(orient) {
                panic!("Not equivalent!");
            }
            new_orient
        }
        let mut table = CachePadded::new([Orientation::IDENTITY; 192]);
        let mut orient_i = 0u8;
        while orient_i < 192 {
            let orient = unsafe { Orientation::from_u8_unchecked(orient_i) };
            table.value[orient_i as usize] = canonicalize_slow(orient);
            orient_i += 1;
        }
        table
    }};
}

impl Orientation {
    pub(crate) const TOTAL_ORIENTATION_COUNT: u8 = /* Flip */ 8 * /* Angle */ 4 * /* Up */ 6;
    pub(crate) const ORIENTATION_MAX: u8 = Self::TOTAL_ORIENTATION_COUNT - 1;
    pub const IDENTITY: Orientation = Orientation::new(Rotation::new(Direction::PosY, 0), Flip::NONE);
    pub const MIN: Self = Self::IDENTITY;
    pub const MAX: Self = unsafe { Self::from_u8_unchecked(Self::ORIENTATION_MAX) };
    pub const ROTATE_X: Orientation = Rotation::ROTATE_X.orientation();
    pub const ROTATE_Y: Orientation = Rotation::ROTATE_Y.orientation();
    pub const ROTATE_Z: Orientation = Rotation::ROTATE_Z.orientation();

    ////////////////////////////////////////////////////////////////
    //                       Lookup Tables                        //
    ////////////////////////////////////////////////////////////////

    pub const X_ROTATIONS: [Orientation; 4] = Self::ROTATE_X.angles();
    pub const Y_ROTATIONS: [Orientation; 4] = Self::ROTATE_Y.angles();
    pub const Z_ROTATIONS: [Orientation; 4] = Self::ROTATE_Z.angles();

    pub const CANONICAL_X_GROUPS: [Orientation; 4] = [
        Orientation::IDENTITY,
        Orientation::new(Rotation::new(Direction::NegY, 2), Flip::XY),
        Orientation::new(Rotation::new(Direction::PosY, 2), Flip::XZ),
        Orientation::new(Rotation::new(Direction::NegY, 0), Flip::YZ),
    ];

    pub const CANONICAL_Y_GROUPS: [Orientation; 4] = [
        Orientation::IDENTITY,
        Orientation::new(Rotation::new(Direction::NegY, 2), Flip::XY),
        Orientation::new(Rotation::new(Direction::NegY, 0), Flip::YZ),
        Orientation::new(Rotation::new(Direction::PosY, 2), Flip::XZ),
    ];

    pub const CANONICAL_Z_GROUPS: [Orientation; 4] = [
        Orientation::IDENTITY,
        Orientation::new(Rotation::new(Direction::PosY, 2), Flip::XZ),
        Orientation::new(Rotation::new(Direction::NegY, 0), Flip::YZ),
        Orientation::new(Rotation::new(Direction::NegY, 2), Flip::XY),
    ];

    const INVERT_TABLE: CachePadded<[Self; 192]> = {
        let mut table = CachePadded::new([Self::IDENTITY; 192]);
        let mut orient_int = 0u8;
        while orient_int < 192 {
            let orientation = unsafe { Self::from_u8_unchecked(orient_int) };
            table.value[orient_int as usize] = Self::IDENTITY.deorient(orientation);
            orient_int += 1;
        }
        table
    };

    /// Although it is more logical to cycle through [Rotation] before
    /// [Flip], it is more logical to have [Flip] in the lower bits of
    /// [Orientation] since [Rotation] max value is not a power of two.
    ///
    /// This array is ordered by [Rotation] first, then [Flip], allowing
    /// for cycling through rotations before flips.
    pub const ROTATION_ORDERED: CachePadded<[Self; 192]> = {
        let mut order = CachePadded::new([Self::IDENTITY; 192]);
        let mut flip = 0u8;
        let mut rotation = 0u8;
        let mut index = 0;
        loop {
            let orient_bits = flip | (rotation << 3);
            let orient = Orientation(unsafe { Orient::from_u8_unchecked(orient_bits) });
            order.value[index] = orient;
            index += 1;
            if rotation == 23 {
                if flip == 7 {
                    break;
                }
                rotation = 0;
                flip += 1;
            } else {
                rotation += 1;
            }
        }
        order
    };

    // verified (2025-12-29)
    pub const CORNER_ORIENTATIONS_MATRIX: [[[[Orientation; 3]; 2]; 2]; 2] = [
        [
            [
                Rotation::new(Direction::PosX, 2)
                    .orientation()
                    .corner_angles(),
                Rotation::new(Direction::PosZ, 3)
                    .orientation()
                    .corner_angles(),
            ],
            [
                Rotation::new(Direction::NegZ, 1)
                    .orientation()
                    .corner_angles(),
                Rotation::new(Direction::NegX, 0)
                    .orientation()
                    .corner_angles(),
            ],
        ],
        [
            [
                Rotation::new(Direction::NegZ, 3)
                    .orientation()
                    .corner_angles(),
                Rotation::new(Direction::PosX, 0)
                    .orientation()
                    .corner_angles(),
            ],
            [
                Rotation::new(Direction::NegX, 2)
                    .orientation()
                    .corner_angles(),
                Rotation::new(Direction::PosZ, 1)
                    .orientation()
                    .corner_angles(),
            ],
        ],
    ];

    // verified (2025-12-29)
    // Ordered by Direction rotation discriminant (`PosY`, `PosX`, `PosZ`, `NegY`, `NegX`, `NegZ`)
    pub const FACE_ORIENTATIONS: [[Orientation; 4]; 6] = [
        Self::Y_ROTATIONS,                // PosY
        Self::X_ROTATIONS,                // PosX
        Self::Z_ROTATIONS,                // PosZ
        Self::ROTATE_Y.invert().angles(), // NegY
        Self::ROTATE_X.invert().angles(), // NegX
        Self::ROTATE_Z.invert().angles(), // NegZ
    ];

    /// An orientation that you can orient an orientation by to rotate around a face by angle. That was a mouthful.
    /// If angle == 0, orientation is default orientation.
    #[must_use]
    #[inline]
    pub const fn face_orientation(face: Direction, angle: i32) -> Self {
        Self::FACE_ORIENTATIONS[face.rotation_discriminant() as usize][wrap_angle(angle) as usize]
    }

    // `n <= 0` == `-N`, `n > 0` == `+N`
    #[must_use]
    #[inline]
    pub const fn corner_orientation(x: Pol, y: Pol, z: Pol, angle: i32) -> Orientation {
        let angle = angle.rem_euclid(3) as usize;
        Self::CORNER_ORIENTATIONS_MATRIX[y as usize][z as usize][x as usize][angle]
    }

    #[must_use]
    #[inline(always)]
    pub const fn new(rotation: Rotation, flip: Flip) -> Self {
        Self(unsafe { Orient::from_u8_unchecked(pack_flip_and_rotation(flip, rotation)) })
    }

    #[must_use]
    #[inline(always)]
    pub const unsafe fn from_u8_unchecked(value: u8) -> Self {
        Self(unsafe { Orient::from_u8_unchecked(value) })
    }

    #[must_use]
    #[inline(always)]
    pub const fn from_u8_wrapping(value: u8) -> Self {
        Self(Orient::from_u8_wrapping(value))
    }

    #[must_use]
    #[inline(always)]
    pub const fn from_u8(value: u8) -> Option<Self> {
        if value > Self::MAX.0 as u8 {
            return None;
        }
        // SAFETY: guard clause ensures that u8 is not invalid.
        Some(unsafe { Self::from_u8_unchecked(value) })
    }

    #[must_use]
    #[inline(always)]
    pub const fn as_u8(self) -> u8 {
        self.0 as u8
    }

    #[must_use]
    #[inline]
    pub const fn deconstruct(self) -> DeconstructedOrientation {
        DeconstructedOrientation {
            flip_x: self.flip().x(),
            flip_y: self.flip().y(),
            flip_z: self.flip().z(),
            angle: self.rotation().angle() as u8,
            up: self.rotation().up(),
        }
    }

    transform_impls!(i8, i16, i32, i64, i128, isize, f32, f64,);

    map_coord_impls!(i8, i16, i32, i64, i128, isize, f32, f64,);

    #[must_use]
    #[inline(always)]
    pub const fn canonical_x_orientation(group: CanonicalGroup) -> Orientation {
        Self::CANONICAL_X_GROUPS[group as usize]
    }

    #[must_use]
    #[inline(always)]
    pub const fn canonical_y_orientation(group: CanonicalGroup) -> Orientation {
        Self::CANONICAL_Y_GROUPS[group as usize]
    }

    #[must_use]
    #[inline(always)]
    pub const fn canonical_z_orientation(group: CanonicalGroup) -> Orientation {
        Self::CANONICAL_Z_GROUPS[group as usize]
    }

    #[must_use]
    pub const fn canonical_axis_orientation(axis: Axis, group: CanonicalGroup) -> Orientation {
        match axis {
            Axis::X => Self::canonical_x_orientation(group),
            Axis::Y => Self::canonical_y_orientation(group),
            Axis::Z => Self::canonical_z_orientation(group),
        }
    }

    #[must_use]
    #[inline(always)]
    pub const fn canonical_group_x(self) -> CanonicalGroup {
        self.flip().canonical_group_x()
    }

    #[must_use]
    #[inline(always)]
    pub const fn canonical_group_y(self) -> CanonicalGroup {
        self.flip().canonical_group_y()
    }

    #[must_use]
    #[inline(always)]
    pub const fn canonical_group_z(self) -> CanonicalGroup {
        self.flip().canonical_group_z()
    }

    #[must_use]
    #[inline(always)]
    pub const fn canonical_axis_group(self, axis: Axis) -> CanonicalGroup {
        self.flip().canonical_group(axis)
    }

    #[must_use]
    #[inline]
    pub const fn in_canonical_group_x(self, group: CanonicalGroup) -> Self {
        let self_canon = self.canonicalize_x();
        let group_canon = Self::canonical_x_orientation(group);
        self_canon.deorient(group_canon)
    }

    #[must_use]
    #[inline]
    pub const fn in_canonical_group_y(self, group: CanonicalGroup) -> Self {
        let self_canon = self.canonicalize_y();
        let group_canon = Self::canonical_y_orientation(group);
        self_canon.deorient(group_canon)
    }

    #[must_use]
    #[inline]
    pub const fn in_canonical_group_z(self, group: CanonicalGroup) -> Self {
        let self_canon = self.canonicalize_z();
        let group_canon = Self::canonical_z_orientation(group);
        self_canon.deorient(group_canon)
    }

    #[must_use]
    #[inline]
    pub const fn in_canonical_axis_group(self, axis: Axis, group: CanonicalGroup) -> Self {
        match axis {
            Axis::X => self.in_canonical_group_x(group),
            Axis::Y => self.in_canonical_group_y(group),
            Axis::Z => self.in_canonical_group_z(group),
        }
    }
    #[must_use]
    #[inline(always)]
    pub const fn set_canonical_group_x(&mut self, group: CanonicalGroup) {
        *self = self.in_canonical_group_x(group);
    }

    #[inline(always)]
    pub const fn set_canonical_group_y(&mut self, group: CanonicalGroup) {
        *self = self.in_canonical_group_y(group);
    }

    #[inline(always)]
    pub const fn set_canonical_group_z(&mut self, group: CanonicalGroup) {
        *self = self.in_canonical_group_z(group);
    }

    #[inline(always)]
    pub const fn set_canonical_axis_group(&mut self, axis: Axis, group: CanonicalGroup) {
        *self = self.in_canonical_axis_group(axis, group)
    }

    #[must_use]
    #[inline(always)]
    pub const fn is_canonical_x_group(self, group: CanonicalGroup) -> bool {
        self.canonical_group_x().eq(group)
    }

    #[must_use]
    #[inline(always)]
    pub const fn is_canonical_y_group(self, group: CanonicalGroup) -> bool {
        self.canonical_group_y().eq(group)
    }

    #[must_use]
    pub const fn is_canonical_z_group(self, group: CanonicalGroup) -> bool {
        self.canonical_group_z().eq(group)
    }

    #[must_use]
    #[inline(always)]
    pub const fn is_canonical_axis_group(self, axis: Axis, group: CanonicalGroup) -> bool {
        self.canonical_axis_group(axis).eq(group)
    }

    #[must_use]
    #[inline(always)]
    pub const fn is_canonical_x(self) -> bool {
        matches!(self.canonical_group_x(), CanonicalGroup::Group0)
    }

    #[must_use]
    #[inline(always)]
    pub const fn is_canonical_y(self) -> bool {
        matches!(self.canonical_group_y(), CanonicalGroup::Group0)
    }

    #[must_use]
    #[inline(always)]
    pub const fn is_canonical_z(self) -> bool {
        matches!(self.canonical_group_z(), CanonicalGroup::Group0)
    }

    #[must_use]
    #[inline(always)]
    pub const fn is_canonical_axis(self, axis: Axis) -> bool {
        match axis {
            Axis::X => self.is_canonical_x(),
            Axis::Y => self.is_canonical_y(),
            Axis::Z => self.is_canonical_z(),
        }
    }

    /// Checks if any axis is canonical.
    #[must_use]
    #[inline(always)]
    pub const fn is_canonical(self) -> bool {
        const TABLE: ByteSet = {
            let mut set = ByteSet::new();
            let mut i = 0;
            while i < 192 {
                let orient = unsafe { Orientation::from_u8_unchecked(i) };
                if orient.is_canonical_x() || orient.is_canonical_y() || orient.is_canonical_z() {
                    set.add_byte(i);
                }
                i += 1;
            }
            set
        };
        TABLE.has(self.as_u8())
    }

    #[must_use]
    #[inline(always)]
    pub const fn canonicalize_x(self) -> Self {
        const CANONICAL_TABLE: CachePadded<[Orientation; 192]> =
            canonical_table!(CANONICAL_X_GROUPS[canonical_group_x]);
        CANONICAL_TABLE.value[self.0 as usize]
    }

    #[must_use]
    #[inline(always)]
    pub const fn canonicalize_y(self) -> Self {
        const CANONICAL_TABLE: CachePadded<[Orientation; 192]> =
            canonical_table!(CANONICAL_Y_GROUPS[canonical_group_y]);
        CANONICAL_TABLE.value[self.0 as usize]
    }

    #[must_use]
    #[inline(always)]
    pub const fn canonicalize_z(self) -> Self {
        const CANONICAL_TABLE: CachePadded<[Orientation; 192]> =
            canonical_table!(CANONICAL_Z_GROUPS[canonical_group_z]);
        CANONICAL_TABLE.value[self.0 as usize]
    }

    #[must_use]
    #[inline]
    pub const fn canonicalize_axis(self, axis: Axis) -> Self {
        match axis {
            Axis::X => self.canonicalize_x(),
            Axis::Y => self.canonicalize_y(),
            Axis::Z => self.canonicalize_z(),
        }
    }

    // verified (2025-12-28)
    /// A helper function to create 4 orientations for an orientation group.
    /// An orientation group is a series of "contiguous" orientations. That is, the orientations are logically sequential.
    /// An example would be rotations around an axis, or around a face, where there are 4 orientations possible.
    /// The first orientation is unoriented, the second orientation is the target orientation
    /// applied once, the third orientation is the target orientation applied twice,
    /// and the fourth orientation is the target orientation applied three times.
    #[must_use]
    pub const fn angles(self) -> [Orientation; 4] {
        let angle2 = self.reorient(self);
        let angle3 = angle2.reorient(self);
        [Orientation::IDENTITY, self, angle2, angle3]
    }

    // verified (2025-12-28)
    /// A helper function to create 3 orientations for a corner orientation group.
    /// The first orientation is unoriented, the second orientation is the target orientation,
    /// and the third orientation is the target orientation applied to itself.
    #[must_use]
    pub const fn corner_angles(self) -> [Orientation; 3] {
        let angle2 = self.reorient(self);
        [Orientation::IDENTITY, self, angle2]
    }

    #[must_use]
    #[inline(always)]
    pub const fn flip(self) -> Flip {
        unsafe { Flip::from_u8_unchecked(self.0 as u8 & 0b111) }
    }

    #[must_use]
    #[inline(always)]
    pub const fn flipped(self, flip: Flip) -> Self {
        Self(unsafe { Orient::from_u8_unchecked(self.0 as u8 ^ flip.0 as u8) })
    }

    #[must_use]
    #[inline(always)]
    pub const fn rotation(self) -> Rotation {
        // SAFETY: Here, we assume that `self` is a valid Orientation.
        unsafe { Rotation::from_u8_unchecked(self.0 as u8 >> 3) }
    }

    #[inline]
    pub const fn set_flip(&mut self, flip: Flip) {
        self.0 = unsafe { Orient::from_u8_unchecked((self.0 as u8 & 0b11111000) | flip.0 as u8) };
    }

    #[inline]
    pub const fn reset_flip(&mut self) {
        self.0 = unsafe { Orient::from_u8_unchecked(self.0 as u8 & 0b11111000) };
    }

    pub const fn set_flip_x(&mut self, x: bool) {
        let mut flip = self.flip();
        flip.set_x(x);
        self.set_flip(flip);
    }

    pub const fn set_flip_y(&mut self, y: bool) {
        let mut flip = self.flip();
        flip.set_y(y);
        self.set_flip(flip);
    }

    pub const fn set_flip_z(&mut self, z: bool) {
        let mut flip = self.flip();
        flip.set_z(z);
        self.set_flip(flip);
    }

    pub const fn set_flip_xy(&mut self, xy: bool) {
        let mut flip = self.flip();
        flip.set_xy(xy);
        self.set_flip(flip);
    }

    pub const fn set_flip_xz(&mut self, xz: bool) {
        let mut flip = self.flip();
        flip.set_xz(xz);
        self.set_flip(flip);
    }

    pub const fn set_flip_yz(&mut self, yz: bool) {
        let mut flip = self.flip();
        flip.set_yz(yz);
        self.set_flip(flip);
    }

    pub const fn set_flip_xyz(&mut self, xyz: bool) {
        let mut flip = self.flip();
        flip.set_xyz(xyz);
        self.set_flip(flip);
    }

    #[inline]
    pub const fn set_rotation(&mut self, rotation: Rotation) {
        self.0 = unsafe {
            Orient::from_u8_unchecked((self.0 as u8 & 0b111) | ((rotation.0 as u8) << 3))
        };
    }

    #[inline]
    pub const fn reset_rotation(&mut self) {
        self.0 = unsafe { Orient::from_u8_unchecked(self.0 as u8 & 0b111) };
    }

    #[inline]
    pub fn set_up(&mut self, up: Direction) {
        let mut rot = self.rotation();
        rot.set_up(up);
        self.set_rotation(rot);
    }

    #[inline]
    pub fn set_angle(&mut self, angle: i32) {
        let mut rot = self.rotation();
        rot.set_angle(angle);
        self.set_rotation(rot);
    }

    /// Cycle through each [Orientation].
    ///
    /// This will cycle through the [Flip] first.
    /// There are other cycling options, such as cycling through
    /// the rotations first, then the flips, or cycling through
    /// only the rotations.
    #[must_use]
    #[inline]
    pub const fn cycle(self, offset: i32) -> Self {
        // Here, we assume that `self` has a valid bit representation.
        Self(unsafe {
            Orient::from_u8_unchecked(
                (self.0 as i64 + offset as i64).rem_euclid(Self::TOTAL_ORIENTATION_COUNT as i64)
                    as u8,
            )
        })
    }

    #[must_use]
    #[inline]
    pub const fn cycle_rotation_first(self, offset: i32) -> Self {
        let index = self.flip().0 as i64 * 24 + self.rotation().0 as i64;
        let offset_index =
            (index + offset as i64).rem_euclid(Self::TOTAL_ORIENTATION_COUNT as i64) as usize;
        Self::ROTATION_ORDERED.value[offset_index]
    }

    /// Keeps the [Flip], but cycles through [Rotation].
    #[must_use]
    #[inline]
    pub const fn cycle_rotation(self, offset: i32) -> Self {
        Self::new(self.rotation().cycle(offset), self.flip())
    }

    #[must_use]
    pub const fn count_reorient_cycle_calc(self, orientation: Self) -> (u8, [Orientation; 4]) {
        orient_cycle_calc_body!(self, orientation, reorient)
    }

    #[must_use]
    pub const fn count_reorient_local_cycle_calc(
        self,
        orientation: Self,
    ) -> (u8, [Orientation; 4]) {
        orient_cycle_calc_body!(self, orientation, reorient_local)
    }

    #[must_use]
    pub const fn count_deorient_cycle_calc(self, orientation: Self) -> (u8, [Orientation; 4]) {
        orient_cycle_calc_body!(self, orientation, deorient)
    }

    #[must_use]
    pub const fn count_deorient_local_cycle_calc(
        self,
        orientation: Self,
    ) -> (u8, [Orientation; 4]) {
        orient_cycle_calc_body!(self, orientation, deorient_local)
    }

    #[must_use]
    #[inline(always)]
    pub const fn count_reorient_cycle(self, orientation: Self) -> u8 {
        self.count_reorient_cycle_calc(orientation).0
    }

    #[must_use]
    #[inline(always)]
    pub const fn count_reorient_local_cycle(self, orientation: Self) -> u8 {
        self.count_reorient_local_cycle_calc(orientation).0
    }

    #[must_use]
    #[inline(always)]
    pub const fn count_deorient_cycle(self, orientation: Self) -> u8 {
        self.count_deorient_cycle_calc(orientation).0
    }

    #[must_use]
    #[inline(always)]
    pub const fn count_deorient_local_cycle(self, orientation: Self) -> u8 {
        self.count_deorient_local_cycle_calc(orientation).0
    }

    #[must_use]
    pub const fn reorient_cycle(self, orientation: Self, cycle: i32) -> Self {
        orient_cycle_body!(count_reorient_cycle_calc(self, orientation, cycle))
    }

    #[must_use]
    pub const fn reorient_local_cycle(self, orientation: Self, cycle: i32) -> Self {
        orient_cycle_body!(count_reorient_local_cycle_calc(self, orientation, cycle))
    }

    #[must_use]
    pub const fn deorient_cycle(self, orientation: Self, cycle: i32) -> Self {
        orient_cycle_body!(count_deorient_cycle_calc(self, orientation, cycle))
    }

    #[must_use]
    pub const fn deorient_local_cycle(self, orientation: Self, cycle: i32) -> Self {
        orient_cycle_body!(count_deorient_local_cycle_calc(self, orientation, cycle))
    }

    // TODO: Maybe use a custom iterator type so that this can be made const?
    /// This will cycle through the 8 [Flip] states before cycling the 24 [Rotation] states.
    ///
    /// If you would like a version that cycles the rotations before cycling the flips, use [Orientation::iter_rotation_order].
    #[must_use]
    #[inline]
    pub fn iter() -> impl Iterator<Item = Self> {
        (0..Self::TOTAL_ORIENTATION_COUNT)
            .map(move |i| Self(unsafe { Orient::from_u8_unchecked(i) }))
    }

    #[must_use]
    #[inline(always)]
    pub const fn cartesian_product<const PRODUCTS: usize>() -> CartesianOrientationIter<PRODUCTS> {
        CartesianOrientationIter::new()
    }

    /// Cycle through the 24 [Rotation] states before cycling through the 8 [Flip] states.
    #[must_use]
    #[inline]
    pub const fn iter_rotation_order() -> RotationFirstOrientationIterator {
        RotationFirstOrientationIterator::START
    }

    #[must_use]
    #[inline(always)]
    pub fn iter_canonical_x() -> CanonicalIter {
        CanonicalIter::new(Axis::X)
    }

    #[must_use]
    #[inline(always)]
    pub fn iter_canonical_y() -> CanonicalIter {
        CanonicalIter::new(Axis::Y)
    }

    #[must_use]
    #[inline(always)]
    pub fn iter_canonical_z() -> CanonicalIter {
        CanonicalIter::new(Axis::Z)
    }

    #[must_use]
    #[inline(always)]
    pub fn iter_canonical_axis(axis: Axis) -> CanonicalIter {
        CanonicalIter::new(axis)
    }

    // verified (2025-12-30)
    /// `reface` can be used to determine where a face will end up after orientation.
    /// First rotates and then flips the face.
    #[must_use]
    #[inline]
    pub const fn reface(self, face: Direction) -> Direction {
        let rotated = self.rotation().reface(face);
        rotated.flip(self.flip())
    }

    // verified (2025-12-30)
    /// This determines which face was oriented to `face`.
    #[must_use]
    #[inline]
    pub const fn source_face(self, face: Direction) -> Direction {
        let flipped = face.flip(self.flip());
        self.rotation().source_face(flipped)
    }

    // verified (2025-12-30)
    /// Gets the direction that [Direction::PosY] is pointing towards.
    #[must_use]
    #[inline]
    pub const fn up(self) -> Direction {
        self.reface(Direction::PosY)
    }

    // verified (2025-12-30)
    /// Gets the direction that [Direction::NegY] is pointing towards.
    #[must_use]
    #[inline]
    pub const fn down(self) -> Direction {
        self.reface(Direction::NegY)
    }

    // verified (2025-12-30)
    /// Gets the direction that [Direction::NegZ] is pointing towards.
    #[must_use]
    #[inline]
    pub const fn forward(self) -> Direction {
        self.reface(Direction::NegZ)
    }

    // verified (2025-12-30)
    /// Gets the direction that [Direction::PosZ] is pointing towards.
    #[must_use]
    #[inline]
    pub const fn backward(self) -> Direction {
        self.reface(Direction::PosZ)
    }

    // verified (2025-12-30)
    /// Gets the direction that [Direction::NegX] is pointing towards.
    #[must_use]
    #[inline]
    pub const fn left(self) -> Direction {
        self.reface(Direction::NegX)
    }

    // verified (2025-12-30)
    /// Gets the direction that [Direction::PosX] is pointing towards.
    #[must_use]
    #[inline]
    pub const fn right(self) -> Direction {
        self.reface(Direction::PosX)
    }

    /// Determines which orientation must be applied to `self` to get `orientation`.
    pub const fn difference(self, orientation: Self) -> Self {
        self.invert().reorient(orientation)
    }

    /// Reorient `self` with `orientation`.
    #[must_use]
    pub const fn reorient(self, orientation: Self) -> Self {
        let up = self.up();
        let fwd = self.forward();
        let reup = orientation.reface(up);
        let refwd = orientation.reface(fwd);
        let flip = self.flip().flip(orientation.flip());
        let flipup = reup.flip(flip);
        let flipfwd = refwd.flip(flip);
        let Some(rot) = Rotation::from_up_and_forward(flipup, flipfwd) else {
            // SAFETY: This is guaranteed to never be reached, as all of the logic has been verified.
            unsafe { ::core::hint::unreachable_unchecked() }
        };
        Orientation::new(rot, flip)
    }

    #[must_use]
    #[inline(always)]
    pub const fn reorient_local(self, orientation: Self) -> Self {
        orientation.reorient(self)
    }

    #[must_use]
    #[inline]
    pub const fn reorient_canonical_x(self, orientation: Self) -> Self {
        let self_canon = self.canonicalize_x();
        let orientation = orientation.canonicalize_x();
        self_canon.reorient(orientation)
    }

    #[must_use]
    #[inline]
    pub const fn reorient_canonical_x_local(self, orientation: Self) -> Self {
        let self_canon = self.canonicalize_x();
        let orientation = orientation.canonicalize_x();
        orientation.reorient(self_canon)
    }

    #[must_use]
    #[inline]
    pub const fn reorient_canonical_y(self, orientation: Self) -> Self {
        let self_canon = self.canonicalize_y();
        let orientation = orientation.canonicalize_y();
        self_canon.reorient(orientation)
    }

    #[must_use]
    #[inline]
    pub const fn reorient_canonical_y_local(self, orientation: Self) -> Self {
        let self_canon = self.canonicalize_y();
        let orientation = orientation.canonicalize_y();
        orientation.reorient(self_canon)
    }

    #[must_use]
    #[inline]
    pub const fn reorient_canonical_z(self, orientation: Self) -> Self {
        let self_canon = self.canonicalize_z();
        let orientation = orientation.canonicalize_z();
        self_canon.reorient(orientation)
    }

    #[must_use]
    #[inline]
    pub const fn reorient_canonical_z_local(self, orientation: Self) -> Self {
        let self_canon = self.canonicalize_z();
        let orientation = orientation.canonicalize_z();
        orientation.reorient(self_canon)
    }

    #[must_use]
    pub const fn reorient_canonical_axis(self, axis: Axis, orientation: Self) -> Self {
        match axis {
            Axis::X => self.reorient_canonical_x(orientation),
            Axis::Y => self.reorient_canonical_y(orientation),
            Axis::Z => self.reorient_canonical_z(orientation),
        }
    }

    #[must_use]
    pub const fn reorient_canonical_axis_local(self, axis: Axis, orientation: Self) -> Self {
        match axis {
            Axis::X => self.reorient_canonical_x_local(orientation),
            Axis::Y => self.reorient_canonical_y_local(orientation),
            Axis::Z => self.reorient_canonical_z_local(orientation),
        }
    }

    /// Remove an orientation from an orientation.
    /// This is the inverse operation to [Orientation::reorient].
    #[must_use]
    pub const fn deorient(self, orientation: Self) -> Self {
        let up = self.up();
        let fwd = self.forward();
        let reup = orientation.source_face(up);
        let refwd = orientation.source_face(fwd);
        let flip = self.flip().flip(orientation.flip());
        let flipup = reup.flip(flip);
        let flipfwd = refwd.flip(flip);
        let Some(rot) = Rotation::from_up_and_forward(flipup, flipfwd) else {
            // SAFETY: This is guaranteed to never be reached, as all of the logic has been verified.
            unsafe { ::core::hint::unreachable_unchecked() }
        };
        Orientation::new(rot, flip)
    }

    #[must_use]
    #[inline]
    pub const fn deorient_local(self, orientation: Self) -> Self {
        orientation.invert().reorient(self)
    }

    #[must_use]
    #[inline]
    pub const fn deorient_canonical_x(self, orientation: Self) -> Self {
        let self_canon = self.canonicalize_x();
        let orientation = orientation.canonicalize_x();
        self_canon.deorient(orientation)
    }

    #[must_use]
    #[inline]
    pub const fn deorient_canonical_x_local(self, orientation: Self) -> Self {
        let self_canon = self.canonicalize_x();
        let orientation = orientation.canonicalize_x();
        self_canon.deorient_local(orientation)
    }

    #[must_use]
    #[inline]
    pub const fn deorient_canonical_y(self, orientation: Self) -> Self {
        let self_canon = self.canonicalize_y();
        let orientation = orientation.canonicalize_y();
        self_canon.deorient(orientation)
    }

    #[must_use]
    #[inline]
    pub const fn deorient_canonical_y_local(self, orientation: Self) -> Self {
        let self_canon = self.canonicalize_y();
        let orientation = orientation.canonicalize_y();
        self_canon.deorient_local(orientation)
    }

    #[must_use]
    #[inline]
    pub const fn deorient_canonical_z(self, orientation: Self) -> Self {
        let self_canon = self.canonicalize_z();
        let orientation = orientation.canonicalize_z();
        self_canon.deorient(orientation)
    }

    #[must_use]
    #[inline]
    pub const fn deorient_canonical_z_local(self, orientation: Self) -> Self {
        let self_canon = self.canonicalize_z();
        let orientation = orientation.canonicalize_z();
        self_canon.deorient_local(orientation)
    }

    #[must_use]
    pub const fn deorient_canonical_axis(self, axis: Axis, orientation: Orientation) -> Self {
        match axis {
            Axis::X => self.deorient_canonical_x(orientation),
            Axis::Y => self.deorient_canonical_y(orientation),
            Axis::Z => self.deorient_canonical_z(orientation),
        }
    }

    #[must_use]
    pub const fn deorient_canonical_axis_local(self, axis: Axis, orientation: Orientation) -> Self {
        match axis {
            Axis::X => self.deorient_canonical_x_local(orientation),
            Axis::Y => self.deorient_canonical_y_local(orientation),
            Axis::Z => self.deorient_canonical_z_local(orientation),
        }
    }

    pub const fn conjugate(self, orientation: Orientation) -> Self {
        self.invert().reorient(orientation).reorient(self)
    }

    /// Returns the orientation that can be applied to deorient by [self].
    #[must_use]
    #[inline]
    pub const fn invert(self) -> Self {
        Self::INVERT_TABLE.value[self.0 as usize]
    }

    /// Flip the [Orientation] along the `X` axis.
    #[must_use]
    #[inline]
    pub const fn flip_x(self) -> Self {
        self.flipped(Flip::X)
    }

    /// Flip the [Orientation] along the `Y` axis.
    #[must_use]
    #[inline]
    pub const fn flip_y(self) -> Self {
        self.flipped(Flip::Y)
    }

    /// Flip the [Orientation] along the `Z` axis.
    #[must_use]
    #[inline]
    pub const fn flip_z(self) -> Self {
        self.flipped(Flip::Z)
    }

    /// Flip the [Orientation] along the `X` and `Y` axes.
    #[must_use]
    #[inline]
    pub const fn flip_xy(self) -> Self {
        self.flipped(Flip::XY)
    }

    /// Flip the [Orientation] along the `X` and `Z` axes.
    #[must_use]
    #[inline]
    pub const fn flip_xz(self) -> Self {
        self.flipped(Flip::XZ)
    }

    /// Flip the [Orientation] along the `Y` and `Z` axes.
    #[must_use]
    #[inline]
    pub const fn flip_yz(self) -> Self {
        self.flipped(Flip::YZ)
    }

    /// Flip the [Orientation] along the `X`, `Y`, and `Z` axes.
    #[must_use]
    #[inline]
    pub const fn flip_xyz(self) -> Self {
        self.flipped(Flip::XYZ)
    }

    #[must_use]
    #[inline]
    pub const fn rotate_x(self, angle: i32) -> Self {
        self.reorient(Orientation::X_ROTATIONS[wrap_angle(angle) as usize])
    }

    #[must_use]
    #[inline]
    pub const fn rotate_local_x(self, angle: i32) -> Self {
        self.reorient_local(Orientation::X_ROTATIONS[wrap_angle(angle) as usize])
    }

    #[must_use]
    #[inline]
    pub const fn rotate_y(self, angle: i32) -> Self {
        self.reorient(Orientation::Y_ROTATIONS[wrap_angle(angle) as usize])
    }

    #[must_use]
    #[inline]
    pub const fn rotate_local_y(self, angle: i32) -> Self {
        self.reorient_local(Orientation::Y_ROTATIONS[wrap_angle(angle) as usize])
    }

    #[must_use]
    #[inline]
    pub const fn rotate_z(self, angle: i32) -> Self {
        self.reorient(Orientation::Z_ROTATIONS[wrap_angle(angle) as usize])
    }

    #[must_use]
    #[inline]
    pub const fn rotate_local_z(self, angle: i32) -> Self {
        self.reorient_local(Orientation::Z_ROTATIONS[wrap_angle(angle) as usize])
    }

    /// Rotate `face` clockwise by `angle`. Use a negative `angle` to rotate counter-clockwise.
    #[must_use]
    #[inline]
    pub const fn rotate_face(self, face: Direction, angle: i32) -> Self {
        let orient = Self::face_orientation(face, angle);
        self.reorient(orient)
    }

    #[must_use]
    #[inline]
    pub const fn rotate_local_face(self, face: Direction, angle: i32) -> Self {
        let orient = Self::face_orientation(face, angle);
        self.reorient_local(orient)
    }

    #[must_use]
    #[inline]
    pub const fn rotate_corner(self, x: Pol, y: Pol, z: Pol, angle: i32) -> Self {
        let orient = Self::corner_orientation(x, y, z, angle);
        self.reorient(orient)
    }

    #[must_use]
    #[inline]
    pub const fn rotate_local_corner(self, x: Pol, y: Pol, z: Pol, angle: i32) -> Self {
        let orient = Self::corner_orientation(x, y, z, angle);
        self.reorient_local(orient)
    }

    #[cfg(feature = "glam")]
    #[inline]
    #[must_use]
    pub const fn to_matrix(self) -> glam::Mat4 {
        const MATRICES: CachePadded<[glam::Mat4; 192]> = {
            pub const fn to_matrix(orient: Orientation) -> glam::Mat4 {
                let flip = orient.flip();
                let rotation = orient.rotation();
                let scale = flip.to_scale_vec3();
                let up = rotation.reface(Direction::PosY).to_vec3();
                let forward = rotation.reface(Direction::PosZ).to_vec3();
                let right = rotation.reface(Direction::PosX).to_vec3();
                glam::Mat4::from_cols(
                    glam::Vec4::new(right.x * scale.x, right.y * scale.y, right.z * scale.z, 0.0),
                    glam::Vec4::new(up.x * scale.x, up.y * scale.y, up.z * scale.z, 0.0),
                    glam::Vec4::new(forward.x * scale.x, forward.y * scale.y, forward.z * scale.z, 0.0),
                    glam::Vec4::W,
                )
            }
            let mut matrices = CachePadded {
                value: [glam::Mat4::IDENTITY; 192],
            };
            let mut i = 0u8;
            while i < 192 {
                let orient = unsafe { Orientation::from_u8_unchecked(i) };
                matrices.value[i as usize] = to_matrix(orient);
                i += 1;
            }
            matrices
        };
        MATRICES.value[self.0 as usize]
    }

    #[cfg(feature = "glam")]
    #[inline]
    #[must_use]
    pub fn transform_vec3(self, v: glam::Vec3) -> glam::Vec3 {
        use glam::Vec4Swizzles;
        let mat = self.to_matrix();
        (mat * v.extend(1.0)).xyz()
    }

    #[must_use]
    #[inline(always)]
    pub const fn eq(self, other: Self) -> bool {
        self.0 as u8 == other.0 as u8
    }

    #[must_use]
    #[inline(always)]
    pub const fn ne(self, other: Self) -> bool {
        self.0 as u8 != other.0 as u8
    }

    #[must_use]
    pub const fn is_equivalent(self, other: Self) -> bool {
        self.up() as u8 == other.up() as u8
            && self.forward() as u8 == other.forward() as u8
            && self.right() as u8 == other.right() as u8
    }

    #[must_use]
    #[inline(always)]
    pub const fn is_chiral(self) -> bool {
        !self.flip().is_mesh_inside_out()
    }

    #[must_use]
    #[inline(always)]
    pub const fn display(self, short: bool) -> OrientDisplay {
        if short {
            OrientDisplay::Short(OrientShortDisplay(self))
        } else {
            OrientDisplay::Long(OrientLongDisplay(self))
        }
    }
}

impl std::ops::Neg for Orientation {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self.invert()
    }
}

impl std::ops::Mul<Orientation> for Orientation {
    type Output = Self;

    fn mul(self, rhs: Orientation) -> Self::Output {
        self.reorient(rhs)
    }
}

impl std::ops::Div<Orientation> for Orientation {
    type Output = Self;

    fn div(self, rhs: Orientation) -> Self::Output {
        self.deorient(rhs)
    }
}

impl std::ops::Sub<Orientation> for Orientation {
    type Output = Self;

    fn sub(self, rhs: Orientation) -> Self::Output {
        self.difference(rhs)
    }
}

impl Into<u8> for Orientation {
    #[inline]
    fn into(self) -> u8 {
        self.0 as u8
    }
}

impl From<Rotation> for Orientation {
    #[inline]
    fn from(value: Rotation) -> Self {
        Orientation::new(value, Flip::NONE)
    }
}

impl From<Flip> for Orientation {
    #[inline]
    fn from(value: Flip) -> Self {
        Orientation::new(Rotation::default(), value)
    }
}

#[cfg(feature = "glam")]
impl Into<glam::Mat4> for Orientation {
    fn into(self) -> glam::Mat4 {
        self.to_matrix()
    }
}

impl std::fmt::Display for Orientation {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Orientation({},{})", self.flip(), self.rotation())
    }
}

// verified (2025-12-29)
/// Used to iterate over each [Orientation] in the order where [Rotation] cycles before [Flip].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RotationFirstOrientationIterator {
    flip: u8,
    rotation: u8,
}

impl RotationFirstOrientationIterator {
    pub(crate) const START: Self = Self {
        flip: 0,
        rotation: 0,
    };
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self::START
    }

    #[must_use]
    #[inline]
    pub const fn start_at(orientation: Orientation) -> Self {
        Self {
            flip: orientation.flip().0 as u8,
            rotation: orientation.rotation().0 as u8,
        }
    }

    /// Gets the current element without advancing the iterator.
    #[must_use]
    #[inline]
    pub const fn current(self) -> Option<Orientation> {
        if self.flip >= 8 {
            return None;
        }
        Some(Orientation::new(
            unsafe { Rotation::from_u8_unchecked(self.rotation) },
            unsafe { Flip::from_u8_unchecked(self.flip) },
        ))
    }

    #[must_use]
    pub const fn next(&mut self) -> Option<Orientation> {
        if self.flip == 8 {
            return None;
        }
        let result = Some(Orientation::new(
            unsafe { Rotation::from_u8_unchecked(self.rotation) },
            unsafe { Flip::from_u8_unchecked(self.flip) },
        ));
        self.rotation += 1;
        if self.rotation == 24 {
            self.flip += 1;
            self.rotation = 0;
        }
        result
    }
}

impl Iterator for RotationFirstOrientationIterator {
    type Item = Orientation;

    fn size_hint(&self) -> (usize, Option<usize>) {
        const SIZE: usize = Orientation::TOTAL_ORIENTATION_COUNT as usize;
        let total = (self.flip as usize * 24) + self.rotation as usize;
        (SIZE - total, Some(SIZE - total))
    }

    fn next(&mut self) -> Option<Self::Item> {
        if self.flip == 8 {
            return None;
        }
        let result = Some(Orientation::new(
            unsafe { Rotation::from_u8_unchecked(self.rotation) },
            unsafe { Flip::from_u8_unchecked(self.flip) },
        ));
        self.rotation += 1;
        if self.rotation == 24 {
            self.flip += 1;
            self.rotation = 0;
        }
        result
    }
}

#[derive(Clone)]
pub struct CanonicalIter {
    rotation: u8,
    flipped: bool,
    flip: Flip,
}

impl CanonicalIter {
    #[must_use]
    #[inline(always)]
    pub const fn new(flip: Axis) -> Self {
        Self {
            rotation: 0,
            flipped: false,
            flip: flip.as_flip(),
        }
    }

    #[must_use]
    #[inline]
    pub const fn remaining(&self) -> usize {
        (24 - self.rotation as usize) + (24 * !self.flipped as usize)
    }

    #[must_use]
    pub const fn next(&mut self) -> Option<Orientation> {
        if self.flipped && self.rotation >= 24 {
            return None;
        }
        let rotation = unsafe { Rotation::from_u8_unchecked(self.rotation) };
        let flip = [Flip::NONE, self.flip][self.flipped as usize];
        self.rotation += 1;
        if self.rotation >= 24 && !self.flipped {
            self.rotation = 0;
            self.flipped = true;
        }
        Some(Orientation::new(rotation, flip))
    }
}

impl Iterator for CanonicalIter {
    type Item = Orientation;

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remain = self.remaining();
        (remain, Some(remain))
    }

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        CanonicalIter::next(self)
    }
}

#[derive(Clone)]
pub struct CartesianOrientationIter<const PRODUCTS: usize> {
    iterators: [u8; PRODUCTS],
}

impl<const PRODUCTS: usize> CartesianOrientationIter<PRODUCTS> {
    pub const fn new() -> Self {
        Self {
            iterators: [0u8; PRODUCTS],
        }
    }

    #[must_use]
    pub const fn next(&mut self) -> Option<[Orientation; PRODUCTS]> {
        if PRODUCTS == 0 {
            return None;
        }

        if self.iterators[0] >= 192 {
            return None;
        }

        union Transmuter<const PRODUCTS: usize> {
            u8_prods: [u8; PRODUCTS],
            orient_prods: [Orientation; PRODUCTS],
        }
        let result = Some(unsafe {
            Transmuter {
                u8_prods: self.iterators,
            }.orient_prods
        });

        let mut i = PRODUCTS;
        if i != 0 {
            loop {
                i -= 1;
                if i == 0 || self.iterators[i] < 191 {
                    self.iterators[i] += 1;
                    break;
                }
                self.iterators[i] = 0;
            }
        }
        
        result
    }
}

impl<const PRODUCTS: usize> Iterator for CartesianOrientationIter<PRODUCTS> {
    type Item = [Orientation; PRODUCTS];

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct OrientLongDisplay(pub Orientation);

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct OrientShortDisplay(pub Orientation);

#[derive(Debug, Clone, Copy)]
pub enum OrientDisplay {
    Short(OrientShortDisplay),
    Long(OrientLongDisplay),
}

impl std::fmt::Display for OrientLongDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Orientation({rotation}, {flip})",
            rotation = self.0.rotation().display(false),
            flip = self.0.flip().display(false),
        )
    }
}

impl std::fmt::Display for OrientShortDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{rotation}|{flip}",
            rotation = self.0.rotation().display(true),
            flip = self.0.flip().display(true),
        )
    }
}

impl std::fmt::Display for OrientDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrientDisplay::Short(disp) => write!(f, "{disp}"),
            OrientDisplay::Long(disp) => write!(f, "{disp}"),
        }
    }
}

#[allow(unused)]
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    #[ignore]
    fn sandbox() {
        ////////////////////////////////
        //      Testing Sandbox.      //
        ////////////////////////////////
        
        for base in Orientation::iter_rotation_order() {
            for orient in Orientation::iter_rotation_order() {
                for cycle in -3..4 {
                    let reoriented = base.reorient_cycle(orient, cycle);
                    let deoriented = reoriented.deorient_cycle(orient, cycle);
                    assert_eq!(base, deoriented);
                }
            }
        }
    }

    #[test]
    fn in_canonical_test() {
        const GROUPS: [CanonicalGroup; 4] = unsafe { core::mem::transmute([0u8, 1, 2, 3]) };
        for orient in Orientation::iter() {
            for group in GROUPS {
                let in_group_x = orient.in_canonical_group_x(group);
                let in_group_y = orient.in_canonical_group_y(group);
                let in_group_z = orient.in_canonical_group_z(group);
                assert!(orient.is_equivalent(in_group_x));
                assert!(orient.is_equivalent(in_group_y));
                assert!(orient.is_equivalent(in_group_z));
            }
        }
    }

    #[test]
    fn rotation_first_order_test() {
        let iterator = ::core::iter::zip(
            Orientation::ROTATION_ORDERED.iter().copied(),
            RotationFirstOrientationIterator::START,
        );
        for (lhs, rhs) in iterator {
            assert_eq!(lhs, rhs);
        }
    }

    #[test]
    fn deorient_local_test() {
        for [base, orient] in Orientation::cartesian_product() {
            let reorient = base.reorient(orient);
            let deorient = reorient.deorient(orient);
            assert_eq!(base, deorient);
            if orient != Orientation::IDENTITY {
                assert_ne!(base, reorient);
                assert_ne!(reorient, deorient);
            }
        }
    }
}
