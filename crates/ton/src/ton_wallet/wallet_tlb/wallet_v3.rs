use crate::ton_wallet::wallet_tlb::wallet_ext_msg_utils::{read_up_to_4_msgs, write_up_to_4_msgs};
use ton_core::TLB;
use ton_core::cell::{CellBuilder, CellParser, TonCell, TonHash};
use ton_core::errors::TonCoreError;
use ton_core::traits::tlb::TLB;

/// Wallet v3 data.
#[derive(Debug, PartialEq, Clone, TLB)]
pub struct WalletV3Data {
    pub seqno: u32,
    pub wallet_id: i32,
    pub public_key: TonHash,
}

impl WalletV3Data {
    /// Creates wallet data with sequence number zero.
    pub fn new(wallet_id: i32, public_key: TonHash) -> Self {
        Self {
            seqno: 0,
            wallet_id,
            public_key,
        }
    }
}

/// Unsigned wallet v3 external-message body.
///
/// Schema: <https://docs.ton.org/participate/wallets/contracts#wallet-v3>
#[derive(Debug, PartialEq, Clone)]
pub struct WalletV3ExtMsgBody {
    pub subwallet_id: i32,
    pub valid_until: u32,
    pub msg_seqno: u32,
    pub msgs_modes: Vec<u8>,
    pub msgs: Vec<TonCell>,
}

impl TLB for WalletV3ExtMsgBody {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        let subwallet_id = TLB::read(parser)?;
        let valid_until = TLB::read(parser)?;
        let msg_seqno = TLB::read(parser)?;
        let (msgs_modes, msgs) = read_up_to_4_msgs(parser)?;
        Ok(Self {
            subwallet_id,
            msg_seqno,
            valid_until,
            msgs_modes,
            msgs,
        })
    }

    fn write_definition(&self, dst: &mut CellBuilder) -> Result<(), TonCoreError> {
        self.subwallet_id.write(dst)?;
        self.valid_until.write(dst)?;
        self.msg_seqno.write(dst)?;
        write_up_to_4_msgs(dst, &self.msgs, &self.msgs_modes)?;
        Ok(())
    }
}

impl WalletV3ExtMsgBody {
    /// Reads a signature followed by the message body.
    pub fn read_signed(parser: &mut CellParser) -> Result<(Self, Vec<u8>), TonCoreError> {
        let signature = parser.read_bits(512)?;
        Ok((Self::read(parser)?, signature))
    }
}

#[cfg(test)]
#[path = "../../tests/ton_wallet/wallet_tlb/test_wallet_v3.rs"]
mod tests;
