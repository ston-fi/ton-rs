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
                log::warn!("Fail to read metadata prefix: {e}");
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
            log::warn!("Fail to parse metadata: {err}");
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

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use ton_core::traits::tlb::TLB;

    use crate::tep::metadata::MetadataContent;
    use log::LevelFilter;
    use log4rs::Config;
    use log4rs::append::console::{ConsoleAppender, Target};
    use log4rs::config::{Appender, Root};
    use std::sync::Once;

    static LOG: Once = Once::new();
    pub fn init_logging() {
        LOG.call_once(|| {
            let stderr = ConsoleAppender::builder()
                .target(Target::Stderr)
                .encoder(Box::new(log4rs::encode::pattern::PatternEncoder::new(
                    "{d(%Y-%m-%d %H:%M:%S%.6f)} {T:>15.15} {h({l:>5.5})} {t}:{L} - {m}{n}",
                )))
                .build();

            let config = Config::builder()
                .appender(Appender::builder().build("stderr", Box::new(stderr)))
                .build(Root::builder().appender("stderr").build(LevelFilter::Info))
                .unwrap();

            log4rs::init_config(config).unwrap();
        })
    }

    #[test]
    fn byte_unaligned_content() -> Result<()> {
        init_logging();
        let content = MetadataContent::from_boc_hex(
            "b5ee9c7201010a0100d400010300c00102012002060143bff872ebdb514d9c97c283b7f0ae5179029e2b6119c39462719e4f46ed8f7413e6400301050000c00402012005060143bff872ebdb514d9c97c283b7f0ae5179029e2b6119c39462719e4f46ed8f7413e640070143bff7407e978f01a40711411b1acb773a96bdd93fa83bb5ca8435013c8c4b3ac91f400901020008008c68747470733a2f2f6a736f6e626c6f622e636f6d2f6170692f6a736f6e426c6f622f30313964666366342d333764322d376139332d393333392d31303131303138356365333000040039",
        )?;

        dbg!(content);
        Ok(())
    }
}
