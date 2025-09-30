use serde::de::IntoDeserializer;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

pub(super) mod serde_block_id_ext {
    use super::*;
    use crate::block_tlb::{BlockIdExt, ShardIdent};
    use crate::clients::tl_client::tl::Base64Standard;
    use serde_aux::prelude::deserialize_number_from_string;
    use ton_lib_core::cell::TonHash;

    // tonlib_api.tl_api, line 51
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
    struct TLBlockIdExt {
        pub workchain: i32,
        #[serde(deserialize_with = "deserialize_number_from_string")]
        pub shard: i64,
        pub seqno: i32,
        #[serde(with = "Base64Standard")]
        pub root_hash: Vec<u8>,
        #[serde(with = "Base64Standard")]
        pub file_hash: Vec<u8>,
    }

    pub fn serialize<S: Serializer>(data: &BlockIdExt, serializer: S) -> Result<S::Ok, S::Error> {
        TLBlockIdExt {
            workchain: data.shard_ident.workchain,
            shard: data.shard_ident.shard as i64,
            seqno: data.seqno as i32,
            root_hash: data.root_hash.as_slice().to_vec(),
            file_hash: data.file_hash.as_slice().to_vec(),
        }
        .serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<BlockIdExt, D::Error> {
        let tl_block_id_ext = TLBlockIdExt::deserialize(deserializer)?;
        Ok(BlockIdExt {
            shard_ident: ShardIdent {
                workchain: tl_block_id_ext.workchain,
                shard: tl_block_id_ext.shard as u64,
            },
            seqno: tl_block_id_ext.seqno as u32,
            root_hash: TonHash::from_vec(tl_block_id_ext.root_hash).map_err(Error::custom)?,
            file_hash: TonHash::from_vec(tl_block_id_ext.file_hash).map_err(Error::custom)?,
        })
    }
}

pub(super) mod serde_block_id_ext_vec {
    use super::*;
    use crate::block_tlb::BlockIdExt;
    pub fn serialize<S: Serializer>(data: &[BlockIdExt], serializer: S) -> Result<S::Ok, S::Error> {
        let tl_wrapped: Vec<_> = data
            .iter()
            .map(|b| serde_block_id_ext::serialize(b, serde_json::value::Serializer))
            .collect::<Result<_, _>>()
            .map_err(serde::ser::Error::custom)?;
        tl_wrapped.serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<BlockIdExt>, D::Error> {
        let values: Vec<serde_json::Value> = Vec::deserialize(deserializer)?;
        values.into_iter().map(serde_block_id_ext::deserialize).collect::<Result<_, _>>().map_err(Error::custom)
    }
}

pub(super) mod serde_block_id_ext_vec_opt {
    use super::*;
    use crate::block_tlb::BlockIdExt;

    pub fn serialize<S: Serializer>(data: &Option<Vec<BlockIdExt>>, serializer: S) -> Result<S::Ok, S::Error> {
        match data {
            Some(vec) => serde_block_id_ext_vec::serialize(vec, serializer),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Vec<BlockIdExt>>, D::Error> {
        let opt = Option::<Vec<serde_json::Value>>::deserialize(deserializer)?;
        match opt {
            Some(v) => {
                let vec = serde_block_id_ext_vec::deserialize(v.into_deserializer()).map_err(Error::custom)?;
                Ok(Some(vec))
            }
            None => Ok(None),
        }
    }
}
