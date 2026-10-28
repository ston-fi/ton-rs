use crate::bail_ton;
use crate::block_tlb::{BlockIdExt, ShardIdent, TVMStackValue, TVMTuple};
use crate::errors::TonResult;
use crate::ton_core::errors::TonCoreError;
use fastnum::I512;
use std::ops::Deref;
use std::sync::Arc;
use ton_core::bail_ton_core_data;
use ton_core::cell::{CellBuilder, CellParser, TonHash};
use ton_core::errors::TonCoreResult;
use ton_core::traits::tlb::TLB;

// 13th element of c7 register
// https://docs.ton.org/v3/documentation/tvm/changelog/tvm-upgrade-2023-07#opcodes-to-work-with-new-c7-values
#[derive(Debug, PartialEq)]
pub struct C7PrevBlocksInfo {
    pub last_mc_blocks: Arc<Vec<BlockIdExt>>,
    pub prev_key_block: BlockIdExt,
    pub last_mc_blocks_div_100: Arc<Vec<BlockIdExt>>,
}

// [ wc:Integer shard:Integer seqno:Integer root_hash:Integer file_hash:Integer ] = BlockId;
// [ last_mc_blocks:BlockId[] prev_key_block:BlockId last_mc_blocks_divisible_by_100:BlockId ] = PrevBlocksInfo;
impl TLB for C7PrevBlocksInfo {
    fn read_definition(parser: &mut CellParser) -> TonCoreResult<Self> {
        let main_tuple = TVMTuple::read(parser)?;
        if main_tuple.len() != 3 {
            bail_ton_core_data!("Expected PrevBlocksInfo tuple of length 3, found {}", main_tuple.len());
        }

        let last_mc_blocks_tuple = main_tuple.get_tuple(0)?;
        let mut last_mc_blocks = Vec::with_capacity(last_mc_blocks_tuple.len());
        for value in last_mc_blocks_tuple.deref() {
            last_mc_blocks.push(block_id_from_stack_value(value)?);
        }

        let prev_key_block = block_id_from_stack_value(main_tuple.get(1).unwrap())?;

        let last_mc_blocks_div_100_tuple = main_tuple.get_tuple(2)?;
        let mut last_mc_blocks_div_100 = Vec::with_capacity(last_mc_blocks_div_100_tuple.len());
        for value in last_mc_blocks_div_100_tuple.deref() {
            last_mc_blocks_div_100.push(block_id_from_stack_value(value)?);
        }

        Ok(C7PrevBlocksInfo {
            last_mc_blocks: Arc::new(last_mc_blocks),
            prev_key_block,
            last_mc_blocks_div_100: Arc::new(last_mc_blocks_div_100),
        })
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> TonCoreResult<()> {
        let mut last_mc_blocks_tuple = TVMTuple::default();
        for block in self.last_mc_blocks.deref() {
            last_mc_blocks_tuple.push_tuple(block_id_to_tuple(block));
        }

        let prev_key_block_tuple = block_id_to_tuple(&self.prev_key_block);

        let mut last_mc_blocks_div_100_tuple = TVMTuple::default();
        for block in self.last_mc_blocks_div_100.deref() {
            last_mc_blocks_div_100_tuple.push_tuple(block_id_to_tuple(block));
        }

        let mut main_tuple = TVMTuple::default();
        main_tuple.push_tuple(last_mc_blocks_tuple);
        main_tuple.push_tuple(prev_key_block_tuple);
        main_tuple.push_tuple(last_mc_blocks_div_100_tuple);
        main_tuple.write(builder)
    }
}

fn block_id_to_tuple(block: &BlockIdExt) -> TVMTuple {
    let mut tuple = TVMTuple::default();
    tuple.push_int(I512::from_i32(block.shard_ident.workchain));
    tuple.push_int(I512::from_u64(block.shard_ident.shard).unwrap());
    tuple.push_int(I512::from_u32(block.seqno));
    tuple.push_int(block.root_hash.to_i512());
    tuple.push_int(block.file_hash.to_i512());
    tuple
}

fn block_id_from_stack_value(value: &TVMStackValue) -> TonResult<BlockIdExt> {
    let TVMStackValue::Tuple(tuple) = value else {
        bail_ton!("Expected Tuple for BlockIdExt, found {value:?}");
    };
    let Ok(workchain) = tuple.get_int(0)?.to_i32() else {
        bail_ton!("Can't convert {} to i32", tuple.get_int(0)?);
    };
    let shard = tuple.get_int(1)?.to_u64().unwrap();
    let seqno = tuple.get_int(2)?.to_u32().unwrap();
    let root_hash = TonHash::from_i512(tuple.get_int(3)?)?;
    let file_hash = TonHash::from_i512(tuple.get_int(4)?)?;

    Ok(BlockIdExt {
        shard_ident: ShardIdent { workchain, shard },
        seqno,
        root_hash,
        file_hash,
    })
}

#[cfg(test)]
#[path = "../../tests/emulators/tx_emulator/test_c7_prev_blocks_info.rs"]
mod tests;
