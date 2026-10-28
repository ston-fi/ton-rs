use crate::ton_wallet::wallet_tlb::wallet_ext_msg_utils::*;
use ton_core::TLB;
use ton_core::cell::{CellBuilder, CellParser, TonCell, TonHash};
use ton_core::errors::TonCoreError;
use ton_core::traits::tlb::{TLB, TLBPrefix};
use ton_core::types::tlb_core::TLBRef;

/// Wallet v5r1 data.
///
/// Schema: <https://github.com/ton-blockchain/wallet-contract-v5/blob/main/types.tlb#L29>
#[derive(Debug, PartialEq, Clone, TLB)]
pub struct WalletV5Data {
    pub sign_allowed: bool,
    pub seqno: u32,
    pub wallet_id: i32,
    pub public_key: TonHash,
    pub extensions: Option<TLBRef<TonCell>>,
}

impl WalletV5Data {
    /// Creates wallet data with signing enabled, sequence number zero, and no extensions.
    pub fn new(wallet_id: i32, public_key: TonHash) -> Self {
        Self {
            sign_allowed: true,
            seqno: 0,
            wallet_id,
            public_key,
            extensions: None,
        }
    }
}

/// Unsigned wallet v5r1 external-message body.
///
/// Schema: <https://github.com/ton-blockchain/wallet-contract-v5/blob/main/types.tlb>
#[derive(Debug, PartialEq, Clone)]
pub struct WalletV5ExtMsgBody {
    pub wallet_id: i32,
    pub valid_until: u32,
    pub msg_seqno: u32,
    pub msgs_modes: Vec<u8>,
    pub msgs: Vec<TonCell>,
}

impl TLB for WalletV5ExtMsgBody {
    const PREFIX: TLBPrefix = TLBPrefix::new(0x7369676e, 32);
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        let wallet_id = TLB::read(parser)?;
        let valid_until = TLB::read(parser)?;
        let msg_seqno = TLB::read(parser)?;
        let inner_request = InnerRequest::read(parser)?;
        let (msgs, msgs_modes) = parse_inner_request(inner_request)?;
        Ok(Self {
            wallet_id,
            valid_until,
            msg_seqno,
            msgs_modes,
            msgs,
        })
    }

    fn write_definition(&self, dst: &mut CellBuilder) -> Result<(), TonCoreError> {
        self.wallet_id.write(dst)?;
        self.valid_until.write(dst)?;
        self.msg_seqno.write(dst)?;
        let inner_req = build_inner_request(&self.msgs, &self.msgs_modes)?;
        inner_req.write(dst)?;
        Ok(())
    }
}

impl WalletV5ExtMsgBody {
    /// Reads the message body followed by its signature.
    pub fn read_signed(parser: &mut CellParser) -> Result<(Self, Vec<u8>), TonCoreError> {
        let body = Self::read(parser)?;
        let signature = parser.read_bits(512)?;
        Ok((body, signature))
    }
}

#[cfg(test)]
#[path = "../../tests/ton_wallet/wallet_tlb/test_wallet_v5.rs"]
mod test;
