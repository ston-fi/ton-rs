use crate::wallet::wallet_tlb::wallet_ext_msg_utils::{build_inner_request, parse_inner_request, InnerRequest};
use ton_lib_core::cell::{CellBuilder, CellParser, TonCellRef, TonHash};
use ton_lib_core::errors::TonCoreError;
use ton_lib_core::traits::tlb::{TLBPrefix, TLB};
use ton_lib_core::TLB;

/// WalletVersion::V5R1
/// https://github.com/ton-blockchain/wallet-contract-v5/blob/main/types.tlb#L29
#[derive(Debug, PartialEq, Clone, TLB)]
pub struct WalletV5Data {
    pub sign_allowed: bool,
    pub seqno: u32,
    pub wallet_id: i32,
    pub public_key: TonHash,
    pub extensions: Option<TonCellRef>,
}

impl WalletV5Data {
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

/// https://docs.ton.org/participate/wallets/contracts#wallet-v5
/// signature is not considered as part of msg body
/// https://github.com/ton-blockchain/wallet-contract-v5/blob/main/types.tlb
/// This implementation support only jetton transfer messages
#[derive(Debug, PartialEq, Clone)]
pub struct WalletV5ExtMsgBody {
    pub wallet_id: i32,
    pub valid_until: u32,
    pub msg_seqno: u32,
    pub msgs_modes: Vec<u8>,
    pub msgs: Vec<TonCellRef>,
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
    pub fn read_signed(parser: &mut CellParser) -> Result<(Self, Vec<u8>), TonCoreError> {
        let body = Self::read(parser)?;
        let signature = parser.read_bits(512)?;
        Ok((body, signature))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::wallet::{WALLET_V5R1_DEFAULT_ID, WALLET_V5R1_DEFAULT_ID_TESTNET};
    use std::str::FromStr;
    use ton_lib_core::cell::TonCell;

    #[test]
    fn test_wallet_data_v5() -> anyhow::Result<()> {
        // https://tonviewer.com/UQDwj2jGHWEbPpDf0I2qktDwqtv6tBCfBVNH9gJEnM-QmHDa
        let src_boc_hex = "b5ee9c7241010101002b00005180000000bfffff88e5f9bbe4db9b026385fb9a446ee75d0a7bb1dd77956387b468eb01950900a4fa20cbe13a2a";
        let wallet_data = WalletV5Data::from_boc_hex(src_boc_hex)?;
        assert_eq!(wallet_data.seqno, 1);
        assert_eq!(wallet_data.wallet_id, WALLET_V5R1_DEFAULT_ID);
        assert_eq!(
            wallet_data.public_key,
            TonHash::from_str("cbf377c9b73604c70bf73488ddceba14f763baef2ac70f68d1d6032a120149f4")?
        );
        assert_eq!(wallet_data.extensions, None);

        let serial_boc_hex = wallet_data.to_boc_hex_extra(true)?;
        assert_eq!(src_boc_hex, serial_boc_hex);
        let restored = WalletV5Data::from_boc_hex(&serial_boc_hex)?;
        assert_eq!(wallet_data, restored);
        Ok(())
    }

    #[test]
    fn test_wallet_data_v5_testnet() -> anyhow::Result<()> {
        let src_boc_hex = "b5ee9c7201010101002b000051800000013ffffffed2b31b23dbe5144a626b9d5d1d4208e36d97e4adb472d42c073bfff85b3107e4a0";
        let wallet_data = WalletV5Data::from_boc_hex(src_boc_hex)?;
        assert_eq!(wallet_data.seqno, 2);
        assert_eq!(wallet_data.wallet_id, WALLET_V5R1_DEFAULT_ID_TESTNET);
        Ok(())
    }

    #[test]
    fn test_wallet_ext_msg_body_v5() -> anyhow::Result<()> {
        // https://tonviewer.com/transaction/b4c5eddc52d0e23dafb2da6d022a5b6ae7eba52876fa75d32b2a95fa30c7e2f0
        let body_signed_cell = TonCell::from_boc_hex("b5ee9c720101040100940001a17369676e7fffff11ffffffff00000000bc04889cb28b36a3a00810e363a413763ec34860bf0fce552c5d36e37289fafd442f1983d740f92378919d969dd530aec92d258a0779fb371d4659f10ca1b3826001020a0ec3c86d030302006642007847b4630eb08d9f486fe846d5496878556dfd5a084f82a9a3fb01224e67c84c187a1200000000000000000000000000000000")?;
        let mut parser = body_signed_cell.parser();
        parser.read_bits(body_signed_cell.data_bits_len - 512)?;
        let sign = parser.read_bits(512)?;

        let body = WalletV5ExtMsgBody::read_signed(&mut body_signed_cell.parser())?.0;

        assert_eq!(body.wallet_id, WALLET_V5R1_DEFAULT_ID);
        assert_eq!(body.valid_until, 4294967295);
        assert_eq!(body.msg_seqno, 0);
        assert_eq!(body.msgs_modes, vec![3]);
        assert_eq!(body.msgs.len(), 1);

        let mut signed_builder = TonCell::builder();
        signed_builder.write_cell(&body.to_cell()?)?;
        signed_builder.write_bits(&sign, 512)?;
        let signed_serial = signed_builder.build()?;
        assert_eq!(body_signed_cell, signed_serial);
        let parsed_back = WalletV5ExtMsgBody::from_cell(&signed_serial)?;
        assert_eq!(body, parsed_back);
        Ok(())
    }
}
