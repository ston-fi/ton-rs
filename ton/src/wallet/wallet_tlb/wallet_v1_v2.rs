use crate::wallet::wallet_tlb::wallet_ext_msg_utils::{read_up_to_4_msgs, write_up_to_4_msgs};
use ton_lib_core::cell::{CellBuilder, CellParser, TonCellRef, TonHash};
use ton_lib_core::errors::TonCoreError;
use ton_lib_core::traits::tlb::TLB;
use ton_lib_core::TLB;

/// Is not covered by tests and basically unsupported
/// WalletVersion::V1R1 | WalletVersion::V1R2 | WalletVersion::V1R3 | WalletVersion::V2R1 | WalletVersion::V2R2
#[derive(Debug, PartialEq, Clone, TLB)]
pub struct WalletV1V2Data {
    pub seqno: u32,
    pub public_key: TonHash,
}

impl WalletV1V2Data {
    pub fn new(public_key: TonHash) -> Self { Self { seqno: 0, public_key } }
}

/// https://docs.ton.org/participate/wallets/contracts#wallet-v2
/// TLB parser doesn't expect signature in cell, check `read_signed()` instead
#[derive(Debug, PartialEq, Clone)]
pub struct WalletV2ExtMsgBody {
    pub msg_seqno: u32,
    pub valid_until: u32,
    pub msgs_modes: Vec<u8>,
    pub msgs: Vec<TonCellRef>,
}

impl TLB for WalletV2ExtMsgBody {
    fn read_definition(parser: &mut CellParser) -> Result<Self, TonCoreError> {
        let msg_seqno = TLB::read(parser)?;
        let valid_until = TLB::read(parser)?;
        let (msgs_modes, msgs) = read_up_to_4_msgs(parser)?;
        Ok(Self {
            msg_seqno,
            valid_until,
            msgs_modes,
            msgs,
        })
    }

    fn write_definition(&self, dst: &mut CellBuilder) -> Result<(), TonCoreError> {
        self.msg_seqno.write(dst)?;
        self.valid_until.write(dst)?;
        write_up_to_4_msgs(dst, &self.msgs, &self.msgs_modes)?;
        Ok(())
    }
}

impl WalletV2ExtMsgBody {
    pub fn read_signed(parser: &mut CellParser) -> Result<(Self, Vec<u8>), TonCoreError> {
        let signature = parser.read_bits(512)?;
        Ok((Self::read(parser)?, signature))
    }
}
