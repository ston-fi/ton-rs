mod mnemonic;
mod wallet_code;
mod wallet_constants;
mod wallet_tlb;
mod wallet_version;

use ed25519_dalek::{Signer, SigningKey};
pub use mnemonic::*;
pub use wallet_code::*;
pub use wallet_constants::*;
pub use wallet_tlb::*;
pub use wallet_version::*;

use super::*;
use crate::block_tlb::*;
use crate::errors::TonError;
use ton_core::cell::TonCell;
use ton_core::traits::tlb::TLB;
use ton_core::types::TonAddress;
use ton_core::types::tlb_core::*;

/// A wallet contract derived from a version and key pair.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
#[non_exhaustive]
pub struct TonWallet {
    pub version: WalletVersion,
    pub key_pair: KeyPair,
    pub address: TonAddress,
    pub wallet_id: i32,
}

impl TonWallet {
    /// Creates a wallet with the default workchain and wallet ID.
    pub fn new(version: WalletVersion, key_pair: KeyPair) -> Result<Self, TonError> {
        let wallet_id = match version {
            WalletVersion::V5R1 => WALLET_V5R1_ID_DEFAULT,
            _ => WALLET_ID_DEFAULT,
        };
        Self::new_with_params(version, key_pair, 0, wallet_id)
    }

    /// Creates a wallet from mnemonic credentials.
    pub fn new_with_creds(version: WalletVersion, seed: &str, pass: Option<String>) -> Result<Self, TonError> {
        Self::new(version, Mnemonic::from_str(seed, pass)?.to_key_pair()?)
    }

    /// Creates a wallet with explicit workchain and wallet ID.
    pub fn new_with_params(
        version: WalletVersion,
        key_pair: KeyPair,
        workchain: i32,
        wallet_id: i32,
    ) -> Result<Self, TonError> {
        let code = WalletVersion::get_code(version)?.clone();
        let data = WalletVersion::get_default_data(version, &key_pair, wallet_id)?;
        let address = StateInit::new(code, data).derive_address(workchain)?;

        Ok(TonWallet {
            key_pair,
            version,
            address,
            wallet_id,
        })
    }

    /// Creates and signs an external inbound message.
    pub fn create_ext_in_msg(
        &self,
        int_msgs: Vec<TonCell>,
        seqno: u32,
        expire_at: u32,
        add_state_init: bool,
    ) -> Result<TonCell, TonError> {
        let body = self.create_ext_in_body(expire_at, seqno, int_msgs)?;
        let signed = self.sign_ext_in_body(&body)?;
        let external = self.create_ext_in_msg_from_body(signed, add_state_init)?;
        Ok(external)
    }

    /// Creates an unsigned external-message body.
    pub fn create_ext_in_body(&self, expire_at: u32, seqno: u32, int_msgs: Vec<TonCell>) -> Result<TonCell, TonError> {
        WalletVersion::build_ext_in_body(self.version, expire_at, seqno, self.wallet_id, int_msgs)
    }

    /// Signs an external-message body.
    pub fn sign_ext_in_body(&self, ext_in_body: &TonCell) -> Result<TonCell, TonError> {
        let msg_hash = ext_in_body.cell_hash()?;

        let signing_key = SigningKey::from_keypair_bytes(&self.key_pair.secret_key)
            .map_err(|err| TonError::Custom(format!("Failed to parse Ed25519 keypair: {err}")))?;

        if signing_key.verifying_key().to_bytes() != self.key_pair.public_key {
            return Err(TonError::Custom("Failed to parse Ed25519 keypair: mismatched public key".to_string()));
        }

        let sign = signing_key.sign(msg_hash.as_slice()).to_bytes().to_vec();
        WalletVersion::sign_msg(self.version, ext_in_body, &sign)
    }

    /// Wraps a signed body in an external inbound message.
    pub fn create_ext_in_msg_from_body(&self, signed_body: TonCell, add_state_init: bool) -> Result<TonCell, TonError> {
        let msg_info = CommonMsgInfo::ExtIn(CommonMsgInfoExtIn {
            src: MsgAddressExt::NONE,
            dst: self.address.to_msg_address_int(),
            import_fee: TLBCoins::ZERO,
        });

        let mut msg = Msg::new(msg_info, signed_body);
        if add_state_init {
            let code = WalletVersion::get_code(self.version)?.clone();
            let data = WalletVersion::get_default_data(self.version, &self.key_pair, self.wallet_id)?;
            let state_init = StateInit::new(code, data);
            msg.init = Some(TLBEitherRef::new(state_init));
        }
        Ok(msg.to_cell()?)
    }
}

#[cfg(test)]
#[path = "tests/test_ton_wallet.rs"]
mod tests;
