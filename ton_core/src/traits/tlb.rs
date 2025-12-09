#[cfg(test)]
mod test_tlb_enum;
mod tlb_bool;
mod tlb_cell;
mod tlb_num;
mod tlb_opt;
mod tlb_ptr;

use crate::bail_ton_core_data;
use crate::cell::CellBuilder;
use crate::cell::CellParser;
use crate::cell::CellType;
use crate::cell::{BoC, INITIAL_STORAGE_CAPACITY};
use crate::cell::{TonCell, TonHash};
use crate::errors::{TonCoreError, TonCoreResult};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use std::any::type_name;
use std::ops::Deref;
use std::sync::Arc;

pub trait TLB: Sized {
    const PREFIX: TLBPrefix = TLBPrefix::NULL;

    /// read-write definition
    /// https://docs.ton.org/v3/documentation/data-formats/tlb/tl-b-language#overview
    ///
    /// must be implemented by all TLB objects
    ///
    /// doesn't include prefix handling
    fn read_definition(parser: &mut CellParser) -> TonCoreResult<Self>;
    fn write_definition(&self, builder: &mut CellBuilder) -> TonCoreResult<()>;

    /// interface - must be used by external code to read/write TLB objects
    fn read(parser: &mut CellParser) -> TonCoreResult<Self> {
        Self::verify_prefix(parser)?;
        Self::read_definition(parser)
    }

    fn write(&self, builder: &mut CellBuilder) -> TonCoreResult<()> {
        Self::write_prefix(builder)?;
        self.write_definition(builder)
    }

    // Utilities
    fn cell_hash(&self) -> TonCoreResult<TonHash> { Ok(self.to_cell()?.hash()?.clone()) }

    /// Reading
    fn from_cell(cell: &TonCell) -> TonCoreResult<Self> { Self::read(&mut cell.parser()) }

    fn from_boc<T: Into<Arc<Vec<u8>>>>(boc: T) -> TonCoreResult<Self> {
        let boc = boc.into();
        match BoC::from_bytes(boc.clone()).and_then(|x| x.single_root()).and_then(|x| Self::from_cell(&x)) {
            Ok(cell) => Ok(cell),
            Err(TonCoreError::TLBEnumOutOfOptions { message, cell_boc_hex }) => {
                Err(TonCoreError::TLBEnumOutOfOptions { message, cell_boc_hex })
            }
            Err(err) => bail_ton_core_data!(
                "Fail to read {} from bytes: {}, err: {err}",
                type_name::<Self>(),
                hex::encode(boc.deref())
            ),
        }
    }

    fn from_boc_hex(boc: &str) -> TonCoreResult<Self> { Self::from_boc(hex::decode(boc)?) }

    fn from_boc_base64(boc: &str) -> TonCoreResult<Self> { Self::from_boc(STANDARD.decode(boc)?) }

    /// Writing
    fn to_cell(&self) -> TonCoreResult<TonCell> {
        let mut builder = TonCell::builder_extra(self.ton_cell_type(), INITIAL_STORAGE_CAPACITY);
        self.write(&mut builder)?;
        builder.build()
    }

    fn to_boc(&self) -> TonCoreResult<Vec<u8>> { self.to_boc_extra(false) }

    fn to_boc_hex(&self) -> TonCoreResult<String> { self.to_boc_hex_extra(false) }

    fn to_boc_base64(&self) -> TonCoreResult<String> { self.to_boc_base64_extra(false) }

    fn to_boc_extra(&self, add_crc32: bool) -> TonCoreResult<Vec<u8>> {
        let mut builder = TonCell::builder();
        self.write(&mut builder)?;
        BoC::new(builder.build()?).to_bytes(add_crc32)
    }

    fn to_boc_hex_extra(&self, add_crc32: bool) -> TonCoreResult<String> {
        Ok(hex::encode(self.to_boc_extra(add_crc32)?))
    }

    fn to_boc_base64_extra(&self, add_crc32: bool) -> TonCoreResult<String> {
        Ok(STANDARD.encode(self.to_boc_extra(add_crc32)?))
    }

    /// Helpers - mostly for internal use
    fn verify_prefix(reader: &mut CellParser) -> TonCoreResult<()> {
        if Self::PREFIX == TLBPrefix::NULL {
            return Ok(());
        }

        let prefix_error = |given, bits_left| {
            Err(TonCoreError::TLBWrongPrefix {
                exp: Self::PREFIX.value,
                given,
                bits_exp: Self::PREFIX.bits_len,
                bits_left,
            })
        };

        if reader.data_bits_left()? < Self::PREFIX.bits_len {
            return prefix_error(0, reader.data_bits_left()?);
        }

        // we handle cell_underflow above - all other errors can be rethrown
        let actual_val: usize = reader.read_num(Self::PREFIX.bits_len)?;

        if actual_val != Self::PREFIX.value {
            reader.seek_bits(-(Self::PREFIX.bits_len as i32))?; // revert reader position
            return prefix_error(actual_val, reader.data_bits_left()?);
        }
        Ok(())
    }

    fn write_prefix(builder: &mut CellBuilder) -> TonCoreResult<()> {
        if Self::PREFIX != TLBPrefix::NULL {
            builder.write_num(&Self::PREFIX.value, Self::PREFIX.bits_len)?;
        }
        Ok(())
    }

    // when we write an object, we have to idea of it's type - including writing TonCell itself
    // so for all types except TonCell & TonCellRef we return Ordinary, but for them we return proper type
    // it's required to build a proper BOC
    fn ton_cell_type(&self) -> CellType { CellType::Ordinary }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TLBPrefix {
    pub value: usize,
    pub bits_len: usize,
}

impl TLBPrefix {
    pub const NULL: TLBPrefix = TLBPrefix::new(0, 0);
    pub const fn new(value: usize, bits_len: usize) -> Self { TLBPrefix { value, bits_len } }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ton_macros::TLB;

    #[test]
    fn test_tlb_derive() -> anyhow::Result<()> {
        #[derive(TLB)]
        #[tlb(prefix = 0x01, bits_len = 8)]
        struct TestTLBObject(u32);
        let cell = TestTLBObject(42).to_cell()?;
        let parsed = TestTLBObject::from_cell(&cell)?;
        assert_eq!(parsed.0, 42);

        let parser = &mut cell.parser();
        let prefix: u32 = parser.read_num(8)?;
        assert_eq!(prefix, 0x01);

        let data: u32 = parser.read_num(32)?;
        assert_eq!(data, 42);
        Ok(())
    }

    #[test]
    fn test_tlb_derive_const_prefix() -> anyhow::Result<()> {
        const PREFIX: usize = 0x02;

        #[derive(TLB)]
        #[tlb(prefix = PREFIX, bits_len = 8)]
        struct TestTLBObject(u32);
        let cell = TestTLBObject(42).to_cell()?;
        let parsed = TestTLBObject::from_cell(&cell)?;
        assert_eq!(parsed.0, 42);
        Ok(())
    }
}
