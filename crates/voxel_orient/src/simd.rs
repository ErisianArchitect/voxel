// TODO: This is not yet ready. Figure out how to make this work on AMD64. No SIMD support for other targets.

// In my benchmarks on x86_64, using SIMD is faster than matrix multiplication for transforming vertices.

#[repr(u8)]
#[derive(Clone, Copy)]
enum XYZ {
    X = 3,
    Y = 2,
    Z = 1,
}

#[repr(C, align(8))]
#[derive(Clone, Copy)]
struct Orienter {
    x_map: XYZ,
    y_map: XYZ,
    z_map: XYZ,
    _pad0: u8,
    x_neg: u32,
    y_neg: u32,
    z_neg: u32,
}

impl Orienter {
    const DEFAULT: Self = Self {
        x_map: XYZ::X,
        y_map: XYZ::Y,
        z_map: XYZ::Z,
        _pad0: 0,
        x_neg: 0,
        y_neg: 0,
        z_neg: 0,
    };
    const ORIENTERS: [Orienter; 192] = {
        const fn orient_data(v: i32) -> (XYZ, u32) {
            use XYZ::*;
            const SIGN_BIT: u32 = 2u32.pow(31);
            match v {
                1 => (X, 0),
                2 => (Y, 0),
                3 => (Z, 0),
                -1 => (X, SIGN_BIT),
                -2 => (Y, SIGN_BIT),
                -3 => (Z, SIGN_BIT),
                _ => panic!("Wrong input, dumbass."),
            }
        }
        let mut orienters = [Orienter::DEFAULT; 192];
        let mut i = 0usize;
        while i < 192 {
            let v = (1, 2, 3);
            let o = unsafe { Orientation::from_u8_unchecked(i as u8) };
            let t = o.transform_i32(v);
            let x_d = orient_data(t.0);
            let y_d = orient_data(t.1);
            let z_d = orient_data(t.2);
            orienters[i] = Orienter {
                x_map: x_d.0,
                y_map: y_d.0,
                z_map: z_d.0,
                _pad0: 0,
                x_neg: x_d.1,
                y_neg: y_d.1,
                z_neg: z_d.1,
            };
            i += 1;
        }
        orienters
    };

    pub const fn from_orientation(orientation: Orientation) -> Self {
        Self::ORIENTERS[orientation.as_u8() as usize]
    }
    
    #[must_use]
    #[inline(always)]
    pub const fn orient(self, v: Vec3) -> Vec3 {
        unsafe {
            let ptr = &v as *const Vec3 as *const f32;
            let arr = ::core::slice::from_raw_parts(ptr, 3);
            // let x_ptr = ptr.byte_add(self.x_map as usize);
            // let y_ptr = ptr.byte_add(self.y_map as usize);
            // let z_ptr = ptr.byte_add(self.z_map as usize);

            let x = arr[self.x_map as usize];
            let y = arr[self.y_map as usize];
            let z = arr[self.z_map as usize];
            let x = f32::from_bits(x.to_bits() ^ self.x_neg);
            let y = f32::from_bits(y.to_bits() ^ self.y_neg);
            let z = f32::from_bits(z.to_bits() ^ self.z_neg);
            Vec3::new(x, y, z)
        }
    }
}

use std::arch::x86_64;

use glam::{Vec3, Vec4, Vec4Swizzles};
use lolevel::cache_padded::CachePadded;
use crate::Orientation;

union SimdTransmute {
    simd: x86_64::__m128,
    array: [u32; 4],
}
const fn simd(array: [u32; 4]) -> x86_64::__m128 {
    unsafe {
        SimdTransmute {
            array,
        }.simd
    }
}

#[repr(C, align(4))]
#[derive(Clone, Copy)]
struct Indices32i8([u8; 4]);

impl Indices32i8 {
    const NOOP: Self = Self([0x80, 0x80, 0x80, 0x80]);
    #[must_use]
    #[inline]
    pub const fn index(i: u8) -> Self {
        debug_assert!(i < 4);
        let start = i << 2;
        Self([
            // I just like the way it looks, okay!?
            start | 0,
            start | 1,
            start | 2,
            start | 3,
        ])
    }
}

union M128i8Transmute {
    simd: x86_64::__m128i,
    inds: [Indices32i8; 4],
    f32s: [f32; 4],
}

const fn indices(i0: u8, i1: u8, i2: u8) -> x86_64::__m128i {
    unsafe {
        ::core::mem::transmute([
            Indices32i8::index(i0),
            Indices32i8::index(i1),
            Indices32i8::index(i2),
            Indices32i8::NOOP,
        ])
    }
}

#[must_use]
#[inline]
const fn order(x: XYZ, y: XYZ, z: XYZ) -> x86_64::__m128i {
    indices(x as u8, y as u8, z as u8)
}

#[derive(Clone, Copy)]
#[repr(C, align(32))]
pub struct SimdOrienter {
    shuffle: std::arch::x86_64::__m128i,
    xor: std::arch::x86_64::__m128,
}

impl SimdOrienter {
    const ZERO: Self = unsafe {
        Self {
            shuffle: ::core::mem::zeroed(),
            xor: ::core::mem::zeroed(),
        }
    };
    pub const ORIENTERS: CachePadded<[SimdOrienter; 192]> = {
        let mut table = CachePadded::new([SimdOrienter::ZERO; 192]);
        let mut i = 0;
        while i < 192 {
            let orienter = Orienter::ORIENTERS[i];
            let shuffle = order(orienter.x_map, orienter.y_map, orienter.z_map);
            let xor = simd([orienter.x_neg, orienter.y_neg, orienter.z_neg, 0]);
            
            table.value[i] = SimdOrienter {
                shuffle,
                xor,
            };
            i += 1;
        }
        table
    };

    #[must_use]
    #[inline]
    pub const fn from_orient(orientation: Orientation) -> &'static Self {
        &Self::ORIENTERS.value[orientation.0 as usize]
    }

    #[must_use]
    #[inline(always)]
    pub fn orient(&self, vertex: Vec3) -> Vec3 {
        unsafe {
            use ::core::mem::transmute;
            let vertex = transmute(x86_64::_mm_set_ps(
                vertex.x,
                vertex.y,
                vertex.z,
                0.0
            ));
            let shuffled: x86_64::__m128 
                = transmute(x86_64::_mm_shuffle_epi8(vertex, self.shuffle));
            let vec4 = Vec4::from(
                x86_64::_mm_xor_ps(shuffled, self.xor)
            );
            vec4.xyz()
        }
    }
}
