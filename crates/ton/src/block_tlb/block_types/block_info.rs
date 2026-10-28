use crate::block_tlb::GlobalVersion;
use crate::block_tlb::ShardIdent;
use crate::block_tlb::block_types::block_id_ext::BlockIdExt;
use crate::block_tlb::block_types::block_prev_info::{BlockPrevInfoAfterMerge, PrevBlockInfo};
use std::ops::Deref;
use ton_core::TLB;
use ton_core::bail_ton_core;
use ton_core::cell::{CellBuilder, CellParser, TonHash};
use ton_core::errors::TonCoreError;
use ton_core::traits::tlb::{TLB, TLBPrefix};

const GEN_SOFTWARE_EXISTS_FLAG: u8 = 1;

/// Struct doesn't check invariant during read/write. So you're free to build incorrect block_info cell
/// For example, set is_master == false and master_ref != None
/// Just don't do it.
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
                if self.after_split && ext_ref.seqno == 0 {
                    bail_ton_core!("shardchains cannot be split immediately after initial state");
                }
                let shard = match self.after_split {
                    true => self.shard.merge()?,
                    false => self.shard.clone(),
                };
                vec![make_block_id(ext_ref.clone(), shard)]
            },
            PrevBlockInfo::AfterMerge(ext_refs) => {
                if self.after_split {
                    bail_ton_core!("shardchains cannot be simultaneously split and merged at the same block");
                }
                if ext_refs.prev1.seqno == 0 || ext_refs.prev2.seqno == 0 {
                    bail_ton_core!("shardchains cannot be merged immediately after initial state");
                }
                let (shard1, shard2) = self.shard.split()?;
                vec![
                    make_block_id(ext_refs.prev1.deref().clone(), shard1),
                    make_block_id(ext_refs.prev2.deref().clone(), shard2),
                ]
            },
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

        let gen_software =
            if (flags & GEN_SOFTWARE_EXISTS_FLAG) != 0 { Some(GlobalVersion::read(parser)?) } else { None };
        let master_ref =
            if not_master { Some(ExtBlockRef::read(&mut parser.read_next_ref()?.parser())?) } else { None };

        let mut pref_ref_parser = parser.read_next_ref()?.parser();
        let prev_ref = if after_merge {
            PrevBlockInfo::AfterMerge(BlockPrevInfoAfterMerge::read(&mut pref_ref_parser)?)
        } else {
            PrevBlockInfo::Regular(ExtBlockRef::read(&mut pref_ref_parser)?)
        };

        let prev_vert_ref =
            if vert_seqno_incr { Some(ExtBlockRef::read(&mut parser.read_next_ref()?.parser())?) } else { None };

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
            builder.write_ref(master_ref.to_cell()?)?;
        }

        builder.write_ref(self.prev_ref.to_cell()?)?;
        if let Some(prev_vert_ref) = &self.prev_vert_ref {
            builder.write_ref(prev_vert_ref.to_cell()?)?;
        }

        Ok(())
    }
}

#[cfg(test)]
#[path = "../../tests/block_tlb/block_types/test_block_info.rs"]
mod tests;
