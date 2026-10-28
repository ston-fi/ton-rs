use crate::ton_wallet::wallet_tlb::wallet_ext_msg_utils::{read_up_to_4_msgs, write_up_to_4_msgs};
use ton_core::cell::{CellBuilder, CellParser, TonCell, TonHash};
use ton_core::errors::TonCoreError;
use ton_core::traits::tlb::TLB;
use ton_core::types::tlb_core::TLBRef;
use ton_core::{TLB, bail_ton_core};

/// Wallet v4 data.
#[derive(Debug, PartialEq, Clone, TLB)]
pub struct WalletV4Data {
    pub seqno: u32,
    pub wallet_id: i32,
    pub public_key: TonHash,
    pub plugins: Option<TLBRef<TonCell>>,
}

impl WalletV4Data {
    /// Creates wallet data with sequence number zero and no plugins.
    pub fn new(wallet_id: i32, public_key: TonHash) -> Self {
        Self {
            seqno: 0,
            wallet_id,
            public_key,
            plugins: None,
        }
    }
}

/// Unsigned wallet v4 external-message body.
///
/// Schema: <https://docs.ton.org/participate/wallets/contracts#wallet-v4>
#[derive(Debug, PartialEq, Clone)]
pub struct WalletV4ExtMsgBody {
    pub subwallet_id: i32,
    pub valid_until: u32,
    pub msg_seqno: u32,
    pub opcode: u8,
    pub msgs_modes: Vec<u8>,
    pub msgs: Vec<TonCell>,
}

impl TLB for WalletV4ExtMsgBody {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        let subwallet_id = TLB::read(parser)?;
        let valid_until = TLB::read(parser)?;
        let msg_seqno = TLB::read(parser)?;
        let opcode = TLB::read(parser)?;
        if opcode != 0 {
            bail_ton_core!("Unsupported opcode: {opcode}");
        }
        let (msgs_modes, msgs) = read_up_to_4_msgs(parser)?;
        Ok(Self {
            subwallet_id,
            valid_until,
            msg_seqno,
            opcode,
            msgs_modes,
            msgs,
        })
    }

    fn write_definition(&self, dst: &mut CellBuilder) -> Result<(), TonCoreError> {
        if self.opcode != 0 {
            bail_ton_core!("Unsupported opcode: {}", self.opcode);
        }
        self.subwallet_id.write(dst)?;
        self.valid_until.write(dst)?;
        self.msg_seqno.write(dst)?;
        self.opcode.write(dst)?;
        write_up_to_4_msgs(dst, &self.msgs, &self.msgs_modes)?;
        Ok(())
    }
}

impl WalletV4ExtMsgBody {
    /// Reads a signature followed by the message body.
    pub fn read_signed(parser: &mut CellParser) -> Result<(Self, Vec<u8>), TonCoreError> {
        let signature = parser.read_bits(512)?;
        Ok((Self::read(parser)?, signature))
    }
}

#[cfg(test)]
#[path = "../../tests/ton_wallet/wallet_tlb/test_wallet_v4.rs"]
mod tests;
