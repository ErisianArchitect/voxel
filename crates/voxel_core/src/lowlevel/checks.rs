use std::{num::NonZero, ptr::NonNull};

#[track_caller]
#[inline(always)]
pub const fn assert_size<T, const SIZE: usize>() {
    if const { size_of::<T>() != SIZE } {
        panic!("Size mismatch");
    }
}
const _: () = assert_size::<u8, 1>();

#[track_caller]
#[inline(always)]
pub const fn assert_same_size<L, R>() {
    if const { size_of::<L>() != size_of::<R>() } {
        panic!("Size mismatch");
    }
}
const _: () = assert_same_size::<u8, i8>();

#[track_caller]
#[inline(always)]
pub const fn assert_different_size<L, R>() {
    if const { size_of::<L>() == size_of::<R>() } {
        panic!("Size match");
    }
}
const _: () = assert_different_size::<u8, u16>();

#[track_caller]
#[inline(always)]
pub const fn assert_compatible_size<Min, Max>() {
    if const { size_of::<Min>() > size_of::<Max>() } {
        panic!("Incompatible size");
    }
}
const _: () = assert_compatible_size::<u8, u16>();

#[track_caller]
#[inline(always)]
pub const fn assert_align<T, const ALIGN: usize>() {
    if const { align_of::<T>() != ALIGN } {
        panic!("Alignment mismatch");
    }
}
const _: () = assert_align::<u64, 8>();

#[track_caller]
#[inline(always)]
pub const fn assert_same_align<L, R>() {
    if const { align_of::<L>() != align_of::<R>() } {
        panic!("Alignment mismatch");
    }
}
const _: () = assert_same_align::<u8, i8>();

#[track_caller]
#[inline(always)]
pub const fn assert_different_align<L, R>() {
    if const { align_of::<L>() == align_of::<R>() } {
        panic!("Same alignment");
    }
}
const _: () = assert_different_align::<u8, u16>();

#[track_caller]
#[inline(always)]
pub const fn assert_compatible_align<Min, Max>() {
    if const { align_of::<Min>() > align_of::<Max>() } {
        panic!("Incompatible align");
    }
}
const _: () = assert_compatible_align::<u8, u16>();

#[track_caller]
#[inline(always)]
pub const fn assert_size_align<T, const SIZE: usize, const ALIGN: usize>() {
    let size_check = const { size_of::<T>() == SIZE };
    let align_check = const { align_of::<T>() == ALIGN };
    match (size_check, align_check) {
        (false, false) => panic!("Size and alignment mismatch"),
        (true, false) => panic!("Alignment mismatch"),
        (false, true) => panic!("Size mismatch"),
        (true, true) => (),
    }
}

#[track_caller]
#[inline(always)]
pub const fn assert_same_size_align<L, R>() {
    let size_check = const { size_of::<L>() == size_of::<R>() };
    let align_check = const { align_of::<L>() == align_of::<R>() };
    match (size_check, align_check) {
        (false, false) => panic!("Size and alignment mismatch"),
        (true, false) => panic!("Alignment mismatch"),
        (false, true) => panic!("Size mismatch"),
        (true, true) => (),
    }
}
const _: () = assert_same_size_align::<i8, u8>();

#[track_caller]
#[inline(always)]
pub const fn assert_compatible_size_align<Min, Max>() {
    let size_check = const { size_of::<Min>() <= size_of::<Max>() };
    let align_check = const { align_of::<Min>() <= align_of::<Max>() };
    match (size_check, align_check) {
        (false, false) => panic!("Incompatible size and alignment"),
        (true, false) => panic!("Incompatible alignment"),
        (false, true) => panic!("Incompatible size"),
        (true, true) => (),
    }
}
const _: () = assert_compatible_size_align::<u8, u32>();

#[track_caller]
#[inline(always)]
pub const fn assert_pointer_sized<T>() {
    assert_same_size_align::<T, usize>()
}
const _: () = assert_pointer_sized::<*const ()>();

#[track_caller]
#[inline(always)]
pub const fn assert_pointer_compatible_size_align<T>() {
    assert_compatible_size_align::<T, usize>();
}
const _: () = assert_pointer_compatible_size_align::<u8>();

#[track_caller]
#[inline(always)]
pub const fn assert_t_niche<T, Niched>() {
    assert_same_size_align::<T, Option<Niched>>();
    // Redundant checks for explicit completeness.
    // This ensures that the invariants never change.
    assert_same_size_align::<T, Result<Niched, ()>>();
    assert_same_size_align::<T, Result<(), Niched>>();
}
const _: () = assert_t_niche::<u8, NonZero<u8>>();

#[track_caller]
#[inline(always)]
pub const fn assert_t_niche_compatible<T, Niched>() {
    assert_compatible_size_align::<T, Option<Niched>>();
}
const _: () = assert_t_niche_compatible::<u8, NonZero<u16>>();

#[track_caller]
#[inline(always)]
pub const fn assert_niche<T>() {
    assert_t_niche::<T, T>();
}
const _: () = assert_niche::<NonZero<u32>>();

#[track_caller]
#[inline(always)]
pub const fn assert_pointer_niche<T>() {
    assert_t_niche::<usize, T>();
}
const _: () = assert_pointer_niche::<NonNull<()>>();

#[track_caller]
#[inline(always)]
pub const fn assert_pointer_niche_compatible<T>() {
    assert_t_niche_compatible::<usize, T>();
}
const _: () = assert_pointer_niche_compatible::<NonZero<u128>>();

#[track_caller]
#[inline(always)]
pub const fn assert_byte_niche<T>() {
    assert_t_niche::<u8, T>();
}
const _: () = assert_byte_niche::<NonZero<u8>>();