use crate::wallet::wallet_tlb::wallet_ext_msg_utils::{read_up_to_4_msgs, write_up_to_4_msgs};
use ton_lib_core::cell::{CellBuilder, CellParser, TonCellRef, TonHash};
use ton_lib_core::errors::TonCoreError;
use ton_lib_core::traits::tlb::TLB;
use ton_lib_core::{bail_ton_core, TLB};

#[derive(Debug, PartialEq, Clone, TLB)]
pub struct WalletV4Data {
    pub seqno: u32,
    pub wallet_id: i32,
    pub public_key: TonHash,
    pub plugins: Option<TonCellRef>,
}

impl WalletV4Data {
    pub fn new(wallet_id: i32, public_key: TonHash) -> Self {
        Self {
            seqno: 0,
            wallet_id,
            public_key,
            plugins: None,
        }
    }
}

/// https://docs.ton.org/participate/wallets/contracts#wallet-v4
/// signature is not considered as part of msg body
#[derive(Debug, PartialEq, Clone)]
pub struct WalletV4ExtMsgBody {
    pub subwallet_id: i32,
    pub valid_until: u32,
    pub msg_seqno: u32,
    pub opcode: u8,
    pub msgs_modes: Vec<u8>,
    pub msgs: Vec<TonCellRef>,
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
    pub fn read_signed(parser: &mut CellParser) -> Result<(Self, Vec<u8>), TonCoreError> {
        let signature = parser.read_bits(512)?;
        Ok((Self::read(parser)?, signature))
    }
}

#[cfg(test)]
mod tests {
    use crate::wallet::{WalletV4Data, WalletV4ExtMsgBody, WALLET_DEFAULT_ID};
    use std::str::FromStr;
    use ton_lib_core::cell::{TonCell, TonHash};
    use ton_lib_core::traits::tlb::TLB;

    #[test]
    fn test_wallet_data_v4() -> anyhow::Result<()> {
        // https://tonviewer.com/UQCS65EGyiApUTLOYXDs4jOLoQNCE0o8oNnkmfIcm0iX5FRT
        let src_boc_hex = "b5ee9c7241010101002b0000510000001429a9a317cbf377c9b73604c70bf73488ddceba14f763baef2ac70f68d1d6032a120149f440a6c9f37d";
        let wallet_data = WalletV4Data::from_boc_hex(src_boc_hex)?;
        assert_eq!(wallet_data.seqno, 20);
        assert_eq!(wallet_data.wallet_id, WALLET_DEFAULT_ID);
        assert_eq!(
            wallet_data.public_key,
            TonHash::from_str("cbf377c9b73604c70bf73488ddceba14f763baef2ac70f68d1d6032a120149f4")?
        );
        assert_eq!(wallet_data.plugins, None);

        let serial_boc_hex = wallet_data.to_boc_hex()?;
        let restored = WalletV4Data::from_boc_hex(&serial_boc_hex)?;
        assert_eq!(wallet_data, restored);
        Ok(())
    }

    #[test]
    fn test_wallet_ext_msg_body_v4() -> anyhow::Result<()> {
        // https://tonviewer.com/transaction/891dbceffb986251768d4c33bb8dcf11d522408ff78b8e683d135304ca377b8b
        let body_signed_cell = TonCell::from_boc_hex("b5ee9c7201010201008700019c9dcd3a68926ad6fb9d094c5b72901bfc359ada50f22b648c6c2223c767135d397c7489c121071e45a5316a94a533d80c41450049ebeed406c419fea99117f40629a9a31767ad328900000013000301006842007847b4630eb08d9f486fe846d5496878556dfd5a084f82a9a3fb01224e67c84c200989680000000000000000000000000000")?;
        let mut parser = body_signed_cell.parser();
        parser.read_bits(512)?;
        let body_no_sign = parser.read_cell()?;

        let body = WalletV4ExtMsgBody::read_signed(&mut body_signed_cell.parser())?.0;
        assert_eq!(body.subwallet_id, WALLET_DEFAULT_ID);
        assert_eq!(body.valid_until, 1739403913);
        assert_eq!(body.msg_seqno, 19);
        assert_eq!(body.opcode, 0);
        assert_eq!(body.msgs_modes, vec![3]);
        assert_eq!(body.msgs.len(), 1);

        let serial_cell = body.to_cell()?;
        assert_eq!(body_no_sign, serial_cell);
        Ok(())
    }
}
