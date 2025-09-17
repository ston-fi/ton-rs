use crate::block_tlb::block_types::block_id_ext::BlockIdExt;
use crate::block_tlb::ShardDescr;
use crate::block_tlb::{ConfigParams, CurrencyCollection};
use crate::block_tlb::{ShardIdent, ShardPfx};
use crate::tlb_adapters::{BinTree, DictKeyAdapterInto, DictValAdapterTLB, TLBHashMapE};
use std::collections::HashMap;
use ton_lib_core::cell::{CellBuilder, CellParser, TonCell, TonCellRef, TonHash};
use ton_lib_core::errors::TonCoreError;
use ton_lib_core::traits::tlb::{TLBPrefix, TLB};
use ton_lib_core::{bail_ton_core_data, TLB};

// https://github.com/ton-blockchain/ton/blame/6f745c04daf8861bb1791cffce6edb1beec62204/crypto/block/block.tlb#L593
#[derive(Debug, Clone, PartialEq)]
pub struct MCBlockExtra {
    pub key_block: bool,
    pub shard_hashes: HashMap<i32, HashMap<ShardPfx, ShardDescr>>, // wc_id -> shard_pfx -> ShardDescr
    pub shard_fees: Option<TonCellRef>,
    shard_fees_crated: ShardFeesCreated, // this is a mock to read/write cell properly while we don't support a fair HashmapAugE
    // https://github.com/ton-blockchain/ton/blob/6f745c04daf8861bb1791cffce6edb1beec62204/crypto/block/block.tlb#L597
    pub ref_data: TonCellRef,
    pub config: Option<ConfigParams>,
}

impl MCBlockExtra {
    pub fn shard_ids(&self) -> Vec<BlockIdExt> {
        let mut shard_ids = vec![];
        for (wc, shards) in &self.shard_hashes {
            for (shard_pfx, descr) in shards {
                shard_ids.push(BlockIdExt {
                    shard_ident: ShardIdent::from_pfx(*wc, shard_pfx),
                    seqno: descr.seqno,
                    root_hash: TonHash::from_slice_sized(descr.root_hash.as_slice_sized()),
                    file_hash: TonHash::from_slice_sized(descr.file_hash.as_slice_sized()),
                });
            }
        }
        shard_ids
    }
}

#[derive(Debug, Clone, PartialEq, TLB)]
pub struct ShardFeesCreated {
    pub fees: CurrencyCollection,
    pub create: CurrencyCollection,
}

impl TLB for MCBlockExtra {
    const PREFIX: TLBPrefix = TLBPrefix::new(0xcca5, 16);
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        let key_block = TLB::read(parser)?;
        let shards_dict = TLBHashMapE::<DictKeyAdapterInto, DictValAdapterTLB, u32, TonCellRef>::new(32);
        let mut shard_hashes = HashMap::new();
        for (wc_id, cell_ref) in shards_dict.read(parser)? {
            let cur_hashes = BinTree::<DictValAdapterTLB, _>::read(&mut cell_ref.parser())?;
            shard_hashes.insert(wc_id as i32, cur_hashes);
        }
        let shard_fees = TLB::read(parser)?;
        let shard_fees_crated = TLB::read(parser)?;
        let ref_data = TLB::read(parser)?;

        let config = match key_block {
            true => Some(TLB::read(parser)?),
            false => None,
        };
        Ok(Self {
            key_block,
            shard_hashes,
            shard_fees,
            shard_fees_crated,
            ref_data,
            config,
        })
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonCoreError> {
        self.key_block.write(builder)?;
        let mut shards_dict = HashMap::<u32, TonCellRef>::new();
        for (wc_id, shards) in &self.shard_hashes {
            let mut val_builder = TonCell::builder();
            BinTree::<DictValAdapterTLB, _>::write(&mut val_builder, shards)?;
            shards_dict.insert(*wc_id as u32, val_builder.build_ref()?);
        }
        TLBHashMapE::<DictKeyAdapterInto, DictValAdapterTLB, _, _>::new(32).write(builder, &shards_dict)?;
        self.shard_fees.write(builder)?;
        self.shard_fees_crated.write(builder)?;
        self.ref_data.write(builder)?;
        if self.key_block {
            match &self.config {
                Some(config) => config.write(builder)?,
                None => bail_ton_core_data!("key_block=true, but config_types is None"),
            }
        }
        Ok(())
    }
}
