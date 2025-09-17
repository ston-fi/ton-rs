use super::*;
use crate::block_tlb::*;
use crate::errors::TonError;
use nacl::sign::signature;
use ton_lib_core::cell::{TonCell, TonCellRef};
use ton_lib_core::traits::tlb::TLB;
use ton_lib_core::types::tlb_core::{MsgAddressExt, TLBEitherRef};
use ton_lib_core::types::TonAddress;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct TonWallet {
    pub version: WalletVersion,
    pub key_pair: KeyPair,
    pub address: TonAddress,
    pub wallet_id: i32,
}

impl TonWallet {
    pub fn new(version: WalletVersion, key_pair: KeyPair) -> Result<Self, TonError> {
        let wallet_id = match version {
            WalletVersion::V5R1 => WALLET_V5R1_DEFAULT_ID,
            _ => WALLET_DEFAULT_ID,
        };
        Self::new_with_params(version, key_pair, 0, wallet_id)
    }

    pub fn new_with_creds(version: WalletVersion, seed: &str, pass: Option<String>) -> Result<Self, TonError> {
        Self::new(version, Mnemonic::from_str(seed, pass)?.to_key_pair()?)
    }

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

    pub fn create_ext_in_msg(
        &self,
        int_msgs: Vec<TonCellRef>,
        seqno: u32,
        expire_at: u32,
        add_state_init: bool,
    ) -> Result<TonCell, TonError> {
        let body = self.create_ext_in_body(expire_at, seqno, int_msgs)?;
        let signed = self.sign_ext_in_body(&body)?;
        let external = self.create_ext_in_msg_from_body(signed, add_state_init)?;
        Ok(external)
    }

    pub fn create_ext_in_body(
        &self,
        expire_at: u32,
        seqno: u32,
        int_msgs: Vec<TonCellRef>,
    ) -> Result<TonCell, TonError> {
        WalletVersion::build_ext_in_body(self.version, expire_at, seqno, self.wallet_id, int_msgs)
    }

    pub fn sign_ext_in_body(&self, ext_in_body: &TonCell) -> Result<TonCell, TonError> {
        let message_hash = ext_in_body.cell_hash()?;
        let sign = match signature(message_hash.as_slice(), self.key_pair.secret_key.as_slice()) {
            Ok(signature) => signature,
            Err(err) => return Err(TonError::Custom(format!("{err:?}"))),
        };
        WalletVersion::sign_msg(self.version, ext_in_body, &sign)
    }

    pub fn create_ext_in_msg_from_body(&self, signed_body: TonCell, add_state_init: bool) -> Result<TonCell, TonError> {
        let msg_info = CommonMsgInfo::ExtIn(CommonMsgInfoExtIn {
            src: MsgAddressExt::NONE,
            dst: self.address.to_msg_address_int(),
            import_fee: Coins::ZERO,
        });

        let mut message = Msg::new(msg_info, signed_body);
        if add_state_init {
            let code = WalletVersion::get_code(self.version)?.clone();
            let data = WalletVersion::get_default_data(self.version, &self.key_pair, self.wallet_id)?;
            let state_init = StateInit::new(code, data);
            message.init = Some(TLBEitherRef::new(state_init));
        }
        Ok(message.to_cell()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    const MNEMONIC_STR: &str = "fancy carpet hello mandate penalty trial consider property top vicious exit rebuild tragic profit urban major total month holiday sudden rib gather media vicious";
    const MNEMONIC_STR_V5: &str = "section garden tomato dinner season dice renew length useful spin trade intact use universe what post spike keen mandate behind concert egg doll rug";

    fn make_keypair(mnemonic_str: &str) -> KeyPair {
        let mnemonic = Mnemonic::from_str(mnemonic_str, None).unwrap();
        mnemonic.to_key_pair().unwrap()
    }

    #[test]
    fn test_ton_wallet_create() -> anyhow::Result<()> {
        let key_pair = make_keypair(MNEMONIC_STR);

        let wallet_v3 = TonWallet::new(WalletVersion::V3R1, key_pair.clone())?;
        let expected_v3 = TonAddress::from_str("EQBiMfDMivebQb052Z6yR3jHrmwNhw1kQ5bcAUOBYsK_VPuK")?;
        assert_eq!(wallet_v3.address, expected_v3);

        let wallet_v3r2 = TonWallet::new(WalletVersion::V3R2, key_pair.clone())?;
        let expected_v3r2 = TonAddress::from_str("EQA-RswW9QONn88ziVm4UKnwXDEot5km7GEEXsfie_0TFOCO")?;
        assert_eq!(wallet_v3r2.address, expected_v3r2);

        let wallet_v4r2 = TonWallet::new(WalletVersion::V4R2, key_pair.clone())?;
        let expected_v4r2 = TonAddress::from_str("EQCDM_QGggZ3qMa_f3lRPk4_qLDnLTqdi6OkMAV2NB9r5TG3")?;
        assert_eq!(wallet_v4r2.address, expected_v4r2);

        let key_pair_v5 = make_keypair(MNEMONIC_STR_V5);
        let wallet_v5 = TonWallet::new(WalletVersion::V5R1, key_pair_v5.clone())?;
        let expected_v5 = TonAddress::from_str("UQDv2YSmlrlLH3hLNOVxC8FcQf4F9eGNs4vb2zKma4txo6i3")?;
        assert_eq!(wallet_v5.address, expected_v5);
        Ok(())
    }

    #[test]
    fn test_ton_wallet_debug() -> anyhow::Result<()> {
        let key_pair = KeyPair {
            public_key: vec![1, 2, 3],
            secret_key: vec![4, 5, 6],
        };
        let wallet = TonWallet {
            key_pair,
            version: WalletVersion::V4R2,
            address: TonAddress::from_str("EQBiMfDMivebQb052Z6yR3jHrmwNhw1kQ5bcAUOBYsK_VPuK")?,
            wallet_id: 42,
        };

        let debug_output = format!("{wallet:?}");
        let expected_output = "TonWallet { version: V4R2, key_pair: KeyPair { public_key: [1, 2, 3], secret_key: \"***REDACTED***\" }, address: TonAddress(\"EQBiMfDMivebQb052Z6yR3jHrmwNhw1kQ5bcAUOBYsK_VPuK\"), wallet_id: 42 }";
        assert_eq!(debug_output, expected_output);
        Ok(())
    }

    #[test]
    fn test_ton_wallet_create_external_msg_v3() -> anyhow::Result<()> {
        let key_pair = make_keypair(MNEMONIC_STR);
        let wallet = TonWallet::new(WalletVersion::V3R1, key_pair)?;

        let int_msg = TonCell::builder().build()?.into_ref();

        let ext_body_cell = wallet.create_ext_in_body(13, 7, vec![int_msg.clone()])?;
        let body = WalletV3ExtMsgBody::from_cell(&ext_body_cell)?;
        let expected = WalletV3ExtMsgBody {
            subwallet_id: WALLET_DEFAULT_ID,
            msg_seqno: 7,
            valid_until: 13,
            msgs_modes: vec![3],
            msgs: vec![int_msg.clone()],
        };
        assert_eq!(body, expected);
        Ok(())
    }

    #[test]
    fn test_ton_wallet_create_external_msg_v4() -> anyhow::Result<()> {
        let key_pair = make_keypair(MNEMONIC_STR);
        let wallet = TonWallet::new(WalletVersion::V4R1, key_pair)?;

        let int_msg = TonCell::builder().build()?.into_ref();

        let ext_body_cell = wallet.create_ext_in_body(13, 7, vec![int_msg.clone()])?;
        let body = WalletV4ExtMsgBody::from_cell(&ext_body_cell)?;
        let expected = WalletV4ExtMsgBody {
            subwallet_id: WALLET_DEFAULT_ID,
            msg_seqno: 7,
            opcode: 0,
            valid_until: 13,
            msgs_modes: vec![3],
            msgs: vec![int_msg],
        };
        assert_eq!(body, expected);
        Ok(())
    }

    #[test]
    fn test_ton_wallet_create_external_msg_v5() -> anyhow::Result<()> {
        let key_pair = make_keypair(MNEMONIC_STR_V5);
        let wallet = TonWallet::new(WalletVersion::V5R1, key_pair)?;

        let msgs_cnt = 10usize;
        let mut int_msgs = vec![];
        for i in 0..msgs_cnt as u32 {
            let mut builder = TonCell::builder();
            i.write(&mut builder)?;
            int_msgs.push(builder.build()?.into_ref());
        }
        TonCell::builder().build()?.into_ref();

        let ext_body_cell = wallet.create_ext_in_body(13, 7, int_msgs.clone())?;
        let body = WalletV5ExtMsgBody::from_cell(&ext_body_cell)?;
        let expected = WalletV5ExtMsgBody {
            wallet_id: WALLET_V5R1_DEFAULT_ID,
            msg_seqno: 7,
            valid_until: 13,
            msgs_modes: vec![3; msgs_cnt],
            msgs: int_msgs,
        };
        assert_eq!(body, expected);
        Ok(())
    }

    #[test]
    fn test_ton_wallet_create_external_msg_signed() -> anyhow::Result<()> {
        let key_pair_v3 = make_keypair(MNEMONIC_STR);
        let wallet_v3 = TonWallet::new(WalletVersion::V3R1, key_pair_v3)?;

        let key_pair_v5 = make_keypair(MNEMONIC_STR_V5);
        let wallet_v5 = TonWallet::new(WalletVersion::V5R1, key_pair_v5)?;
        dbg!(&wallet_v5);

        let mut builder = TonCell::builder();
        100u32.write(&mut builder)?;
        let msg = builder.build()?.into_ref();

        for wallet in [wallet_v3, wallet_v5] {
            let body = wallet.create_ext_in_body(1, 3, vec![msg.clone()])?;
            let signed_msg = wallet.sign_ext_in_body(&body)?;

            let mut parser = signed_msg.parser();
            match wallet.version {
                WalletVersion::V5R1 => {
                    // sign in last 512 bits
                    let data_size_bits = signed_msg.data_bits_len - 512;
                    let mut builder = TonCell::builder();
                    builder.write_bits(parser.read_bits(data_size_bits)?.as_slice(), data_size_bits)?;
                    while let Ok(cell_ref) = parser.read_next_ref() {
                        builder.write_ref(cell_ref.clone())?;
                    }
                    assert_eq!(body, builder.build()?)
                }
                _ => {
                    // sign in first 512 bits
                    parser.read_bits(512)?;
                    assert_eq!(body, TonCell::read(&mut parser)?);
                }
            }
        }
        Ok(())
    }
}
