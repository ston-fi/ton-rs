use crate::block_tlb::{Msg, ShardAccount};
use crate::emulators::emul_bc_config::EmulBCConfig;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::sync::Arc;
use ton_core::cell::TonHash;
use ton_core::serde::serde_ton_hash_hex;
use ton_core::traits::tlb::TLB;

use crate::errors::{TonError, TonResult};

#[derive(Debug, Clone, PartialEq)]
pub struct TXEmulOrdArgs {
    pub in_msg_boc: Arc<Vec<u8>>,
    pub emul_args: TXEmulArgs,
}

#[derive(Debug, Clone)]
pub struct TXEmulTickTockArgs {
    pub is_tock: bool,
    pub emul_args: TXEmulArgs,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TXEmulArgs {
    #[serde(with = "serde_arc_vec_u8_base64")]
    pub shard_account_boc: Arc<Vec<u8>>,
    #[serde(with = "crate::emulators::emul_bc_config::serde_emul_bc_config")]
    pub bc_config: EmulBCConfig,
    #[serde(with = "serde_ton_hash_hex")]
    pub rand_seed: TonHash,
    pub utime: u32,
    pub lt: u64,
    pub ignore_chksig: bool,
    #[serde(with = "serde_opt_arc_vec_u8_base64")]
    pub prev_blocks_boc: Option<Arc<Vec<u8>>>,
    #[serde(with = "serde_opt_arc_vec_u8_base64")]
    pub libs_boc: Option<Arc<Vec<u8>>>,
}

impl Display for TXEmulArgs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let shard_acc_str = hex::encode(self.shard_account_boc.deref());

        let prev_blocks_str = match &self.prev_blocks_boc {
            None => "None",
            Some(boc) => &hex::encode(boc.deref()),
        };

        let libs_str = match &self.libs_boc {
            None => "None",
            Some(boc) => &hex::encode(boc.deref()),
        };

        f.write_fmt(format_args!(
            "shard_account_boc: {}, bc_config: {}, rand_seed: {}, utime: {}, lt: {}, ignore_chksig: {}, prev_blocks_boc: {}, libs_boc: {}",
            shard_acc_str, self.bc_config.to_string_lossy(), self.rand_seed, self.utime, self.lt, self.ignore_chksig, prev_blocks_str, libs_str
        ))
    }
}

impl Display for TXEmulOrdArgs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "in_msg_boc : {}, emul_args: {}",
            hex::encode(self.in_msg_boc.deref()),
            &self.emul_args
        ))
    }
}

impl Display for TXEmulTickTockArgs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("is_tock: {}, emul_args: {}", self.is_tock, &self.emul_args))
    }
}

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

// Custom serialization for Arc<Vec<u8>> as base64
mod serde_arc_vec_u8_base64 {
    use super::*;
    use serde::de::Error;

    pub fn serialize<S: Serializer>(data: &Arc<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&STANDARD.encode(data.as_ref()))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Arc<Vec<u8>>, D::Error> {
        let s = String::deserialize(deserializer)?;
        let bytes = STANDARD.decode(&s).map_err(Error::custom)?;
        Ok(Arc::new(bytes))
    }
}

// Custom serialization for Option<Arc<Vec<u8>>> as base64
mod serde_opt_arc_vec_u8_base64 {
    use super::*;
    use serde::de::Error;

    pub fn serialize<S: Serializer>(data: &Option<Arc<Vec<u8>>>, serializer: S) -> Result<S::Ok, S::Error> {
        match data {
            Some(bytes) => serializer.serialize_some(&STANDARD.encode(bytes.as_ref())),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Arc<Vec<u8>>>, D::Error> {
        let opt = Option::<String>::deserialize(deserializer)?;
        match opt {
            Some(s) => {
                let bytes = STANDARD.decode(&s).map_err(Error::custom)?;
                Ok(Some(Arc::new(bytes)))
            }
            None => Ok(None),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct TXEmulOrdArgsSerializable {
    in_msg_boc: String,        // base64 encoded
    bc_config_boc: String,     // base64 encoded
    shard_account_boc: String, // base64 encoded
    rand_seed: String,         // hex encoded
    utime: u32,
    lt: u64,
    ignore_chksig: bool,
    prev_blocks_boc: Option<String>, // base64 encoded
    libs_boc: Option<String>,        // base64 encoded
}

pub fn dump_tx_emul_ord_args(args: TXEmulOrdArgs) -> TonResult<Vec<u8>> {
    let bc_config_boc = args.emul_args.bc_config.to_boc()?;
    let serializable = TXEmulOrdArgsSerializable {
        in_msg_boc: STANDARD.encode(args.in_msg_boc.as_ref()),
        bc_config_boc: STANDARD.encode(&bc_config_boc),
        shard_account_boc: STANDARD.encode(args.emul_args.shard_account_boc.as_ref()),
        rand_seed: hex::encode(args.emul_args.rand_seed.as_slice()),
        utime: args.emul_args.utime,
        lt: args.emul_args.lt,
        ignore_chksig: args.emul_args.ignore_chksig,
        prev_blocks_boc: args.emul_args.prev_blocks_boc.as_ref().map(|b| STANDARD.encode(b.as_ref())),
        libs_boc: args.emul_args.libs_boc.as_ref().map(|b| STANDARD.encode(b.as_ref())),
    };
    let json_str = serde_json::to_string(&serializable)
        .map_err(|e| TonError::Custom(format!("Failed to encode TXEmulOrdArgs to JSON: {e}")))?;
    Ok(json_str.into_bytes())
}

pub fn load_tx_emul_ord_args(binary: Vec<u8>) -> TonResult<TXEmulOrdArgs> {
    let json_str = String::from_utf8(binary)
        .map_err(|e| TonError::Custom(format!("Failed to decode binary to UTF-8 string: {e}")))?;
    let serializable: TXEmulOrdArgsSerializable = serde_json::from_str(&json_str)
        .map_err(|e| TonError::Custom(format!("Failed to decode TXEmulOrdArgs from JSON: {e}")))?;

    let in_msg_boc = STANDARD
        .decode(&serializable.in_msg_boc)
        .map_err(|e| TonError::Custom(format!("Failed to decode in_msg_boc from base64: {e}")))?;
    let bc_config_boc = STANDARD
        .decode(&serializable.bc_config_boc)
        .map_err(|e| TonError::Custom(format!("Failed to decode bc_config_boc from base64: {e}")))?;
    let shard_account_boc = STANDARD
        .decode(&serializable.shard_account_boc)
        .map_err(|e| TonError::Custom(format!("Failed to decode shard_account_boc from base64: {e}")))?;
    let rand_seed = hex::decode(&serializable.rand_seed)
        .map_err(|e| TonError::Custom(format!("Failed to decode rand_seed from hex: {e}")))?;

    let bc_config = EmulBCConfig::from_boc(&bc_config_boc)?;
    let rand_seed = TonHash::from_slice(&rand_seed)?;

    let prev_blocks_boc = serializable
        .prev_blocks_boc
        .map(|s| {
            STANDARD
                .decode(&s)
                .map_err(|e| TonError::Custom(format!("Failed to decode prev_blocks_boc from base64: {e}")))
        })
        .transpose()?;
    let libs_boc = serializable
        .libs_boc
        .map(|s| {
            STANDARD.decode(&s).map_err(|e| TonError::Custom(format!("Failed to decode libs_boc from base64: {e}")))
        })
        .transpose()?;

    Ok(TXEmulOrdArgs {
        in_msg_boc: in_msg_boc.into(),
        emul_args: TXEmulArgs {
            shard_account_boc: shard_account_boc.into(),
            bc_config,
            rand_seed,
            utime: serializable.utime,
            lt: serializable.lt,
            ignore_chksig: serializable.ignore_chksig,
            prev_blocks_boc: prev_blocks_boc.map(Into::into),
            libs_boc: libs_boc.map(Into::into),
        },
    })
}

pub fn create_test_tx_emul_ord_args(
    ext_in_msg: Msg,
    shard_account: &ShardAccount,
    emul_bc_cfg: &EmulBCConfig,
    rand_seed: TonHash,
    utime: u32,
    lt: u64,
) -> TonResult<TXEmulOrdArgs> {
    let in_msg_boc = ext_in_msg.to_boc()?;
    let shard_account_boc = shard_account.to_boc()?;
    Ok(TXEmulOrdArgs {
        in_msg_boc: in_msg_boc.into(),
        emul_args: TXEmulArgs {
            shard_account_boc: shard_account_boc.into(),
            bc_config: emul_bc_cfg.clone(),
            rand_seed: rand_seed,
            utime,
            lt: lt,
            ignore_chksig: false,
            prev_blocks_boc: None,
            libs_boc: None,
        },
    })
}

#[cfg(test)]
mod tests {
    use crate::emulators::tx_emulator::{create_test_tx_emul_ord_args, dump_tx_emul_ord_args};

    #[test]
    fn test_tx_emul_args_serialize() -> anyhow::Result<()> {
        let test_shard_acc = crate::emulators::tx_emulator::tests::TEST_SHARD_ACCOUNT.clone();
        let ord_args = create_test_tx_emul_ord_args(
            crate::emulators::tx_emulator::tests::TEST_MSG_IN_EXT.clone(),
            &test_shard_acc,
            &crate::emulators::tx_emulator::tests::BC_CONFIG,
            crate::emulators::tx_emulator::tests::TEST_RAND_SEED.clone(),
            1738323935,
            53483578000001,
        )?;

        let dumped = dump_tx_emul_ord_args(ord_args.clone())?;

        let loaded = super::load_tx_emul_ord_args(dumped)?;
        assert_eq!(ord_args, loaded);

        Ok(())
    }
}
