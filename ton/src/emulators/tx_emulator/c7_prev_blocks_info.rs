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
    pub last_mc_block_divided_100: BlockIdExt,
}

// [ wc:Integer shard:Integer seqno:Integer root_hash:Integer file_hash:Integer ] = BlockId;
// [ last_mc_blocks:BlockId[] prev_key_block:BlockId last_mc_blocks_divisible_by_100:BlockId ] = PrevBlocksInfo;
impl TLB for C7PrevBlocksInfo {
    fn read_definition(parser: &mut CellParser) -> TonCoreResult<Self> {
        let main_tuple = TVMTuple::read(parser)?;
        if main_tuple.len() != 3 {
            bail_ton_core_data!("Expected PrevBlocksInfo tuple of length 2, found {}", main_tuple.len());
        }
        let last_mc_blocks_tuple = main_tuple.get_tuple(0)?;
        let mut last_mc_blocks = Vec::with_capacity(last_mc_blocks_tuple.len());
        for value in last_mc_blocks_tuple.deref() {
            last_mc_blocks.push(block_id_from_stack_value(value)?);
        }
        let prev_key_block = block_id_from_stack_value(main_tuple.get(1).unwrap())?;
        let last_mc_block_divided_100 = block_id_from_stack_value(main_tuple.get(2).unwrap())?;
        Ok(C7PrevBlocksInfo {
            last_mc_blocks: Arc::new(last_mc_blocks),
            prev_key_block,
            last_mc_block_divided_100,
        })
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> TonCoreResult<()> {
        let mut prev_blocks = TVMTuple::default();
        for block in self.last_mc_blocks.deref() {
            prev_blocks.push_tuple(block_id_to_tuple(block));
        }

        let prev_key_block = block_id_to_tuple(&self.prev_key_block);
        let last_mc_block_divided_100 = block_id_to_tuple(&self.last_mc_block_divided_100);

        let mut main_tuple = TVMTuple::default();
        main_tuple.push_tuple(prev_blocks);
        main_tuple.push_tuple(prev_key_block);
        main_tuple.push_tuple(last_mc_block_divided_100);
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
mod tests {
    use crate::emulators::tx_emulator::C7PrevBlocksInfo;
    use std::ops::Deref;
    use ton_core::traits::tlb::TLB;

    #[test]
    fn test_c7_prev_blocks_info_serde() -> anyhow::Result<()> {
        let b64 = "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";
        let parsed = C7PrevBlocksInfo::from_boc_base64(&b64)?;
        assert_eq!(parsed.last_mc_blocks.len(), 16);
        let mut cur_seqno = 55182463u32;
        for block in parsed.last_mc_blocks.deref() {
            assert_eq!(block.seqno, cur_seqno);
            cur_seqno += 1;
        }

        let serialized = parsed.to_boc_base64()?;
        let parsed_back = C7PrevBlocksInfo::from_boc_base64(&serialized)?;
        assert_eq!(parsed, parsed_back);
        Ok(())
    }
}
