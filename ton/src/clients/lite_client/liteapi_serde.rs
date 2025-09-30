use crate::block_tlb::{BlockIdExt, ShardIdent};
use crate::clients::client_types::{BlockId, MasterchainInfo, ZeroStateIdExt};
use ton_liteapi::tl::common::Int256;
// impl From<Int256> for TonHash {
//     fn from(value: Int256) -> Self { TonHash::from_slice_sized(&value.0) }
// }
//
// impl From<TonHash> for Int256 {
//     fn from(value: TonHash) -> Self { Int256(*value.as_slice_sized()) }
// }

impl From<ton_liteapi::tl::common::BlockId> for BlockId {
    fn from(value: ton_liteapi::tl::common::BlockId) -> Self {
        BlockId {
            wc: value.workchain,
            shard: value.shard,
            seqno: value.seqno,
        }
    }
}

impl From<BlockId> for ton_liteapi::tl::common::BlockId {
    fn from(value: BlockId) -> Self {
        ton_liteapi::tl::common::BlockId {
            workchain: value.wc,
            shard: value.shard,
            seqno: value.seqno,
        }
    }
}

impl From<ton_liteapi::tl::common::BlockIdExt> for BlockIdExt {
    fn from(value: ton_liteapi::tl::common::BlockIdExt) -> Self {
        BlockIdExt {
            shard_ident: ShardIdent::new(value.workchain, value.shard),
            seqno: value.seqno,
            root_hash: value.root_hash.0.into(),
            file_hash: value.file_hash.0.into(),
        }
    }
}

impl From<BlockIdExt> for ton_liteapi::tl::common::BlockIdExt {
    fn from(value: BlockIdExt) -> Self {
        ton_liteapi::tl::common::BlockIdExt {
            workchain: value.shard_ident.workchain,
            shard: value.shard_ident.shard,
            seqno: value.seqno,
            root_hash: Int256(*value.root_hash.as_slice_sized()),
            file_hash: Int256(*value.file_hash.as_slice_sized()),
        }
    }
}

impl From<ton_liteapi::tl::common::ZeroStateIdExt> for ZeroStateIdExt {
    fn from(value: ton_liteapi::tl::common::ZeroStateIdExt) -> Self {
        ZeroStateIdExt {
            workchain: value.workchain,
            root_hash: value.root_hash.0.into(),
            file_hash: value.file_hash.0.into(),
        }
    }
}

impl From<ZeroStateIdExt> for ton_liteapi::tl::common::ZeroStateIdExt {
    fn from(value: ZeroStateIdExt) -> Self {
        ton_liteapi::tl::common::ZeroStateIdExt {
            workchain: value.workchain,
            root_hash: Int256(*value.root_hash.as_slice_sized()),
            file_hash: Int256(*value.file_hash.as_slice_sized()),
        }
    }
}

impl From<ton_liteapi::tl::response::MasterchainInfo> for MasterchainInfo {
    fn from(value: ton_liteapi::tl::response::MasterchainInfo) -> Self {
        MasterchainInfo {
            last: value.last.into(),
            state_root_hash: value.state_root_hash.0.into(),
            init: value.init.into(),
        }
    }
}

impl From<MasterchainInfo> for ton_liteapi::tl::response::MasterchainInfo {
    fn from(value: MasterchainInfo) -> Self {
        ton_liteapi::tl::response::MasterchainInfo {
            last: value.last.into(),
            state_root_hash: Int256(*value.state_root_hash.as_slice_sized()),
            init: value.init.into(),
        }
    }
}
