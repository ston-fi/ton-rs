mod tlb_bool;
mod tlb_cell;
mod tlb_num;
mod tlb_opt;
mod tlb_ptr;

use crate::boc::BOC;
use crate::cell::CellBuilder;
use crate::cell::CellParser;
use crate::cell::CellType;
use crate::cell::{TonCell, TonCellRef, TonHash};
use crate::error::TLCoreError;
use crate::error::TLCoreError::TLBWrongData;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;

pub trait TLB: Sized {
    const PREFIX: TLBPrefix = TLBPrefix::NULL;

    /// read-write definition
    /// https://docs.ton.org/v3/documentation/data-formats/tlb/tl-b-language#overview
    /// must be implemented by all TLB objects
    /// doesn't include prefix handling
    fn read_definition(parser: &mut CellParser) -> Result<Self, TLCoreError>;
    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TLCoreError>;

    /// interface - must be used by external code to read/write TLB objects
    fn read(parser: &mut CellParser) -> Result<Self, TLCoreError> {
        Self::verify_prefix(parser)?;
        Self::read_definition(parser)
    }

    fn write(&self, builder: &mut CellBuilder) -> Result<(), TLCoreError> {
        Self::write_prefix(builder)?;
        self.write_definition(builder)
    }

    // Utilities
    fn cell_hash(&self) -> Result<TonHash, TLCoreError> { Ok(self.to_cell()?.hash()?.clone()) }

    /// Reading
    fn from_cell(cell: &TonCell) -> Result<Self, TLCoreError> { Self::read(&mut cell.parser()) }

    fn from_boc(boc: &[u8]) -> Result<Self, TLCoreError> {
        match BOC::from_bytes(boc).and_then(|x| x.single_root()).and_then(|x| Self::from_cell(&x)) {
            Ok(cell) => Ok(cell),
            Err(err) => {
                let msg = format!(
                    "Fail to read {} from bytes: {}, err: {err}",
                    std::any::type_name::<Self>(),
                    hex::encode(boc)
                );
                Err(TLBWrongData(msg))
            }
        }
    }

    fn from_boc_hex(boc: &str) -> Result<Self, TLCoreError> { Self::from_boc(&hex::decode(boc)?) }

    fn from_boc_b64(boc: &str) -> Result<Self, TLCoreError> { Self::from_boc(&BASE64_STANDARD.decode(boc)?) }

    /// Writing
    fn to_cell(&self) -> Result<TonCell, TLCoreError> {
        let mut builder = TonCell::builder_typed(self.cell_type());
        self.write(&mut builder)?;
        builder.build()
    }

    fn to_cell_ref(&self) -> Result<TonCellRef, TLCoreError> { Ok(self.to_cell()?.into_ref()) }

    fn to_boc(&self) -> Result<Vec<u8>, TLCoreError> { self.to_boc_extra(false) }

    fn to_boc_hex(&self) -> Result<String, TLCoreError> { self.to_boc_hex_extra(false) }

    fn to_boc_base64(&self) -> Result<String, TLCoreError> { self.to_boc_base64_extra(false) }

    fn to_boc_extra(&self, add_crc32: bool) -> Result<Vec<u8>, TLCoreError> {
        let mut builder = TonCell::builder();
        self.write(&mut builder)?;
        BOC::new(builder.build()?.into_ref()).to_bytes(add_crc32)
    }

    fn to_boc_hex_extra(&self, add_crc32: bool) -> Result<String, TLCoreError> {
        Ok(hex::encode(self.to_boc_extra(add_crc32)?))
    }

    fn to_boc_base64_extra(&self, add_crc32: bool) -> Result<String, TLCoreError> {
        Ok(BASE64_STANDARD.encode(self.to_boc_extra(add_crc32)?))
    }

    /// Helpers - mostly for internal use
    fn verify_prefix(reader: &mut CellParser) -> Result<(), TLCoreError> {
        if Self::PREFIX == TLBPrefix::NULL {
            return Ok(());
        }

        let prefix_error = |given, bits_left| {
            Err(TLCoreError::TLBWrongPrefix {
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

    fn write_prefix(builder: &mut CellBuilder) -> Result<(), TLCoreError> {
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
