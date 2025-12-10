use crate::block_tlb::ShardIdent;
use ton_core::TLB;
use ton_core::cell::TonHash;
use ton_core::constants::{TON_MASTERCHAIN, TON_SHARD_FULL};

#[derive(Debug, Clone, PartialEq, Eq, Hash, TLB)]
pub struct BlockIdExt {
    pub shard_ident: ShardIdent,
    pub seqno: u32,
    pub root_hash: TonHash,
    pub file_hash: TonHash,
}

impl BlockIdExt {
    #[rustfmt::skip]
    pub const ZERO_BLOCK_ID: BlockIdExt = BlockIdExt {
        shard_ident: ShardIdent {
            workchain: TON_MASTERCHAIN,
            shard: TON_SHARD_FULL,
        },
        seqno: 0,
        root_hash: TonHash::from_slice_sized(&[23, 163, 169, 41, 146, 170, 190, 167, 133, 167, 160, 144, 152, 90, 38, 92, 211, 31, 50, 61, 132, 157, 165, 18, 57, 115, 126, 50, 31, 176, 85, 105]),
        file_hash: TonHash::from_slice_sized(&[94, 153, 79, 207, 77, 66, 92, 10, 108, 230, 167, 146, 89, 75, 113, 115, 32, 95, 116, 10, 57, 205, 86, 245, 55, 222, 253, 40, 180, 138, 15, 110]),
    };

    #[rustfmt::skip]
    pub const ZERO_BLOCK_ID_TESTNET: BlockIdExt = BlockIdExt {
        shard_ident: ShardIdent {
            workchain: TON_MASTERCHAIN,
            shard: TON_SHARD_FULL,
        },
        seqno: 0,
        root_hash: TonHash::from_slice_sized(&[130, 63, 129, 243, 6, 255, 2, 105, 79, 147, 92, 245, 2, 21, 72, 227, 206, 43, 134, 181, 41, 129, 42, 246, 161, 33, 72, 135, 158, 149, 161, 40]),
        file_hash: TonHash::from_slice_sized(&[103, 226, 10, 193, 132, 185, 224, 57, 166, 38, 103, 172, 195, 249, 192, 15, 144, 243, 89, 167, 103, 56, 35, 51, 121, 239, 164, 118, 4, 152, 12, 232]),
    };
}
