use crate::block_tlb::BlockExtra;
use crate::block_tlb::BlockInfo;
use crate::tlb_adapters::TLBRef;
use ton_lib_core::cell::TonCellRef;
use ton_lib_core::TLB;

// https://github.com/ton-blockchain/ton/blob/6f745c04daf8861bb1791cffce6edb1beec62204/crypto/block/block.tlb#L462
#[derive(Debug, Clone, PartialEq, TLB)]
#[tlb(prefix = 0x11ef55aa, bits_len = 32)]
pub struct Block {
    pub global_id: i32,
    #[tlb(adapter = "TLBRef")]
    pub info: BlockInfo,
    pub value_flow: TonCellRef,   // TODO
    pub state_update: TonCellRef, // TODO
    #[tlb(adapter = "TLBRef")]
    pub extra: BlockExtra,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use crate::block_tlb::block_types::block_info::ExtBlockRef;
    use crate::block_tlb::GlobalVersion;
    use crate::block_tlb::ShardIdent;
    use crate::block_tlb::_test_block_data::MASTER_BLOCK_BOC_HEX;
    use std::str::FromStr;
    use tokio_test::assert_ok;
    use ton_lib_core::cell::TonHash;
    use ton_lib_core::traits::tlb::TLB;

    #[test]
    fn test_block_tlb_block() -> anyhow::Result<()> {
        let parsed = Block::from_boc_hex(MASTER_BLOCK_BOC_HEX)?;
        assert_eq!(
            parsed.extra.mc_block_extra.as_ref().unwrap().cell_hash()?,
            TonHash::from_str("D0D3EA6B963ABEB1C5E2F9936DB6DA8ADDB1DF1F221F1F13C85120DE0BB79DA0")?
        );
        assert_eq!(
            parsed.cell_hash()?,
            TonHash::from_str("CBEBAA6AC4270C987C90C5ED930FF37F9B73C705999585D6D8C1C5E9FA3DD6E3")?
        );
        // test block.info
        let expected_block_info = BlockInfo {
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
            prev_ref: ExtBlockRef {
                end_lt: 56255101000004,
                seqno: 46991998,
                root_hash: TonHash::from_str("a16dd643a1b54a6804ce3264503d9feab4e0f5d1de450888f188179557093595")?,
                file_hash: TonHash::from_str("2e58dcf8fe16ccc203ddd1d053984f9ac6eafcf0543ced95f96ab9e7e411d256")?,
            }
            .into(),

            prev_vert_ref: None,
        };
        assert_eq!(expected_block_info, parsed.info);

        assert!(parsed.extra.mc_block_extra.is_some());

        // test block.extra.mc_block_extra.shard_hashes
        let expected_shards = HashMap::from([
            (0x2000000000000000u64, 52077744),
            (0x6000000000000000, 52097945),
            (0xa000000000000000, 51731388),
            (0xe000000000000000, 51757085),
        ]);
        let parsed_shard_hashes = &parsed.extra.mc_block_extra.as_ref().unwrap().shard_hashes;
        assert_eq!(parsed_shard_hashes.len(), 1);
        assert!(parsed_shard_hashes.contains_key(&0));
        let parsed_shard_descr = parsed_shard_hashes.get(&0).unwrap();
        assert_eq!(parsed_shard_descr.len(), expected_shards.len());
        for (shard_pfx, descr) in parsed_shard_descr {
            let shard = shard_pfx.to_shard();
            assert!(expected_shards.contains_key(&shard), "shard: {shard:X}, shard_pfx: {shard_pfx:?}");
            let expected_seqno = expected_shards.get(&shard).unwrap();
            assert_eq!(*expected_seqno, descr.seqno);
        }

        // full serialization test
        let serialized = parsed.to_boc()?;
        let parsed_back = Block::from_boc(&serialized)?;
        assert_eq!(parsed_back, parsed);
        assert_eq!(
            parsed_back.cell_hash()?,
            TonHash::from_str("CBEBAA6AC4270C987C90C5ED930FF37F9B73C705999585D6D8C1C5E9FA3DD6E3")?
        );
        Ok(())
    }

    #[test]
    fn test_block_with_shard_descr_future_split_merge() -> anyhow::Result<()> {
        assert_ok!(Block::from_boc_hex("b5ee9c7201022c0100062400041011ef55aaffffff112a24220104894a33f6fd44497b4bdc346b40844af13fa021d22f55dddebe8848a28f3c041923df3540fc4d24df2bcf448907d602ef189f3fff31ae9832b5704bf48fd4774e0630e82e20c0212020021317d23c0cdd2dedd74ab696f071c39836a85bd321a55cd3f48db8d26b36149a75fe0005cca569be9b40ec44a817c804140d03010150040201610b0503af7333333333333333333333333333333333333333333333333333333333333333300003592345ce3027beb11aad9d710be57e281aaa9cf9f9929d28801b632f951516e9fb854806baf00003592345ce3016862c66900014080a0906020f04093e8edfded8110807005bc00000000000000000000000012d452da449e50b8cf7dd27861f146122afe1b546bb8b70fc8216f0c614139f8e0400a042af7010b0760000000000000000006400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008272b4422d651e1fb4a5b4c6fe2d861119ef0c01585641229c46ef0824f4cae4f70bdef199700b69dd90a5888117241b6f3fc10cdba22ff9735d63a6ce688fdbd4300101a00c01064606000c00ab69fe00000000000000000000000000000000000000000000000000000000000000013fccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccd3e8edfdec0000006b2468b9c600d0c58cd2400215c8137d3681d889502f9008110e021311fb83190c859682f008100f0037be800000000000000106d1a0f2081dcd650041b4683c8207735940200037be800000000000000118e6909ec83b9aca004639a427b20ee6b2802002131181b368cc83b9aca00813120037be80000000000000010d6f7d18c81dcd6500435bdf463207735940200037be80000000000000010aabb974081dcd650042aaee5d0207735940200103d040150201c01b160201c0191701db5019df3018178c37d80001ac91a1f2f4000001ac91a1f2f4b656a359ed7915c09d41b4404204d47590279af58bd861ee89022b8669b459b95b16a8dd1e01919540efb41946d32ef25fc8b544a2cc1cb98a96397f81bd730a2000005693c70000000000000000178c37c34316332218001341b4683c82077359402001eb5019dcdb30178c37d80001ac91a26d06000001ac91a26d0834567237718b34fba73a5f534585934904537d61d3a8bdfdb978e7da1f7c19e0ccaf3c58490506bf60b72248616a8eab7330b9f3a83bb59bc7d545ca5f665df3c9000056998d0000000000000000178c37cb43163334d0c58d9a000000c91a00134639a427b20ee6b280200201c01e1c01db501a0a9440178c37d80001ac91a1f2f4000001ac91a1f2f4e1d8c43eb086223ad1e1ce3477d9eada0ad063e5585c90e0adfa8817766558fd651fb5553c9d3f2e6c592aa74aeec696fde5a52a229ab539cd3005a5e6be80561000005693030000000000000000178c37c34316332a1d0013435bdf4632077359402001db501a08ff50178c37d80001ac91a1f2f4000001ac91a1f2f56bd6930b2976c075423d295c9f31b4cebd6d297e3e13276b8374535ccf4595a46b1ff73c56046425b2841ced1f532941f1465c8510f8d23debdbdffa6f535bc96800005693410000000000000000178c37c3431633221f001342aaee5d020773594020000102000300200a8a0496a296d224f285c67bee93c30f8a309157f0daa35dc5b87e410b78630a09cfc796a296d224f285c67bee93c30f8a309157f0daa35dc5b87e410b78630a09cfc70000000023230000021b3ebf98b74fa3b7f7b2253308fda02625001d4df4da07627d1dbfbd91954fc40008022581b62f87bdc3484d3c0db17c3e6b3802274008272702012029280015bfffffffbcbd0efda563d00015be000003bcb355ab466ad001a09bc7a98700000000040102f186fb0000000100ffffffff00000000000000006862c66900003592345ce30000003592345ce3049ac5beab000acb8b02f186f802f15970c40000000b00000000000001ee2b009800003592344da0c402f186faa5d232d23ae3920b54612b370dd3f4dc817234df917e3567fdbd74abafd866ac789f076e2f96c0cd2e318606631a9c0d2078e41dea7792ea2c64b6e92331208b"));
        Ok(())
    }
}
