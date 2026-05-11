use crate::block_tlb::{FromTVMStack, TVMStack};
use crate::errors::TonResult;
use crate::tep::snake_data::SnakeData;
use crate::tlb_adapters::DictKeyAdapterTonHash;
use crate::tlb_adapters::DictValAdapterTLB;
use crate::tlb_adapters::TLBHashMapE;
use std::collections::HashMap;
use std::fmt::Debug;
use ton_core::TLB;
use ton_core::cell::{CellBuilder, CellParser, TonCell, TonHash};
use ton_core::errors::TonCoreResult;
use ton_core::traits::tlb::TLB;
use ton_core::types::tlb_core::TLBRef;

pub type MetadataDict = HashMap<TonHash, TLBRef<SnakeData>>;

#[derive(PartialEq, Debug, Clone)]
pub enum MetadataContent {
    Internal(MetadataInternal),
    External(MetadataExternal),
    Unsupported(MetadataUnsupported),
}

#[derive(PartialEq, Debug, Clone, TLB)]
#[tlb(prefix = 0x0, bits_len = 8)]
pub struct MetadataInternal {
    #[tlb(adapter = "TLBHashMapE::<DictKeyAdapterTonHash, DictValAdapterTLB<_>>::new(256)")]
    pub data: MetadataDict,
}

#[derive(PartialEq, Eq, Debug, Clone, TLB)]
#[tlb(prefix = 0x1, bits_len = 8)]
pub struct MetadataExternal {
    pub uri: SnakeData,
}

#[derive(PartialEq, Eq, Debug, Clone, TLB)]
pub struct MetadataUnsupported {
    pub cell: TonCell,
}

impl FromTVMStack for MetadataContent {
    fn from_stack(stack: &mut TVMStack) -> TonResult<Self> { Ok(MetadataContent::from_cell(&stack.pop_cell()?)?) }
}

impl TLB for MetadataContent {
    fn read_definition(parser: &mut CellParser) -> Result<Self, ton_core::errors::TonCoreError> {
        let mut original_parser = parser.clone();
        let prefix: u8 = match parser.read_num(8) {
            Ok(x) => x,
            Err(e) => {
                log::debug!("Fail to read metadata prefix: {e}");
                let unsupported = MetadataUnsupported {
                    cell: original_parser.read_remaining()?,
                };
                return Ok(unsupported.into());
            }
        };
        parser.seek_bits(-8)?;
        let parsed: TonCoreResult<MetadataContent> = match prefix {
            0 => MetadataInternal::read(parser).map(Into::into),
            1 => MetadataExternal::read(parser).map(Into::into),
            _ => MetadataUnsupported::read(parser).map(Into::into),
        };

        if let Err(err) = &parsed {
            log::debug!("Fail to parse metadata: {err}");
            let unsupported = MetadataUnsupported {
                cell: original_parser.read_remaining()?,
            };
            return Ok(unsupported.into());
        }
        parsed
    }
    fn write_definition(&self, builder: &mut CellBuilder) -> TonCoreResult<()> {
        match self {
            Self::Internal(value) => value.write(builder)?,
            Self::External(value) => value.write(builder)?,
            Self::Unsupported(value) => value.write(builder)?,
        }
        Ok(())
    }
}

impl MetadataContent {
    pub fn as_internal(&self) -> Option<&MetadataInternal> {
        match self {
            MetadataContent::Internal(inner) => Some(inner),
            _ => None,
        }
    }
    pub fn as_internal_mut(&mut self) -> Option<&mut MetadataInternal> {
        match self {
            MetadataContent::Internal(inner) => Some(inner),
            _ => None,
        }
    }
    pub fn into_internal(self) -> Option<MetadataInternal> {
        match self {
            MetadataContent::Internal(inner) => Some(inner),
            _ => None,
        }
    }
    pub fn as_external(&self) -> Option<&MetadataExternal> {
        match self {
            MetadataContent::External(inner) => Some(inner),
            _ => None,
        }
    }
    pub fn as_external_mut(&mut self) -> Option<&mut MetadataExternal> {
        match self {
            MetadataContent::External(inner) => Some(inner),
            _ => None,
        }
    }
    pub fn into_external(self) -> Option<MetadataExternal> {
        match self {
            MetadataContent::External(inner) => Some(inner),
            _ => None,
        }
    }
    pub fn as_unsupported(&self) -> Option<&MetadataUnsupported> {
        match self {
            MetadataContent::Unsupported(inner) => Some(inner),
            _ => None,
        }
    }
    pub fn as_unsupported_mut(&mut self) -> Option<&mut MetadataUnsupported> {
        match self {
            MetadataContent::Unsupported(inner) => Some(inner),
            _ => None,
        }
    }
    pub fn into_unsupported(self) -> Option<MetadataUnsupported> {
        match self {
            MetadataContent::Unsupported(inner) => Some(inner),
            _ => None,
        }
    }
}
impl From<MetadataInternal> for MetadataContent {
    fn from(v: MetadataInternal) -> Self { MetadataContent::Internal(v) }
}
impl From<MetadataExternal> for MetadataContent {
    fn from(v: MetadataExternal) -> Self { MetadataContent::External(v) }
}
impl From<MetadataUnsupported> for MetadataContent {
    fn from(v: MetadataUnsupported) -> Self { MetadataContent::Unsupported(v) }
}
