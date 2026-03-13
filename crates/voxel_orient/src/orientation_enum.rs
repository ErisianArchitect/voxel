use vcore::lowlevel::cache_padded::CachePadded;


// Used for niche optimization in [Orientation] struct.
#[allow(unused)]
#[repr(u8)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Orient {
    #[default]
    PosY0    = 0  , // 0b00000000
    PosY0X   = 1  , // 0b00000001
    PosY0Y   = 2  , // 0b00000010
    PosY0XY  = 3  , // 0b00000011
    PosY0Z   = 4  , // 0b00000100
    PosY0XZ  = 5  , // 0b00000101
    PosY0YZ  = 6  , // 0b00000110
    PosY0XYZ = 7  , // 0b00000111
    PosY1    = 8  , // 0b00001000
    PosY1X   = 9  , // 0b00001001
    PosY1Y   = 10 , // 0b00001010
    PosY1XY  = 11 , // 0b00001011
    PosY1Z   = 12 , // 0b00001100
    PosY1XZ  = 13 , // 0b00001101
    PosY1YZ  = 14 , // 0b00001110
    PosY1XYZ = 15 , // 0b00001111
    PosY2    = 16 , // 0b00010000
    PosY2X   = 17 , // 0b00010001
    PosY2Y   = 18 , // 0b00010010
    PosY2XY  = 19 , // 0b00010011
    PosY2Z   = 20 , // 0b00010100
    PosY2XZ  = 21 , // 0b00010101
    PosY2YZ  = 22 , // 0b00010110
    PosY2XYZ = 23 , // 0b00010111
    PosY3    = 24 , // 0b00011000
    PosY3X   = 25 , // 0b00011001
    PosY3Y   = 26 , // 0b00011010
    PosY3XY  = 27 , // 0b00011011
    PosY3Z   = 28 , // 0b00011100
    PosY3XZ  = 29 , // 0b00011101
    PosY3YZ  = 30 , // 0b00011110
    PosY3XYZ = 31 , // 0b00011111
    PosX0    = 32 , // 0b00100000
    PosX0X   = 33 , // 0b00100001
    PosX0Y   = 34 , // 0b00100010
    PosX0XY  = 35 , // 0b00100011
    PosX0Z   = 36 , // 0b00100100
    PosX0XZ  = 37 , // 0b00100101
    PosX0YZ  = 38 , // 0b00100110
    PosX0XYZ = 39 , // 0b00100111
    PosX1    = 40 , // 0b00101000
    PosX1X   = 41 , // 0b00101001
    PosX1Y   = 42 , // 0b00101010
    PosX1XY  = 43 , // 0b00101011
    PosX1Z   = 44 , // 0b00101100
    PosX1XZ  = 45 , // 0b00101101
    PosX1YZ  = 46 , // 0b00101110
    PosX1XYZ = 47 , // 0b00101111
    PosX2    = 48 , // 0b00110000
    PosX2X   = 49 , // 0b00110001
    PosX2Y   = 50 , // 0b00110010
    PosX2XY  = 51 , // 0b00110011
    PosX2Z   = 52 , // 0b00110100
    PosX2XZ  = 53 , // 0b00110101
    PosX2YZ  = 54 , // 0b00110110
    PosX2XYZ = 55 , // 0b00110111
    PosX3    = 56 , // 0b00111000
    PosX3X   = 57 , // 0b00111001
    PosX3Y   = 58 , // 0b00111010
    PosX3XY  = 59 , // 0b00111011
    PosX3Z   = 60 , // 0b00111100
    PosX3XZ  = 61 , // 0b00111101
    PosX3YZ  = 62 , // 0b00111110
    PosX3XYZ = 63 , // 0b00111111
    PosZ0    = 64 , // 0b01000000
    PosZ0X   = 65 , // 0b01000001
    PosZ0Y   = 66 , // 0b01000010
    PosZ0XY  = 67 , // 0b01000011
    PosZ0Z   = 68 , // 0b01000100
    PosZ0XZ  = 69 , // 0b01000101
    PosZ0YZ  = 70 , // 0b01000110
    PosZ0XYZ = 71 , // 0b01000111
    PosZ1    = 72 , // 0b01001000
    PosZ1X   = 73 , // 0b01001001
    PosZ1Y   = 74 , // 0b01001010
    PosZ1XY  = 75 , // 0b01001011
    PosZ1Z   = 76 , // 0b01001100
    PosZ1XZ  = 77 , // 0b01001101
    PosZ1YZ  = 78 , // 0b01001110
    PosZ1XYZ = 79 , // 0b01001111
    PosZ2    = 80 , // 0b01010000
    PosZ2X   = 81 , // 0b01010001
    PosZ2Y   = 82 , // 0b01010010
    PosZ2XY  = 83 , // 0b01010011
    PosZ2Z   = 84 , // 0b01010100
    PosZ2XZ  = 85 , // 0b01010101
    PosZ2YZ  = 86 , // 0b01010110
    PosZ2XYZ = 87 , // 0b01010111
    PosZ3    = 88 , // 0b01011000
    PosZ3X   = 89 , // 0b01011001
    PosZ3Y   = 90 , // 0b01011010
    PosZ3XY  = 91 , // 0b01011011
    PosZ3Z   = 92 , // 0b01011100
    PosZ3XZ  = 93 , // 0b01011101
    PosZ3YZ  = 94 , // 0b01011110
    PosZ3XYZ = 95 , // 0b01011111
    NegY0    = 96 , // 0b01100000
    NegY0X   = 97 , // 0b01100001
    NegY0Y   = 98 , // 0b01100010
    NegY0XY  = 99 , // 0b01100011
    NegY0Z   = 100, // 0b01100100
    NegY0XZ  = 101, // 0b01100101
    NegY0YZ  = 102, // 0b01100110
    NegY0XYZ = 103, // 0b01100111
    NegY1    = 104, // 0b01101000
    NegY1X   = 105, // 0b01101001
    NegY1Y   = 106, // 0b01101010
    NegY1XY  = 107, // 0b01101011
    NegY1Z   = 108, // 0b01101100
    NegY1XZ  = 109, // 0b01101101
    NegY1YZ  = 110, // 0b01101110
    NegY1XYZ = 111, // 0b01101111
    NegY2    = 112, // 0b01110000
    NegY2X   = 113, // 0b01110001
    NegY2Y   = 114, // 0b01110010
    NegY2XY  = 115, // 0b01110011
    NegY2Z   = 116, // 0b01110100
    NegY2XZ  = 117, // 0b01110101
    NegY2YZ  = 118, // 0b01110110
    NegY2XYZ = 119, // 0b01110111
    NegY3    = 120, // 0b01111000
    NegY3X   = 121, // 0b01111001
    NegY3Y   = 122, // 0b01111010
    NegY3XY  = 123, // 0b01111011
    NegY3Z   = 124, // 0b01111100
    NegY3XZ  = 125, // 0b01111101
    NegY3YZ  = 126, // 0b01111110
    NegY3XYZ = 127, // 0b01111111
    NegX0    = 128, // 0b10000000
    NegX0X   = 129, // 0b10000001
    NegX0Y   = 130, // 0b10000010
    NegX0XY  = 131, // 0b10000011
    NegX0Z   = 132, // 0b10000100
    NegX0XZ  = 133, // 0b10000101
    NegX0YZ  = 134, // 0b10000110
    NegX0XYZ = 135, // 0b10000111
    NegX1    = 136, // 0b10001000
    NegX1X   = 137, // 0b10001001
    NegX1Y   = 138, // 0b10001010
    NegX1XY  = 139, // 0b10001011
    NegX1Z   = 140, // 0b10001100
    NegX1XZ  = 141, // 0b10001101
    NegX1YZ  = 142, // 0b10001110
    NegX1XYZ = 143, // 0b10001111
    NegX2    = 144, // 0b10010000
    NegX2X   = 145, // 0b10010001
    NegX2Y   = 146, // 0b10010010
    NegX2XY  = 147, // 0b10010011
    NegX2Z   = 148, // 0b10010100
    NegX2XZ  = 149, // 0b10010101
    NegX2YZ  = 150, // 0b10010110
    NegX2XYZ = 151, // 0b10010111
    NegX3    = 152, // 0b10011000
    NegX3X   = 153, // 0b10011001
    NegX3Y   = 154, // 0b10011010
    NegX3XY  = 155, // 0b10011011
    NegX3Z   = 156, // 0b10011100
    NegX3XZ  = 157, // 0b10011101
    NegX3YZ  = 158, // 0b10011110
    NegX3XYZ = 159, // 0b10011111
    NegZ0    = 160, // 0b10100000
    NegZ0X   = 161, // 0b10100001
    NegZ0Y   = 162, // 0b10100010
    NegZ0XY  = 163, // 0b10100011
    NegZ0Z   = 164, // 0b10100100
    NegZ0XZ  = 165, // 0b10100101
    NegZ0YZ  = 166, // 0b10100110
    NegZ0XYZ = 167, // 0b10100111
    NegZ1    = 168, // 0b10101000
    NegZ1X   = 169, // 0b10101001
    NegZ1Y   = 170, // 0b10101010
    NegZ1XY  = 171, // 0b10101011
    NegZ1Z   = 172, // 0b10101100
    NegZ1XZ  = 173, // 0b10101101
    NegZ1YZ  = 174, // 0b10101110
    NegZ1XYZ = 175, // 0b10101111
    NegZ2    = 176, // 0b10110000
    NegZ2X   = 177, // 0b10110001
    NegZ2Y   = 178, // 0b10110010
    NegZ2XY  = 179, // 0b10110011
    NegZ2Z   = 180, // 0b10110100
    NegZ2XZ  = 181, // 0b10110101
    NegZ2YZ  = 182, // 0b10110110
    NegZ2XYZ = 183, // 0b10110111
    NegZ3    = 184, // 0b10111000
    NegZ3X   = 185, // 0b10111001
    NegZ3Y   = 186, // 0b10111010
    NegZ3XY  = 187, // 0b10111011
    NegZ3Z   = 188, // 0b10111100
    NegZ3XZ  = 189, // 0b10111101
    NegZ3YZ  = 190, // 0b10111110
    NegZ3XYZ = 191, // 0b10111111
}

impl Orient {
    pub(crate) const CACHE_WRAP: CachePadded<[Self; 256]> = {
        let mut array = CachePadded::new([Self::PosY0; 256]);
        let mut index = 0usize;
        while index < 256 {
            array.value[index] = unsafe { Orient::from_u8_unchecked(index as u8 % 192) };
            index += 1;
        }
        array
    };
    
    #[inline(always)]
    pub const unsafe fn from_u8_unchecked(value: u8) -> Self {
        unsafe { ::core::mem::transmute(value) }
    }
    
    #[inline(always)]
    pub const fn from_u8_wrapping(value: u8) -> Self {
        Self::CACHE_WRAP.value[value as usize]
    }
}