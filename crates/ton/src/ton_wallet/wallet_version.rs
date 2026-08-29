use crate::errors::TonError;
use crate::ton_wallet::WalletVersion::*;
use crate::ton_wallet::*;
use ton_core::bail_ton_core;
use ton_core::cell::{TonCell, TonHash};
use ton_core::errors::TonCoreError;
use ton_core::traits::tlb::TLB;

/// A TON wallet contract version.
///
/// Serde represents versions using their Rust variant names, such as `"V4R2"` and `"HLV2R2"`.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum WalletVersion {
    V1R1,
    V1R2,
    V1R3,
    V2R1,
    V2R2,
    V3R1,
    V3R2,
    V4R1,
    V4R2,
    V5R1,
    HLV1R1,
    HLV1R2,
    HLV2,
    HLV2R1,
    HLV2R2,
}

impl WalletVersion {
    /// Builds the initial data cell for a wallet version.
    pub fn get_default_data(
        version: WalletVersion,
        key_pair: &KeyPair,
        wallet_id: i32,
    ) -> Result<TonCell, TonCoreError> {
        let public_key = TonHash::from_slice(&key_pair.public_key)?;
        match version {
            V1R1 | V1R2 | V1R3 | V2R1 | V2R2 => WalletV1V2Data::new(public_key).to_cell(),
            V3R1 | V3R2 => WalletV3Data::new(wallet_id, public_key).to_cell(),
            V4R1 | V4R2 => WalletV4Data::new(wallet_id, public_key).to_cell(),
            V5R1 => WalletV5Data::new(wallet_id, public_key).to_cell(),
            HLV2R2 => WalletHLV2R2Data::new(wallet_id, public_key).to_cell(),
            HLV1R1 | HLV1R2 | HLV2 | HLV2R1 => {
                bail_ton_core!("initial_data for {version:?} is unsupported");
            },
        }
    }

    /// Returns the code cell for a wallet version.
    pub fn get_code(version: WalletVersion) -> Result<&'static TonCell, TonCoreError> {
        TON_WALLET_CODE_BY_VERSION
            .get(&version)
            .ok_or_else(|| TonCoreError::Custom(format!("No code found for {version:?}")))
    }

    /// Detects a wallet version from its code hash.
    pub fn get_version_by_code(code_hash: TonHash) -> Result<WalletVersion, TonCoreError> {
        TON_WALLET_VERSION_BY_CODE
            .get(&code_hash)
            .copied()
            .ok_or_else(|| TonCoreError::Custom(format!("No version found for code_hash: {code_hash}")))
    }

    /// Builds an unsigned external-message body.
    pub fn build_ext_in_body(
        version: WalletVersion,
        valid_until: u32,
        msg_seqno: u32,
        wallet_id: i32,
        msgs: Vec<TonCell>,
    ) -> Result<TonCell, TonError> {
        let res = match version {
            V2R1 | V2R2 => WalletV2ExtMsgBody {
                msg_seqno,
                valid_until,
                msgs_modes: vec![3u8; msgs.len()],
                msgs,
            }
            .to_cell(),
            V3R1 | V3R2 => WalletV3ExtMsgBody {
                subwallet_id: wallet_id,
                msg_seqno,
                valid_until,
                msgs_modes: vec![3u8; msgs.len()],
                msgs,
            }
            .to_cell(),
            V4R1 | V4R2 => WalletV4ExtMsgBody {
                subwallet_id: wallet_id,
                valid_until,
                msg_seqno,
                opcode: 0,
                msgs_modes: vec![3u8; msgs.len()],
                msgs,
            }
            .to_cell(),
            V5R1 => WalletV5ExtMsgBody {
                wallet_id,
                valid_until,
                msg_seqno,
                msgs_modes: vec![3u8; msgs.len()],
                msgs,
            }
            .to_cell(),
            _ => Err(TonCoreError::Custom(format!("build_ext_in_body for {version:?} is unsupported"))),
        };
        res.map_err(TonError::from)
    }

    pub(super) fn sign_msg(version: WalletVersion, msg_cell: &TonCell, sign: &[u8]) -> Result<TonCell, TonError> {
        match version {
            // different order
            V5R1 => {
                let mut builder = TonCell::builder();
                builder.write_cell(msg_cell)?;
                builder.write_bits(sign, sign.len() * 8)?;
                Ok(builder.build()?)
            },
            _ => {
                let mut builder = TonCell::builder();
                builder.write_bits(sign, sign.len() * 8)?;
                builder.write_cell(msg_cell)?;
                Ok(builder.build()?)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::WalletVersion;

    #[test]
    fn test_wallet_version_serde_contract() -> anyhow::Result<()> {
        let version = WalletVersion::V4R2;
        let serialized = "\"V4R2\"";

        assert_eq!(serde_json::to_string(&version)?, serialized);
        assert_eq!(serde_json::from_str::<WalletVersion>(serialized)?, version);

        Ok(())
    }
}
