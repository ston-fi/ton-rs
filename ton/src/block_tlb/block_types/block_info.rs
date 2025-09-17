use crate::block_tlb::block_types::block_id_ext::BlockIdExt;
use crate::block_tlb::block_types::block_prev_info::{BlockPrevInfoAfterMerge, PrevBlockInfo};
use crate::block_tlb::GlobalVersion;
use crate::block_tlb::ShardIdent;
use ton_lib_core::cell::{CellBuilder, CellParser, TonHash};
use ton_lib_core::errors::TonCoreError;
use ton_lib_core::traits::tlb::{TLBPrefix, TLB};
use ton_lib_core::TLB;

const GEN_SOFTWARE_EXISTS_FLAG: u8 = 1;

/// Struct doesn't check invariant during read/write. So you're free to build incorrect block_info cell
/// For example, set is_master == false and master_ref != None
/// Don't do it and everything will be fine
// https://github.com/ton-blockchain/ton/blob/6f745c04daf8861bb1791cffce6edb1beec62204/crypto/block/block.tlb#L457
#[derive(Debug, Default, Clone, PartialEq)]
pub struct BlockInfo {
    pub version: u32,
    pub not_master: bool,
    pub after_merge: bool,
    pub before_split: bool,
    pub after_split: bool,
    pub want_split: bool,
    pub want_merge: bool,
    pub key_block: bool,
    pub vert_seqno_incr: bool,
    pub flags: u8,
    pub seqno: u32,
    pub vert_seqno: u32,
    pub shard: ShardIdent,
    pub gen_utime: u32,
    pub start_lt: u64,
    pub end_lt: u64,
    pub gen_validator_list_has_short: u32,
    pub gen_catchain_seqno: u32,
    pub min_ref_mc_seqno: u32,
    pub prev_key_block_seqno: u32,
    pub gen_software: Option<GlobalVersion>,
    pub master_ref: Option<ExtBlockRef>,
    pub prev_ref: PrevBlockInfo,
    pub prev_vert_ref: Option<ExtBlockRef>,
}

#[derive(Debug, Clone, PartialEq, TLB)]
pub struct ExtBlockRef {
    pub end_lt: u64,
    pub seqno: u32,
    pub root_hash: TonHash,
    pub file_hash: TonHash,
}

impl BlockInfo {
    pub fn prev_block_ids(&self) -> Result<Vec<BlockIdExt>, TonCoreError> {
        let make_block_id = |ext_ref: ExtBlockRef, shard| BlockIdExt {
            shard_ident: shard,
            seqno: ext_ref.seqno,
            root_hash: ext_ref.root_hash,
            file_hash: ext_ref.file_hash,
        };
        let prev_ids = match &self.prev_ref {
            PrevBlockInfo::Regular(ext_ref) => {
                let shard = match self.after_split {
                    true => self.shard.merge()?,
                    false => self.shard.clone(),
                };
                vec![make_block_id(ext_ref.clone(), shard)]
            }
            PrevBlockInfo::AfterMerge(ext_refs) => {
                let (shard1, shard2) = self.shard.split()?;
                vec![
                    make_block_id(ext_refs.prev1.clone(), shard1),
                    make_block_id(ext_refs.prev2.clone(), shard2),
                ]
            }
        };
        Ok(prev_ids)
    }
}

impl TLB for BlockInfo {
    const PREFIX: TLBPrefix = TLBPrefix::new(0x9bc7a987, 32);

    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        let version = parser.read_num(32)?;
        let not_master = parser.read_bit()?;
        let after_merge = parser.read_bit()?;
        let before_split = parser.read_bit()?;
        let after_split = parser.read_bit()?;
        let want_split = parser.read_bit()?;
        let want_merge = parser.read_bit()?;
        let key_block = parser.read_bit()?;
        let vert_seqno_incr = parser.read_bit()?;
        let flags = parser.read_num(8)?;
        let seqno = parser.read_num(32)?;
        let vert_seqno = parser.read_num(32)?;
        let shard = ShardIdent::read(parser)?;
        let gen_utime = parser.read_num(32)?;
        let start_lt = parser.read_num(64)?;
        let end_lt = parser.read_num(64)?;
        let gen_validator_list_has_short: u32 = parser.read_num(32)?;
        let gen_catchain_seqno: u32 = parser.read_num(32)?;
        let min_ref_mc_seqno: u32 = parser.read_num(32)?;
        let prev_key_block_seqno: u32 = parser.read_num(32)?;

        let gen_software = if (flags & GEN_SOFTWARE_EXISTS_FLAG) != 0 {
            Some(GlobalVersion::read(parser)?)
        } else {
            None
        };
        let master_ref = if not_master {
            Some(ExtBlockRef::read(&mut parser.read_next_ref()?.parser())?)
        } else {
            None
        };

        let mut pref_ref_parser = parser.read_next_ref()?.parser();
        let prev_ref = if after_merge {
            PrevBlockInfo::AfterMerge(BlockPrevInfoAfterMerge::read(&mut pref_ref_parser)?)
        } else {
            PrevBlockInfo::Regular(ExtBlockRef::read(&mut pref_ref_parser)?)
        };

        let prev_vert_ref = if vert_seqno_incr {
            Some(ExtBlockRef::read(&mut parser.read_next_ref()?.parser())?)
        } else {
            None
        };

        Ok(Self {
            version,
            not_master,
            after_merge,
            before_split,
            after_split,
            want_split,
            want_merge,
            key_block,
            vert_seqno_incr,
            flags,
            seqno,
            vert_seqno,
            shard,
            gen_utime,
            start_lt,
            end_lt,
            gen_validator_list_has_short,
            gen_catchain_seqno,
            min_ref_mc_seqno,
            prev_key_block_seqno,
            gen_software,
            master_ref,
            prev_ref,
            prev_vert_ref,
        })
    }

    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonCoreError> {
        self.version.write(builder)?;
        self.not_master.write(builder)?;
        self.after_merge.write(builder)?;
        self.before_split.write(builder)?;
        self.after_split.write(builder)?;
        self.want_split.write(builder)?;
        self.want_merge.write(builder)?;
        self.key_block.write(builder)?;
        self.vert_seqno_incr.write(builder)?;
        self.flags.write(builder)?;
        self.seqno.write(builder)?;
        self.vert_seqno.write(builder)?;
        self.shard.write(builder)?;
        self.gen_utime.write(builder)?;
        self.start_lt.write(builder)?;
        self.end_lt.write(builder)?;
        self.gen_validator_list_has_short.write(builder)?;
        self.gen_catchain_seqno.write(builder)?;
        self.min_ref_mc_seqno.write(builder)?;
        self.prev_key_block_seqno.write(builder)?;
        if let Some(gen_software) = &self.gen_software {
            gen_software.write(builder)?;
        }
        if let Some(master_ref) = &self.master_ref {
            builder.write_ref(master_ref.to_cell_ref()?)?;
        }

        builder.write_ref(self.prev_ref.to_cell_ref()?)?;
        if let Some(prev_vert_ref) = &self.prev_vert_ref {
            builder.write_ref(prev_vert_ref.to_cell_ref()?)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_tlb::Block;
    use crate::block_tlb::_test_block_data::{MASTER_BLOCK_BOC_HEX, SHARD_BLOCK_BOC_HEX};
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
        assert_eq!(parsed_block_info, expected);

        let parsed_back = BlockInfo::from_boc(&parsed_block_info.to_boc()?)?;
        assert_eq!(parsed_block_info, parsed_back);
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
        assert_eq!(parsed_block_info, expected);
        let parsed_back = BlockInfo::from_boc(&parsed_block_info.to_boc()?)?;
        assert_eq!(parsed_block_info, parsed_back);
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
            },
            prev2: ExtBlockRef {
                end_lt: 5,
                seqno: 6,
                root_hash: TonHash::from_slice_sized(&[7u8; 32]),
                file_hash: TonHash::from_slice_sized(&[8u8; 32]),
            },
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
}
