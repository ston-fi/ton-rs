use super::*;
use std::str::FromStr;
use ton_core::cell::TonCell;
use ton_core::types::TonAddress;

#[test]
fn test_block_tlb_shard_ident_master() -> anyhow::Result<()> {
    // master_block: shard:(shard_ident shard_pfx_bits:0 workchain_id:-1 shard_prefix:0)
    // https://explorer.toncoin.org/viewblock?workchain=-1&shard=8000000000000000&seqno=46991999&roothash=CBEBAA6AC4270C987C90C5ED930FF37F9B73C705999585D6D8C1C5E9FA3DD6E3&filehash=A660FB144506617B0C0132B92CF41E436A6A87E924FD97C769C5D6E36320327B
    let mut builder = TonCell::builder();
    builder.write_num(&0u32, 2)?; // ShardIdent prefix
    builder.write_num(&0u32, 6)?; // shard_pfx_bits
    builder.write_num(&TON_MASTERCHAIN, 32)?; // workchain
    builder.write_num(&0u64, 64)?; // shard_prefix
    let mc_shard_ident_cell = builder.build()?;
    let mc_shard_ident_parsed = ShardIdent::from_cell(&mc_shard_ident_cell)?;
    assert_eq!(mc_shard_ident_parsed.workchain, TON_MASTERCHAIN);
    assert_eq!(mc_shard_ident_parsed.shard, TON_SHARD_FULL);
    let mc_shard_ident_cell_serial = mc_shard_ident_parsed.to_cell()?;
    assert_eq!(mc_shard_ident_cell, mc_shard_ident_cell_serial);
    Ok(())
}

#[test]
fn test_block_tlb_shard_ident_shard() -> anyhow::Result<()> {
    // shard_block: shard:(shard_ident shard_pfx_bits:2 workchain_id:0 shard_prefix:4611686018427387904)
    // https://explorer.toncoin.org/viewblock?workchain=0&shard=6000000000000000&seqno=52111590&roothash=D350895E85FFD081F564E5D138F374A9B52B53AEE0035B07CE5A5D6388B73B45&filehash=16944A89B7DC24BCF46AC434D5E29717B0905FDBCB523FACAFE40547BF2E7DB9
    let mut builder = TonCell::builder();
    builder.write_num(&0u32, 2)?; // ShardIdent prefix
    builder.write_num(&2u32, 6)?; // shard_pfx_bits
    builder.write_num(&0i32, 32)?; // workchain
    builder.write_num(&4611686018427387904u64, 64)?; // shard_prefix, 0x4000000000000000u64
    let shard_shard_ident_cell = builder.build()?;
    let mc_shard_ident_parsed = ShardIdent::from_cell(&shard_shard_ident_cell)?;
    assert_eq!(mc_shard_ident_parsed.workchain, 0);
    assert_eq!(mc_shard_ident_parsed.shard, 0x6000000000000000);
    let mc_shard_ident_cell_serial = mc_shard_ident_parsed.to_cell()?;
    assert_eq!(shard_shard_ident_cell, mc_shard_ident_cell_serial);
    Ok(())
}

#[test]
fn test_block_tlb_shard_ident_split_merge() -> anyhow::Result<()> {
    let shard_ident = ShardIdent::new(0, 0x8000000000000000);
    let (left, right) = shard_ident.split()?;
    assert_eq!(left.workchain, 0);
    assert_eq!(left.shard, 0x4000000000000000);
    assert_eq!(right.workchain, 0);
    assert_eq!(right.shard, 0xC000000000000000);

    let (left2, right2) = left.split()?;
    assert_eq!(left2.shard, 0x2000000000000000);
    assert_eq!(right2.shard, 0x6000000000000000);

    let (left3, right3) = right.split()?;
    assert_eq!(left3.shard, 0xa000000000000000);
    assert_eq!(right3.shard, 0xe000000000000000);

    assert_eq!(left3.merge()?, right);
    assert_eq!(right3.merge()?, right);

    assert_eq!(left2.merge()?, left);
    assert_eq!(right2.merge()?, left);

    assert_eq!(left.merge()?, shard_ident);
    assert_eq!(right.merge()?, shard_ident);
    Ok(())
}

#[test]
fn test_block_tlb_shard_ident_prefix_len() {
    let test_cases = [
        (0x8000_0000_0000_0000, 0),
        (0x4000_0000_0000_0000, 1),
        (0xC000_0000_0000_0000, 1),
        (0x6000_0000_0000_0000, 2),
        (0x8, 60),
    ];

    for (shard, expected_len) in test_cases {
        assert_eq!(ShardIdent::new(0, shard).prefix_len(), expected_len, "shard 0x{shard:016X}");
    }
}

#[test]
fn test_block_tlb_shard_ident_split_rejects_max_depth() {
    let deepest_shard = ShardIdent::new(0, 0x8);
    assert!(deepest_shard.split().is_err());
}

#[test]
fn test_block_tlb_shard_ident_merge_rejects_full_shard() {
    let full_shard = ShardIdent::new(0, TON_SHARD_FULL);
    assert!(full_shard.merge().is_err());
}

#[test]
fn test_shard_ident_contains_addr() -> anyhow::Result<()> {
    let addr = TonAddress::from_str("EQDc_nrm5oOVCVQM8GRJ5q_hr1jgpNQjsGkIGE-uztt26_Ep")?;

    let shard_ident = ShardIdent::new(0, 0x8000000000000000);
    assert!(shard_ident.contains_addr(&addr.to_msg_address_int()));
    let (left, right) = shard_ident.split()?;
    assert!(!left.contains_addr(&addr.to_msg_address_int()), "shard: {left}, addr: {}", addr.to_hex());
    assert!(right.contains_addr(&addr.to_msg_address_int()), "shard: {right}, addr: {}", addr.to_hex());
    Ok(())
}
