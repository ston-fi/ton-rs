use num_bigint::{BigInt, BigUint};
use std::marker::PhantomData;
use ton_lib_core::cell::CellBuilder;
use ton_lib_core::cell::CellParser;
use ton_lib_core::errors::TonCoreError;

/// Adapter to write data with fixed length into a cell.
/// use `#[tlb_derive(bits_len={BITS_LEN})]` to apply it using TLBDerive macro
pub struct ConstLen<T> {
    bits_len: usize,
    _phantom: PhantomData<T>,
}

impl<T> ConstLen<T> {
    pub fn new(bits_len: usize) -> Self {
        Self {
            bits_len,
            _phantom: PhantomData,
        }
    }
}

// Num, Option<Num> impls
macro_rules! const_len_num_impl {
    ($src:ty) => {
        impl ConstLen<$src> {
            pub fn read(&self, parser: &mut CellParser) -> Result<$src, TonCoreError> { parser.read_num(self.bits_len) }
            pub fn write(&self, builder: &mut CellBuilder, val: &$src) -> Result<(), TonCoreError> {
                builder.write_num(val, self.bits_len)
            }
        }
        impl ConstLen<Option<$src>> {
            pub fn read(&self, parser: &mut CellParser) -> Result<Option<$src>, TonCoreError> {
                match parser.read_bit()? {
                    true => Ok(Some(parser.read_num(self.bits_len)?)),
                    false => Ok(None),
                }
            }
            pub fn write(&self, builder: &mut CellBuilder, val: &Option<$src>) -> Result<(), TonCoreError> {
                builder.write_bit(val.is_some())?;
                if let Some(val) = val {
                    return builder.write_num(val, self.bits_len);
                };
                Ok(())
            }
        }
    };
}

const_len_num_impl!(i8);
const_len_num_impl!(u8);
const_len_num_impl!(i16);
const_len_num_impl!(u16);
const_len_num_impl!(i32);
const_len_num_impl!(u32);
const_len_num_impl!(i64);
const_len_num_impl!(u64);
const_len_num_impl!(i128);
const_len_num_impl!(u128);
const_len_num_impl!(BigInt);
const_len_num_impl!(BigUint);

impl ConstLen<Vec<u8>> {
    pub fn read(&self, parser: &mut CellParser) -> Result<Vec<u8>, TonCoreError> { parser.read_bits(self.bits_len) }
    pub fn write(&self, builder: &mut CellBuilder, val: &Vec<u8>) -> Result<(), TonCoreError> {
        builder.write_bits(val, self.bits_len)
    }
}

impl ConstLen<Option<Vec<u8>>> {
    pub fn read(&self, parser: &mut CellParser) -> Result<Option<Vec<u8>>, TonCoreError> {
        match parser.read_bit()? {
            true => Ok(Some(parser.read_bits(self.bits_len)?)),
            false => Ok(None),
        }
    }
    pub fn write(&self, builder: &mut CellBuilder, val: &Option<Vec<u8>>) -> Result<(), TonCoreError> {
        match val {
            Some(val) => {
                builder.write_bit(true)?;
                builder.write_bits(val, self.bits_len)
            }
            None => builder.write_bit(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ton_lib_core::cell::TonCell;
    use ton_lib_core::traits::tlb::TLB;
    use ton_lib_core::TLB;

    #[test]
    fn test_const_len() -> anyhow::Result<()> {
        let mut builder = TonCell::builder();
        ConstLen::<u32>::new(24).write(&mut builder, &1u32)?;
        let cell = builder.build()?;
        assert_eq!(&cell.data, &[0, 0, 1]);
        let parsed = ConstLen::<u32>::new(24).read(&mut cell.parser())?;
        assert_eq!(parsed, 1u32);
        Ok(())
    }

    #[derive(TLB)]
    struct TestType {
        #[tlb(bits_len = 4)]
        a: u32,
    }

    #[test]
    fn test_cont_len_bits_len() -> anyhow::Result<()> {
        let mut builder = TonCell::builder();
        TestType { a: 1 }.write(&mut builder)?;
        let cell = builder.build()?;
        assert_eq!(&cell.data, &[0b00010000]);
        let parsed = TestType::read(&mut cell.parser())?;
        assert_eq!(parsed.a, 1u32);
        Ok(())
    }
}
