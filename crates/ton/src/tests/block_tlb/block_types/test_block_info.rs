use super::*;
use crate::block_tlb::_test_block_data::{MASTER_BLOCK_BOC_HEX, SHARD_BLOCK_BOC_HEX};
use crate::block_tlb::Block;
use std::ops::Deref;
use std::str::FromStr;

#[test]
fn test_block_tlb_block_info_master_key_block() -> anyhow::Result<()> {
    let block = Block::from_boc_hex(MASTER_BLOCK_BOC_HEX)?;
    let parsed_block_info = block.info;
    let expected = BlockInfo {
        version: 0,
        not_master: false,
        after_merge: false,
        before_split: false,
        after_split: false,
        want_split: false,
        want_merge: true,
        key_block: true,
        vert_seqno_incr: false,
        flags: 1,
        seqno: 46991999,
        vert_seqno: 1,
        shard: ShardIdent {
            workchain: -1,
            shard: 0x8000000000000000,
        },
        gen_utime: 1745112841,
        start_lt: 56255102000000,
        end_lt: 56255102000004,
        gen_validator_list_has_short: 4143742061,
        gen_catchain_seqno: 682531,
        min_ref_mc_seqno: 46991995,
        prev_key_block_seqno: 46989053,
        gen_software: Some(GlobalVersion {
            version: 10,
            capabilities: 494,
        }),
        master_ref: None,
        prev_ref: PrevBlockInfo::Regular(ExtBlockRef {
            end_lt: 56255101000004,
            seqno: 46991998,
            root_hash: TonHash::from_str("A16DD643A1B54A6804CE3264503D9FEAB4E0F5D1DE450888F188179557093595")?,
            file_hash: TonHash::from_str("2E58DCF8FE16CCC203DDD1D053984F9AC6EAFCF0543CED95F96AB9E7E411D256")?,
        }),
        prev_vert_ref: None,
    };
    assert_eq!(parsed_block_info.deref(), &expected);

    let parsed_back = BlockInfo::from_boc(parsed_block_info.to_boc()?)?;
    assert_eq!(parsed_block_info.deref(), &parsed_back);
    assert_eq!(parsed_block_info.cell_hash()?, parsed_back.cell_hash()?);
    Ok(())
}

#[test]
fn test_block_tlb_block_info_shard() -> anyhow::Result<()> {
    let block = Block::from_boc_hex(SHARD_BLOCK_BOC_HEX)?;
    let parsed_block_info = block.info;
    let expected = BlockInfo {
        version: 0,
        not_master: true,
        after_merge: false,
        before_split: false,
        after_split: false,
        want_split: false,
        want_merge: true,
        key_block: false,
        vert_seqno_incr: false,
        flags: 1,
        seqno: 52111590,
        vert_seqno: 1,
        shard: ShardIdent {
            workchain: 0,
            shard: 0x6000000000000000,
        },
        gen_utime: 1745147839,
        start_lt: 56269616000000,
        end_lt: 56269616000011,
        gen_validator_list_has_short: 3574137325,
        gen_catchain_seqno: 684420,
        min_ref_mc_seqno: 47004578,
        prev_key_block_seqno: 46991999,
        gen_software: Some(GlobalVersion {
            version: 10,
            capabilities: 494,
        }),
        master_ref: Some(ExtBlockRef {
            end_lt: 56269615000004,
            seqno: 47004578,
            root_hash: TonHash::from_str("B94923821E89A231F697F1434CBF428DCF999FF7E28B468D1CE155EDAD94B019")?,
            file_hash: TonHash::from_str("256B239CC5B7984A4F453E0F619F34F75FF42EB50B5FF67181D1C9779E765197")?,
        }),
        prev_ref: PrevBlockInfo::Regular(ExtBlockRef {
            end_lt: 56269615000010,
            seqno: 52111589,
            root_hash: TonHash::from_str("E05BBE4312F8B110287CCA5A928458778E5DD68F935AF3FD0051D33287EFCD6D")?,
            file_hash: TonHash::from_str("A00B27433D3D2B05C9746F5C73C4C087C7921F1A867FE7ACABEBF0C6668F8D3C")?,
        }),
        prev_vert_ref: None,
    };
    assert_eq!(parsed_block_info.deref(), &expected);
    let parsed_back = BlockInfo::from_boc(parsed_block_info.to_boc()?)?;
    assert_eq!(parsed_block_info.deref(), &parsed_back);
    assert_eq!(parsed_block_info.cell_hash()?, parsed_back.cell_hash()?);
    Ok(())
}

#[test]
fn test_block_tlb_block_info_prev_block_ids() -> anyhow::Result<()> {
    let mut block_info = BlockInfo::default();
    block_info.shard.workchain = 0;
    block_info.shard.shard = 0x8000000000000000;
    block_info.prev_ref = PrevBlockInfo::AfterMerge(BlockPrevInfoAfterMerge {
        prev1: ExtBlockRef {
            end_lt: 1,
            seqno: 2,
            root_hash: TonHash::from_slice_sized(&[3u8; 32]),
            file_hash: TonHash::from_slice_sized(&[4u8; 32]),
        }
        .into(),
        prev2: ExtBlockRef {
            end_lt: 5,
            seqno: 6,
            root_hash: TonHash::from_slice_sized(&[7u8; 32]),
            file_hash: TonHash::from_slice_sized(&[8u8; 32]),
        }
        .into(),
    });
    let prev_block_ids = block_info.prev_block_ids()?;
    assert_eq!(prev_block_ids.len(), 2);
    assert_eq!(
        prev_block_ids[0],
        BlockIdExt {
            shard_ident: ShardIdent {
                workchain: 0,
                shard: 0x4000000000000000,
            },
            seqno: 2,
            root_hash: TonHash::from_slice_sized(&[3u8; 32]),
            file_hash: TonHash::from_slice_sized(&[4u8; 32]),
        }
    );
    assert_eq!(
        prev_block_ids[1],
        BlockIdExt {
            shard_ident: ShardIdent {
                workchain: 0,
                shard: 0xc000000000000000,
            },
            seqno: 6,
            root_hash: TonHash::from_slice_sized(&[7u8; 32]),
            file_hash: TonHash::from_slice_sized(&[8u8; 32]),
        }
    );

    Ok(())
}

#[test]
fn test_block_tlb_block_info_prev_block_ids_after_split_uses_parent_shard() -> anyhow::Result<()> {
    let mut block_info = BlockInfo::default();
    block_info.shard.workchain = 0;
    block_info.shard.shard = 0x6000000000000000;
    block_info.after_split = true;
    block_info.prev_ref = PrevBlockInfo::Regular(ExtBlockRef {
        end_lt: 1,
        seqno: 2,
        root_hash: TonHash::from_slice_sized(&[3u8; 32]),
        file_hash: TonHash::from_slice_sized(&[4u8; 32]),
    });

    let prev_block_ids = block_info.prev_block_ids()?;
    assert_eq!(
        prev_block_ids,
        vec![BlockIdExt {
            shard_ident: ShardIdent {
                workchain: 0,
                shard: 0x4000000000000000,
            },
            seqno: 2,
            root_hash: TonHash::from_slice_sized(&[3u8; 32]),
            file_hash: TonHash::from_slice_sized(&[4u8; 32]),
        }]
    );

    Ok(())
}

#[test]
fn test_block_tlb_block_info_prev_block_ids_rejects_split_after_genesis() {
    let mut block_info = BlockInfo::default();
    block_info.shard.workchain = 0;
    block_info.shard.shard = 0x6000000000000000;
    block_info.after_split = true;
    block_info.prev_ref = PrevBlockInfo::Regular(ExtBlockRef {
        end_lt: 1,
        seqno: 0,
        root_hash: TonHash::from_slice_sized(&[3u8; 32]),
        file_hash: TonHash::from_slice_sized(&[4u8; 32]),
    });

    assert!(block_info.prev_block_ids().is_err());
}

#[test]
fn test_block_tlb_block_info_prev_block_ids_rejects_split_and_merge_at_once() {
    let mut block_info = BlockInfo::default();
    block_info.shard.workchain = 0;
    block_info.shard.shard = 0x8000000000000000;
    block_info.after_split = true;
    block_info.prev_ref = PrevBlockInfo::AfterMerge(BlockPrevInfoAfterMerge {
        prev1: ExtBlockRef {
            end_lt: 1,
            seqno: 2,
            root_hash: TonHash::from_slice_sized(&[3u8; 32]),
            file_hash: TonHash::from_slice_sized(&[4u8; 32]),
        }
        .into(),
        prev2: ExtBlockRef {
            end_lt: 5,
            seqno: 6,
            root_hash: TonHash::from_slice_sized(&[7u8; 32]),
            file_hash: TonHash::from_slice_sized(&[8u8; 32]),
        }
        .into(),
    });

    assert!(block_info.prev_block_ids().is_err());
}

#[test]
fn test_block_tlb_block_info_prev_block_ids_rejects_merge_after_genesis() {
    let mut block_info = BlockInfo::default();
    block_info.shard.workchain = 0;
    block_info.shard.shard = 0x8000000000000000;
    block_info.prev_ref = PrevBlockInfo::AfterMerge(BlockPrevInfoAfterMerge {
        prev1: ExtBlockRef {
            end_lt: 1,
            seqno: 0,
            root_hash: TonHash::from_slice_sized(&[3u8; 32]),
            file_hash: TonHash::from_slice_sized(&[4u8; 32]),
        }
        .into(),
        prev2: ExtBlockRef {
            end_lt: 5,
            seqno: 6,
            root_hash: TonHash::from_slice_sized(&[7u8; 32]),
            file_hash: TonHash::from_slice_sized(&[8u8; 32]),
        }
        .into(),
    });

    assert!(block_info.prev_block_ids().is_err());
}
