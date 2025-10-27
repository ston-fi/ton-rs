use crate::cell::CellBuilder;
use crate::cell::CellParser;
use crate::errors::TonCoreError;
use crate::traits::tlb::TLB;

use fastnum::{I128, I256, I512, U128, U256, U512};

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

// fastnum
tlb_num_impl!(I128, 128);
tlb_num_impl!(I256, 256);
tlb_num_impl!(I512, 512);

tlb_num_impl!(U128, 128);
tlb_num_impl!(U256, 256);
tlb_num_impl!(U512, 512);

#[cfg(test)]
mod tests {
    use crate::traits::tlb::TLB;
    use fastnum::{i256, i512, u256, u512};
    use fastnum::{I256, I512, U256, U512};
    use tokio_test::assert_ok;

    #[test]

    fn test_tlb_num() -> anyhow::Result<()> {
        assert_ok!((-1i8).to_cell());
        assert_ok!(1u8.to_cell());
        assert_ok!((-32i32).to_cell());
        assert_ok!(u256!(123).to_cell());
        assert_ok!(U256::from(32u8).to_cell());
        assert_ok!(i256!(123).to_cell());
        assert_ok!(I256::from(-32i32).to_cell());

        assert_ok!(u512!(123).to_cell());
        assert_ok!(U512::from(32u8).to_cell());
        assert_ok!(i512!(-123).to_cell());
        assert_ok!(I512::from(-32i32).to_cell());

        Ok(())
    }
}
