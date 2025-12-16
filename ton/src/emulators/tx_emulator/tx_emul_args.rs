use crate::block_tlb::{Msg, ShardAccount};
use crate::emulators::emul_bc_config::EmulBCConfig;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::sync::Arc;
use ton_core::cell::TonHash;
use ton_core::traits::tlb::TLB;

use crate::errors::TonResult;

#[derive(Debug, Clone)]
pub struct TXEmulOrdArgs {
    pub in_msg_boc: Arc<Vec<u8>>,
    pub emul_args: TXEmulArgs,
}

#[derive(Debug, Clone)]
pub struct TXEmulTickTockArgs {
    pub is_tock: bool,
    pub emul_args: TXEmulArgs,
}

#[derive(Debug, Clone)]
pub struct TXEmulArgs {
    pub shard_account_boc: Arc<Vec<u8>>,
    pub bc_config: EmulBCConfig,
    pub rand_seed: TonHash,
    pub utime: u32,
    pub lt: u64,
    pub ignore_chksig: bool,
    pub prev_blocks_boc: Option<Arc<Vec<u8>>>,
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

use bincode::{Decode, Encode};
use function_name::named;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
struct TXEmulOrdArgsSerializable {
    in_msg_boc: Vec<u8>,
    bc_config_boc: Vec<u8>,

    shard_account_boc: Vec<u8>,
    rand_seed: Vec<u8>,
    utime: u32,
    lt: u64,
    ignore_chksig: bool,
    prev_blocks_boc: Option<Vec<u8>>,
    libs_boc: Option<Vec<u8>>,
}

/// Serializes the provided `TXEmulOrdArgs` into a binary format.
///
/// # Parameters
/// - `args`: The transaction emulation arguments to serialize, including the input message BOC and emulation parameters.
///
/// # Returns
/// Returns a `TonResult` containing a `Vec<u8>` with the serialized binary data on success.
///
/// # Errors
/// Returns an error if serialization of any of the fields fails.
#[named]
pub fn dump_tx_emul_ord_args(args: TXEmulOrdArgs) -> TonResult<Vec<u8>> {
    let bc_config_boc = args.emul_args.bc_config.to_boc()?;
    let serializable = TXEmulOrdArgsSerializable {
        in_msg_boc: args.in_msg_boc.as_ref().to_vec(),
        bc_config_boc,
        shard_account_boc: args.emul_args.shard_account_boc.as_ref().to_vec(),
        rand_seed: args.emul_args.rand_seed.as_slice().to_vec(),
        utime: args.emul_args.utime,
        lt: args.emul_args.lt,
        ignore_chksig: args.emul_args.ignore_chksig,
        prev_blocks_boc: args.emul_args.prev_blocks_boc.as_ref().map(|b| b.as_ref().to_vec()),
        libs_boc: args.emul_args.libs_boc.as_ref().map(|b| b.as_ref().to_vec()),
    };
    let binary = bincode::encode_to_vec(&serializable, bincode::config::standard()).unwrap();

    Ok(binary)
}

/// Deserializes a binary blob into a `TXEmulOrdArgs` structure.
///
/// # Parameters
/// - `binary`: A `Vec<u8>` containing the serialized `TXEmulOrdArgs`.
///
/// # Returns
/// Returns a `TonResult<TXEmulOrdArgs>` containing the deserialized structure on success,
/// or an error if deserialization fails or if any of the contained fields cannot be parsed.
///
/// # Errors
/// Returns an error if:
/// - The input binary cannot be decoded into a `TXEmulOrdArgsSerializable`.
/// - The contained `bc_config_boc` cannot be parsed into an `EmulBCConfig`.
/// - The contained `rand_seed` cannot be parsed into a `TonHash`.
pub fn load_tx_emul_ord_args(binary: Vec<u8>) -> TonResult<TXEmulOrdArgs> {
    let (serializable, _): (TXEmulOrdArgsSerializable, usize) =
        bincode::decode_from_slice(&binary, bincode::config::standard()).unwrap();
    let bc_config = EmulBCConfig::from_boc(&serializable.bc_config_boc)?;
    let rand_seed = TonHash::from_slice(&serializable.rand_seed)?;

    Ok(TXEmulOrdArgs {
        in_msg_boc: serializable.in_msg_boc.into(),
        emul_args: TXEmulArgs {
            shard_account_boc: serializable.shard_account_boc.into(),
            bc_config,
            rand_seed,
            utime: serializable.utime,
            lt: serializable.lt,
            ignore_chksig: serializable.ignore_chksig,
            prev_blocks_boc: serializable.prev_blocks_boc.map(Into::into),
            libs_boc: serializable.libs_boc.map(Into::into),
        },
    })
}

pub fn create_test_tx_emul_ord_args(
    ext_in_msg: Msg,
    shard_account: &ShardAccount,
    emul_bc_cfg: &EmulBCConfig,
    utime: u32,
    lt: u64,
) -> TonResult<TXEmulOrdArgs> {
    let in_msg_boc = ext_in_msg.to_boc()?;
    let shard_account_boc = shard_account.to_boc()?;
    assert_eq!(lt, shard_account.last_tx_lt);
    Ok(TXEmulOrdArgs {
        in_msg_boc: in_msg_boc.into(),
        emul_args: TXEmulArgs {
            shard_account_boc: shard_account_boc.into(),
            bc_config: emul_bc_cfg.clone(),
            rand_seed: TonHash::ZERO,
            utime,
            lt: shard_account.last_tx_lt,
            ignore_chksig: false,
            prev_blocks_boc: None,
            libs_boc: None,
        },
    })
}
