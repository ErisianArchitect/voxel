use paste::paste;

macro_rules! align_structs {
    ($(
        $align:literal
    ),*) => {
        $(
            paste!{
                #[repr(C, align($align))]
                #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
                pub struct [<Align $align>]<T = ()>(pub T);
                
                impl<T> [<Align $align>]<T> {
                    #[must_use]
                    #[inline(always)]
                    pub const fn new(value: T) -> Self {
                        Self(value)
                    }
                    
                    #[must_use]
                    #[inline(always)]
                    pub fn into_inner(self) -> T {
                        self.0
                    }
                }
            }
        )*
    };
}

align_structs!(1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384);