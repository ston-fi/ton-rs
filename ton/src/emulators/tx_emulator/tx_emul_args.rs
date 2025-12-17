use crate::block_tlb::{Msg, ShardAccount};
use crate::emulators::emul_bc_config::EmulBCConfig;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::sync::Arc;
use ton_core::cell::TonHash;
use ton_core::serde::serde_ton_hash_hex;
use ton_core::traits::tlb::TLB;

use crate::errors::{TonError, TonResult};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TXEmulOrdArgs {
    #[serde(with = "serde_arc_vec_u8_base64")]
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
    use crate::emulators::tx_emulator::{TXEmulOrdArgs, create_test_tx_emul_ord_args};

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

        let dumped = serde_json::to_string_pretty(&ord_args)?;

        let loaded: TXEmulOrdArgs = serde_json::from_str(&dumped)?;
        assert_eq!(ord_args, loaded);

        Ok(())
    }
}
