use crate::cell::{CellBuilder, CellParser};

use std::fmt::Display;
mod ton_cell_bignum;
mod ton_cell_fastnum;
mod ton_cell_primitives;

use crate::errors::TonCoreError;

/// Allows generic read/write operation for any numeric type
pub trait TonCellNum: Display + Sized + Clone {
    /// CellBuilder guarantees 0 < bits_len < 1024
    fn tcn_write_bits(&self, writer: &mut CellBuilder, bits_len: u32) -> Result<(), TonCoreError>;
    /// CellWriter guarantees 0 <= bits_len < 1024
    fn tcn_read_bits(reader: &mut CellParser, bits_len: u32) -> Result<Self, TonCoreError>;
    fn tcn_is_zero(&self) -> bool;
    fn tcn_min_bits_len(&self) -> u32;

    /// Returns the maximum bit size for padding purposes in write_num
    /// For fixed-size types, this is sizeof(T) * 8
    /// For BigInt/BigUint, this is 1024 (same as I1024/U1024)
    fn tcn_max_bits_len() -> u32;
}

#[macro_export]
macro_rules! unsinged_highest_bit_pos {
    ($val:expr,$T:ty) => {{
        let max_bit_id = (std::mem::size_of::<$T>() * 8 - 1) as u32;
        (max_bit_id - $val.leading_zeros())
    }};
}

#[macro_export]
macro_rules! toncellnum_use_type_as {
    (
        $Src:ty,
        $Dst:ty,
        $src_to_dst:expr,   // fn(Src) -> TonCoreResult<Dst>
        $dst_to_src:expr   // fn(Dst) -> TonCoreResult<Src>
    ) => {
        impl TonCellNum for $Src {
            fn tcn_write_bits(&self, writer: &mut CellBuilder, bits_len: u32) -> Result<(), TonCoreError> {
                // fallible Src -> Dst
                let val_as: $Dst = $src_to_dst(self)?;
                val_as.tcn_write_bits(writer, bits_len)
            }

            fn tcn_read_bits(reader: &mut CellParser, bits_len: u32) -> Result<Self, TonCoreError> {
                let val_as = <$Dst>::tcn_read_bits(reader, bits_len)?;
                // fallible Dst -> Src, already returns TonCoreError
                $dst_to_src(val_as)
            }

            fn tcn_is_zero(&self) -> bool {
                // If conversion here can *theoretically* fail, treat it as a logic bug.
                let val_as: $Dst =
                    $src_to_dst(self).expect("toncellnum_use_type_as: Src -> Dst conversion failed in tcn_is_zero");
                val_as.tcn_is_zero()
            }

            fn tcn_min_bits_len(&self) -> u32 {
                let val_as: $Dst = $src_to_dst(self)
                    .expect("toncellnum_use_type_as: Src -> Dst conversion failed in tcn_min_bits_len");
                val_as.tcn_min_bits_len()
            }

            fn tcn_max_bits_len() -> u32 { <$Dst>::tcn_max_bits_len() }
        }
    };
}

#[cfg(test)]
mod tests {

    use crate::cell::TonCellNum;
    use crate::cell::{CellParser, TonCell};
    use fastnum::*;
    use num_bigint::{BigInt, BigUint};
    use std::fmt::Debug;
    //
    pub(crate) fn test_num_read_write<T: TonCellNum + PartialEq + Debug>(
        test_cases: Vec<(T, u32)>,
        type_name: &str,
    ) -> anyhow::Result<()> {
        for (value, bits_len) in test_cases {
            let mut builder = TonCell::builder();
            builder.write_num(&value, bits_len as usize).unwrap();
            let cell = builder.build().unwrap();
            let mut parser = CellParser::new(&cell);
            let parsed = parser.read_num::<T>(bits_len as usize).unwrap();
            assert_eq!(parsed, value, "Failed for {} value {:?} with {} bits", type_name, value, bits_len);
        }
        Ok(())
    }

    #[test]
    fn test_toncellnum_write_num() -> anyhow::Result<()> {
        let num_val = -9i32;
        let bigint_val = BigInt::from(-9i32);
        let i512_val = -I512::from(9i32);

        let mut num_builder = TonCell::builder();
        let mut bigint_builder = TonCell::builder();
        let mut i512_builder = TonCell::builder();

        num_builder.write_num(&num_val, 10).unwrap();
        bigint_builder.write_num(&bigint_val, 10).unwrap();
        let num_cell = num_builder.build().unwrap();
        i512_builder.write_num(&i512_val, 10).unwrap();

        // Two's complement: -9 in 10 bits = 1111110111
        // As bytes (10 bits = 2 bytes): 11111101 11000000 = [253, 192]
        assert_eq!(*num_cell.cell_data.data_storage, vec![0b1111_1101u8, 0b1100_0000u8]);
        assert_eq!(*num_cell.cell_data.data_storage, *bigint_builder.build()?.cell_data.data_storage);
        assert_eq!(*num_cell.cell_data.data_storage, *i512_builder.build()?.cell_data.data_storage);
        Ok(())
    }

    // Test reading and writing zero bits for all supported numeric types
    #[test]
    fn test_toncellnum_zero_bits_all_types() -> anyhow::Result<()> {
        // Test that all types return 0 when reading/writing 0 bits
        let builder = TonCell::builder();
        let cell = builder.build()?;
        let mut parser = CellParser::new(&cell);

        // Test unsigned primitives
        assert_eq!(parser.read_num::<u8>(0)?, 0u8);
        assert_eq!(parser.read_num::<u16>(0)?, 0u16);
        assert_eq!(parser.read_num::<u32>(0)?, 0u32);
        assert_eq!(parser.read_num::<u64>(0)?, 0u64);
        assert_eq!(parser.read_num::<u128>(0)?, 0u128);

        // Test signed primitives
        assert_eq!(parser.read_num::<i8>(0)?, 0i8);
        assert_eq!(parser.read_num::<i16>(0)?, 0i16);
        assert_eq!(parser.read_num::<i32>(0)?, 0i32);
        assert_eq!(parser.read_num::<i64>(0)?, 0i64);
        assert_eq!(parser.read_num::<i128>(0)?, 0i128);

        // Test usize
        assert_eq!(parser.read_num::<usize>(0)?, 0usize);

        // Test BigUint and BigInt
        assert_eq!(parser.read_num::<BigUint>(0)?, BigUint::from(0u32));
        assert_eq!(parser.read_num::<BigInt>(0)?, BigInt::from(0i32));

        // Test fastnum unsigned
        assert_eq!(parser.read_num::<U128>(0)?, U128::from(0u32));
        assert_eq!(parser.read_num::<U256>(0)?, U256::from(0u32));
        assert_eq!(parser.read_num::<U512>(0)?, U512::from(0u32));
        assert_eq!(parser.read_num::<U1024>(0)?, U1024::from(0u32));

        // Test fastnum signed
        assert_eq!(parser.read_num::<I128>(0)?, I128::from(0u32));
        assert_eq!(parser.read_num::<I256>(0)?, I256::from(0u32));
        assert_eq!(parser.read_num::<I512>(0)?, I512::from(0u32));
        assert_eq!(parser.read_num::<I1024>(0)?, I1024::from(0u32));

        Ok(())
    }
}
