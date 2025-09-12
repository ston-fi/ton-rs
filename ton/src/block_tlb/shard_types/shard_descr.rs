use crate::block_tlb::CurrencyCollection;
use ton_lib_core::cell::{CellBuilder, CellParser, TonCell, TonHash};
use ton_lib_core::errors::TonCoreError;
use ton_lib_core::traits::tlb::TLB;
use ton_lib_core::{bail_ton_core_data, TLB};

#[derive(Debug, Clone, PartialEq)]
pub enum ShardDescrTag {
    Old,
    New,
}

// https://github.com/ton-blockchain/ton/blame/26761a1d139402ef343081810677d2582c3eff51/crypto/block/block.tlb#L509
#[derive(Debug, Clone, PartialEq)]
pub struct ShardDescr {
    pub prefix: ShardDescrTag, // in fact it's TLBPrefix
    pub seqno: u32,
    pub reg_mc_seqno: u32,
    pub start_lt: u64,
    pub end_lt: u64,
    pub root_hash: TonHash,
    pub file_hash: TonHash,
    pub before_split: bool,
    pub before_merge: bool,
    pub want_split: bool,
    pub want_merge: bool,
    pub nx_cc_updated: bool,
    pub next_catchain_seqno: u32,
    pub next_validator_shard: u64,
    pub min_ref_mc_seqno: u32,
    pub gen_utime: u32,
    pub split_merge_at: FutureSplitMerge,
    pub fees_collected: CurrencyCollection,
    pub funds_created: CurrencyCollection,
}

// there is a enum, but structures are almost the same (except format)
// so implement TLB manually, including prefix handling
// (implement `read` and `write` as well)
impl TLB for ShardDescr {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        let prefix = match parser.read_num::<u8>(4)? {
            0xb => ShardDescrTag::Old,
            0xa => ShardDescrTag::New,
            x => bail_ton_core_data!("Invalid ShardDescr prefix: {x}"),
        };
        let seqno = TLB::read(parser)?;
        let reg_mc_seqno = TLB::read(parser)?;
        let start_lt = TLB::read(parser)?;
        let end_lt = TLB::read(parser)?;
        let root_hash = TLB::read(parser)?;
        let file_hash = TLB::read(parser)?;
        let before_split = TLB::read(parser)?;
        let before_merge = TLB::read(parser)?;
        let want_split = TLB::read(parser)?;
        let want_merge = TLB::read(parser)?;
        let nx_cc_updated = TLB::read(parser)?;
        parser.read_bits(3)?; // skip
        let next_catchain_seqno = TLB::read(parser)?;
        let next_validator_shard = TLB::read(parser)?;
        let min_ref_mc_seqno = TLB::read(parser)?;
        let gen_utime = TLB::read(parser)?;
        let split_merge_at = TLB::read(parser)?;
        let (fees, funds) = match prefix {
            ShardDescrTag::Old => (TLB::read(parser)?, TLB::read(parser)?),
            ShardDescrTag::New => {
                let mut ref_parser = parser.read_next_ref()?.parser();
                (TLB::read(&mut ref_parser)?, TLB::read(&mut ref_parser)?)
            }
        };
        Ok(Self {
            prefix,
            seqno,
            reg_mc_seqno,
            start_lt,
            end_lt,
            root_hash,
            file_hash,
            before_split,
            before_merge,
            want_split,
            want_merge,
            nx_cc_updated,
            next_catchain_seqno,
            next_validator_shard,
            min_ref_mc_seqno,
            gen_utime,
            split_merge_at,
            fees_collected: fees,
            funds_created: funds,
        })
    }
    fn write_definition(&self, builder: &mut CellBuilder) -> Result<(), TonCoreError> {
        let prefix = match self.prefix {
            ShardDescrTag::Old => 0xb,
            ShardDescrTag::New => 0xa,
        };
        builder.write_num(&prefix, 4)?;
        self.seqno.write(builder)?;
        self.reg_mc_seqno.write(builder)?;
        self.start_lt.write(builder)?;
        self.end_lt.write(builder)?;
        self.root_hash.write(builder)?;
        self.file_hash.write(builder)?;
        self.before_split.write(builder)?;
        self.before_merge.write(builder)?;
        self.want_split.write(builder)?;
        self.want_merge.write(builder)?;
        self.nx_cc_updated.write(builder)?;
        builder.write_num(&0, 3)?;
        self.next_catchain_seqno.write(builder)?;
        self.next_validator_shard.write(builder)?;
        self.min_ref_mc_seqno.write(builder)?;
        self.gen_utime.write(builder)?;
        self.split_merge_at.write(builder)?;

        match &self.prefix {
            ShardDescrTag::Old => {
                self.fees_collected.write(builder)?;
                self.funds_created.write(builder)?;
            }
            ShardDescrTag::New => {
                let mut ref_builder = TonCell::builder();
                self.fees_collected.write(&mut ref_builder)?;
                self.funds_created.write(&mut ref_builder)?;
                builder.write_ref(ref_builder.build()?.into_ref())?;
            }
        }

        Ok(())
    }

    fn read(parser: &mut CellParser) -> Result<Self, TonCoreError> { Self::read_definition(parser) }

    fn write(&self, builder: &mut CellBuilder) -> Result<(), TonCoreError> { self.write_definition(builder) }
}

#[derive(Debug, Clone, PartialEq, TLB)]
pub enum FutureSplitMerge {
    None(FutureSplitMergeNone),
    Split(FutureSplitMergeSplit),
    Merge(FutureSplitMergeMerge),
}

#[derive(Debug, Clone, PartialEq, TLB)]
#[tlb(prefix = 0b0, bits_len = 1)]
pub struct FutureSplitMergeNone;

#[derive(Debug, Clone, PartialEq, TLB)]
#[tlb(prefix = 0b10, bits_len = 2)]
pub struct FutureSplitMergeSplit {
    pub split_utime: u32,
    pub interval: u32,
}

#[derive(Debug, Clone, PartialEq, TLB)]
#[tlb(prefix = 0b11, bits_len = 2)]
pub struct FutureSplitMergeMerge {
    pub merge_utime: u32,
    pub interval: u32,
}
