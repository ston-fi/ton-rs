mod tlb_bool;
mod tlb_cell;
mod tlb_num;
mod tlb_opt;
mod tlb_ptr;

use crate::bail_ton_core_data;
use crate::boc::BoC;
use crate::cell::CellBuilder;
use crate::cell::CellParser;
use crate::cell::CellType;
use crate::cell::{TonCell, TonCellRef, TonHash};
use crate::errors::TonCoreError;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;

pub trait TLB: Sized {
    const PREFIX: TLBPrefix = TLBPrefix::NULL;

    /// read-write definition
    /// https://docs.ton.org/v3/documentation/data-formats/tlb/tl-b-language#overview
    ///
    /// must be implemented by all TLB objects
    ///
    /// doesn't include prefix handling
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError>;
    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonCoreError>;

    /// interface - must be used by external code to read/write TLB objects
    fn read(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        Self::verify_prefix(parser)?;
        Self::read_definition(parser)
    }

    fn write(&self, builder: &mut CellBuilder) -> Result<(), TonCoreError> {
        Self::write_prefix(builder)?;
        self.write_definition(builder)
    }

    // Utilities
    fn cell_hash(&self) -> Result<TonHash, TonCoreError> { Ok(self.to_cell()?.hash()?.clone()) }

    /// Reading
    fn from_cell(cell: &TonCell) -> Result<Self, TonCoreError> { Self::read(&mut cell.parser()) }

    fn from_boc(boc: &[u8]) -> Result<Self, TonCoreError> {
        match BoC::from_bytes(boc).and_then(|x| x.single_root()).and_then(|x| Self::from_cell(&x)) {
            Ok(cell) => Ok(cell),
            Err(err) => {
                bail_ton_core_data!(
                    "Fail to read {} from bytes: {}, err: {err}",
                    std::any::type_name::<Self>(),
                    hex::encode(boc)
                );
            }
        }
    }

    fn from_boc_hex(boc: &str) -> Result<Self, TonCoreError> { Self::from_boc(&hex::decode(boc)?) }

    fn from_boc_b64(boc: &str) -> Result<Self, TonCoreError> { Self::from_boc(&STANDARD.decode(boc)?) }

    /// Writing
    fn to_cell(&self) -> Result<TonCell, TonCoreError> {
        let mut builder = TonCell::builder_typed(self.cell_type());
        self.write(&mut builder)?;
        builder.build()
    }

    fn to_cell_ref(&self) -> Result<TonCellRef, TonCoreError> { Ok(self.to_cell()?.into_ref()) }

    fn to_boc(&self) -> Result<Vec<u8>, TonCoreError> { self.to_boc_extra(false) }

    fn to_boc_hex(&self) -> Result<String, TonCoreError> { self.to_boc_hex_extra(false) }

    fn to_boc_base64(&self) -> Result<String, TonCoreError> { self.to_boc_base64_extra(false) }

    fn to_boc_extra(&self, add_crc32: bool) -> Result<Vec<u8>, TonCoreError> {
        let mut builder = TonCell::builder();
        self.write(&mut builder)?;
        BoC::new(builder.build()?.into_ref()).to_bytes(add_crc32)
    }

    fn to_boc_hex_extra(&self, add_crc32: bool) -> Result<String, TonCoreError> {
        Ok(hex::encode(self.to_boc_extra(add_crc32)?))
    }

    fn to_boc_base64_extra(&self, add_crc32: bool) -> Result<String, TonCoreError> {
        Ok(STANDARD.encode(self.to_boc_extra(add_crc32)?))
    }

    /// Helpers - mostly for internal use
    fn verify_prefix(reader: &mut CellParser) -> Result<(), TonCoreError> {
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

        if reader.data_bits_remaining()? < Self::PREFIX.bits_len {
            return prefix_error(0, reader.data_bits_remaining()?);
        }

        // we handle cell_underflow above - all other errors can be rethrown
        let actual_val: usize = reader.read_num(Self::PREFIX.bits_len)?;

        if actual_val != Self::PREFIX.value {
            reader.seek_bits(-(Self::PREFIX.bits_len as i32))?; // revert reader position
            return prefix_error(actual_val, reader.data_bits_remaining()?);
        }
        Ok(())
    }

    fn write_prefix(builder: &mut CellBuilder) -> Result<(), TonCoreError> {
        if Self::PREFIX != TLBPrefix::NULL {
            builder.write_num(&Self::PREFIX.value, Self::PREFIX.bits_len)?;
        }
        Ok(())
    }

    // when we write an object, we have to idea of it's type - including writing TonCell itself
    // so for all types except TonCell & TonCellRef we return Ordinary, but for them we return proper type
    // it's required to build proper BOC
    fn cell_type(&self) -> CellType { CellType::Ordinary }
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
