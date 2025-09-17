use crate::cell::CellBuilder;
use crate::cell::CellParser;
use crate::errors::TonCoreError;
use crate::traits::tlb::TLB;

macro_rules! tlb_num_impl {
    ($t:ty, $bits:tt) => {
        impl TLB for $t {
            fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> { parser.read_num($bits) }

            fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonCoreError> {
                builder.write_num(self, $bits)
            }
        }
    };
}

// BigNum doesn't have predefined len, so can't be implemented here
tlb_num_impl!(i8, 8);
tlb_num_impl!(i16, 16);
tlb_num_impl!(i32, 32);
tlb_num_impl!(i64, 64);
tlb_num_impl!(i128, 128);
tlb_num_impl!(u8, 8);
tlb_num_impl!(u16, 16);
tlb_num_impl!(u32, 32);
tlb_num_impl!(u64, 64);
tlb_num_impl!(u128, 128);
tlb_num_impl!(usize, 64);
