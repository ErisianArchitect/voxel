// Copyright © 2026 Ada F. <https://github.com/ErisianArchitect>



#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Parity {
    Even = 0,
    Odd = 1,
}

pub trait NumParity {
    fn parity(self) -> Parity;
}

macro_rules! num_parities {
    (
        $(
            $type:ident
        ),*
        $(,)?
    ) => {
        paste::paste! {
            $(
                impl NumParity for $type {
                    #[inline(always)]
                    fn parity(self) -> Parity {
                        Parity::[< $type _parity >](self)
                    }
                }
                
                impl Parity {
                    #[doc = concat!("Get the [Parity] of a ", stringify!($type), " value.")]
                    #[must_use]
                    #[inline(always)]
                    pub const fn [< $type _parity >](value: $type) -> Self {
                        Self::INDEXED[(value & 1) as usize]
                    }
                }
            )*
        }
    };
}

num_parities! {
    i8, i16, i32, i64, i128, isize,
    u8, u16, u32, u64, u128, usize,
}

impl Parity {
    const INDEXED: [Self; 2] = [Self::Even, Self::Odd];

    #[must_use]
    #[inline(always)]
    pub const fn is_even(self) -> bool {
        matches!(self, Self::Even)
    }

    #[must_use]
    #[inline(always)]
    pub const fn is_odd(self) -> bool {
        matches!(self, Self::Odd)
    }
}
