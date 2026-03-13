// Last Reviewed: 2025-12-28
use paste::paste;

use crate::{direction::Direction};



#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlipState {
    #[default]
    None = 0b000,
    X = 0b001,
    Y = 0b010,
    Z = 0b100,
    XY = 0b011,
    XZ = 0b101,
    YZ = 0b110,
    XYZ = 0b111,
}

macro_rules! convert_ops {
    ($(
        $func_name:ident($op:tt)
    )*) => {
        $(
            #[inline(always)]
            pub const fn $func_name(self, rhs: Self) -> Self {
                unsafe {
                    let lhs = self as u8;
                    let rhs = rhs as u8;
                    let result = lhs $op rhs;
                    ::core::mem::transmute(result)
                }
            }
            paste!{
                #[inline(always)]
                pub const fn [<$func_name _assign>](&mut self, rhs: Self) {
                    *self = self.$func_name(rhs);
                }
            }
        )*
    };
}

impl FlipState {
    
    // #[inline(always)]
    // pub const fn as_u8(self) -> u8 {
    //     self as u8
    // }
    
    #[inline(always)]
    pub const unsafe fn from_u8_unchecked(value: u8) -> Self {
        debug_assert!(value < 8, "Value is out of range.");
        unsafe {
            ::core::mem::transmute(value)
        }
    }
    
    #[inline(always)]
    pub const fn from_u8(value: u8) -> Option<Self> {
        if value >= 8 {
            return None;
        }
        Some(unsafe { Self::from_u8_unchecked(value) })
    }
    
    #[inline(always)]
    pub const fn from_u8_wrapping(value: u8) -> Self {
        unsafe { Self::from_u8_unchecked(value & 0b111) }
    }
    
    convert_ops!(
        and(&)
        or(|)
        xor(^)
    );
    
    #[inline(always)]
    pub const fn eq(self, rhs: Self) -> bool {
        self as u8 == rhs as u8
    }
    
    #[inline(always)]
    pub const fn ne(self, rhs: Self) -> bool {
        self as u8 != rhs as u8
    }
    
    #[inline(always)]
    pub const fn inverted(self) -> Self {
        let lhs = self as u8;
        let result = lhs ^ 0b111;
        unsafe { Self::from_u8_unchecked(result) }
    }
    
    #[inline(always)]
    pub const fn x(self) -> bool {
        self as u8 & 0b001 == 0b001
    }
    
    #[inline(always)]
    pub const fn y(self) -> bool {
        self as u8 & 0b010 == 0b010
    }
    
    #[inline(always)]
    pub const fn z(self) -> bool {
        self as u8 & 0b100 == 0b100
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Flip(pub(crate) FlipState);

// Ensure that the niche optimization is working.
const _: () = {
    // I probably could have kept going, but this should be sufficient.
    // I think that there should be something like 248 niches, which means that this can nest like, a billion times.
    assert!(size_of::<Flip>() == size_of::<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Flip>>>>>>>>>>>>>>>>>());
};

macro_rules! flip_axes {
    ($(
        {const $const_name:ident = $val:expr; fn $fn_name:ident}
    )*) => {
        $(
            paste!{
                // pub const X: Self = Self(0b001);
                pub const $const_name: Self = Self($val);
                // pub const fn x(self) -> bool {
                //     self.0 & Self::X.0 == Self::X.0
                // }
                #[inline]
                pub const fn $fn_name(self) -> bool {
                    self.0 as u8 & Self::$const_name.0 as u8 == Self::$const_name.0 as u8
                }
                
                // pub const fn set_x(&mut self, value: bool) -> bool {
                //     let old = self.x();
                //     if value {
                //         self.0 |= Self::X.0;
                //     } else {
                //         self.0 &= const { Self::X.invert().0 };
                //     }
                //     old
                // }
                #[inline]
                pub const fn [<set_ $fn_name>](&mut self, value: bool) -> bool {
                    let old = self.$fn_name();
                    if value {
                        self.0.or_assign(Self::$const_name.0);
                    } else {
                        unsafe {
                            let self_u8 = self.0 as u8;
                            let rhs = Self::$const_name.0 as u8;
                            let new = self_u8 & !rhs;
                            self.0 = FlipState::from_u8_unchecked(new);
                        }
                        self.0.and_assign(const { Self::$const_name.0.inverted() })
                    }
                    old
                }
                
                // pub const fn with_x(mut self, value: bool) -> Self {
                //     self.set_x(value);
                //     self
                // }
                #[inline]
                pub const fn [<with_ $fn_name>](mut self, value: bool) -> Self {
                    self.[<set_ $fn_name>](value);
                    self
                }
                
                // pub const fn flip_x(self) -> Self {
                //     Self(self.0 ^ Self::X.0)
                // }
                #[inline]
                pub const fn [<flip_ $fn_name>](self) -> Self {
                    Self(self.0.xor(Self::$const_name.0))
                }
            }
        )*
    };
}

type Tup3<T> = (T, T, T);

macro_rules! flip_coord_impls {
    ($(
        $type:ty
    ),*$(,)?) => {
        $(
            paste!{
                pub const fn [<flip_coord_ $type>](self, coord: Tup3<$type>) -> Tup3<$type> {
                    let (x, y, z) = coord;
                    match self {
                        Self::NONE => (x, y, z),
                        Self::X => (-x, y, z),
                        Self::Y => (x, -y, z),
                        Self::XY => (-x, -y, z),
                        Self::Z => (x, y, -z),
                        Self::XZ => (-x, y, -z),
                        Self::YZ => (x, -y, -z),
                        Self::XYZ => (-x, -y, -z),
                    }
                }
            }
        )*
    };
}

impl Flip {
    flip_axes!(
        {const X   = FlipState::X;   fn x  } // 1
        {const XY  = FlipState::XY;  fn xy } // 3
        {const XZ  = FlipState::XZ;  fn xz } // 5
        {const Y   = FlipState::Y;   fn y  } // 2
        {const YZ  = FlipState::YZ;  fn yz } // 6
        {const Z   = FlipState::Z;   fn z  } // 4
        {const XYZ = FlipState::XYZ; fn xyz} // 7
    );
    pub const NONE: Flip = Flip(FlipState::None);
    pub const ALL: Flip = Flip::XYZ;

    #[inline]
    pub const fn new(x: bool, y: bool, z: bool) -> Self {
        Self(unsafe { FlipState::from_u8_unchecked((x as u8) | ((y as u8) << 1) | ((z as u8) << 2)) })
    }
    
    /// `bits` must be no greater than `0b111` (7).
    /// If a higher value is passed in, the behavior is undefined.
    #[inline(always)]
    pub const unsafe fn from_u8_unchecked(bits: u8) -> Self {
        Self(unsafe { FlipState::from_u8_unchecked(bits) })
    }
    
    #[inline(always)]
    pub const fn from_u8(bits: u8) -> Option<Self> {
        if bits >= 8 {
            return None;
        }
        Some(Self(unsafe { FlipState::from_u8_unchecked(bits) }))
    }
    
    #[inline(always)]
    pub const fn from_u8_wrapping(value: u8) -> Self {
        Self(FlipState::from_u8_wrapping(value))
    }
    
    #[inline]
    pub const fn as_u8(self) -> u8 {
        self.0 as u8
    }

    #[inline]
    pub const fn flip(self, flip: Flip) -> Self {
        Self(self.0.or(flip.0))
    }
    
    #[inline]
    pub const fn invert(self) -> Self {
        Self(self.0.xor(Self::ALL.0))
    }

    /// Xors all the bits.
    pub const fn bits_xor(self) -> bool {
        self.x() ^ self.y() ^ self.z()
    }

    pub fn flip_coord<T: Copy + std::ops::Neg<Output = T>, C: Into<(T, T, T)> + From<(T, T, T)>>(self, value: C) -> C {
        let (mut x, mut y, mut z): (T, T, T) = value.into();
        if self.x() {
            x = -x;
        }
        if self.y() {
            y = -y;
        }
        if self.z() {
            z = -z;
        }
        C::from((x, y, z))
    }
    
    flip_coord_impls!(
        i8,
        i16,
        i32,
        i64,
        i128,
        isize,
        f32,
        f64,
    );

    // I don't know how useful this would be, but the code is already written.
    /// Determines if a face is on an axis that is flipped.
    pub const fn is_flipped(self, face: Direction) -> bool {
        if self.0 as u8 == FlipState::None as u8 {
            return false;
        }
        use Direction::*;
        match face {
            NegX | PosX if self.x() => true,
            NegY | PosY if self.y() => true,
            NegZ | PosZ if self.z() => true,
            _ => false,
        }
    }
    
    #[inline]
    pub fn iter() -> impl Iterator<Item = Self> {
        (0..8).map(move |val| unsafe { Self::from_u8_unchecked(val) })
    }

    // /// If the [Flip] is being used to flip vertices, this method determines if the indices need to be reversed.
    // #[inline]
    // pub const fn reverse_indices(self) -> bool {
    //     self.x() ^ self.y() ^ self.z()
    // }

    #[cfg(feature = "glam")]
    #[inline]
    pub fn to_scale(self) -> glam::Vec3 {
        #[inline(always)]
        fn select_scale(flipped: bool) -> f32 {
            if flipped {
                -1.0
            } else {
                1.0
            }
        }
        glam::vec3(
            select_scale(self.x()),
            select_scale(self.y()),
            select_scale(self.z()),
        )
    }

    #[cfg(feature = "glam")]
    #[inline]
    pub fn to_matrix(self) -> glam::Mat4 {
        let scale = self.to_scale();
        glam::Mat4::from_scale(scale)
    }
}

impl std::ops::BitOr<Flip> for Flip {
    type Output = Self;
    
    #[inline]
    fn bitor(self, rhs: Flip) -> Self::Output {
        Self(self.0.or(rhs.0))
    }
}

impl std::ops::BitOrAssign<Flip> for Flip {
    #[inline]
    fn bitor_assign(&mut self, rhs: Flip) {
        *self = *self | rhs;
    }
}

impl std::ops::BitAnd<Flip> for Flip {
    type Output = Self;
    
    #[inline]
    fn bitand(self, rhs: Flip) -> Self::Output {
        Self(self.0.and(rhs.0))
    }
}

impl std::ops::BitAndAssign<Flip> for Flip {
    #[inline]
    fn bitand_assign(&mut self, rhs: Flip) {
        self.0.and_assign(rhs.0)
    }
}

impl std::ops::Add<Flip> for Flip {
    type Output = Flip;
    #[inline]
    fn add(self, rhs: Flip) -> Self::Output {
        self | rhs
    }
}

impl std::ops::AddAssign<Flip> for Flip {
    #[inline]
    fn add_assign(&mut self, rhs: Flip) {
        self.0.or_assign(rhs.0);
    }
}

impl std::ops::Sub<Flip> for Flip {
    type Output = Flip;
    
    #[inline]
    fn sub(self, rhs: Flip) -> Self::Output {
        self & !rhs
    }
}

impl std::ops::SubAssign<Flip> for Flip {
    #[inline]
    fn sub_assign(&mut self, rhs: Flip) {
        *self = *self & !rhs;
    }
}

impl std::ops::Not for Flip {
    type Output = Self;
    
    #[inline]
    fn not(self) -> Self::Output {
        Self(self.0.inverted())
    }
}

impl std::fmt::Display for Flip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Flip(")?;
        let mut sep = false;
        if self.x() {
            write!(f, "X")?;
            sep = true;
        }
        if self.y() {
            if sep {
                write!(f, "|")?;
            }
            write!(f, "Y")?;
        }
        if self.z() {
            if sep {
                write!(f, "|")?;
            }
            write!(f, "Z")?;
        }
        write!(f, ")")
    }
}