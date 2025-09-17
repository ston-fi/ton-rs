use crate::block_tlb::*;
use ton_lib_core::cell::TonHash;
use ton_lib_core::types::tlb_core::{MsgAddress, MsgAddressExt, MsgAddressInt, MsgAddressIntStd};
use ton_lib_core::TLB;

// https://github.com/ton-blockchain/ton/blob/050a984163a53df16fb03f66cc445c34bfed48ed/crypto/block/block.tlb#L155
#[derive(Debug, Clone, PartialEq, TLB)]
pub enum CommonMsgInfo {
    Int(CommonMsgInfoInt),
    ExtIn(CommonMsgInfoExtIn),
    ExtOut(CommonMsgInfoExtOut), // is not tested
}

#[derive(Clone, Debug, PartialEq, TLB)]
#[tlb(prefix = 0b10, bits_len = 2)]
pub struct CommonMsgInfoExtIn {
    pub src: MsgAddressExt,
    pub dst: MsgAddressInt,
    pub import_fee: Coins,
}

#[derive(Clone, Debug, PartialEq, TLB)]
#[tlb(prefix = 0b0, bits_len = 1)]
pub struct CommonMsgInfoInt {
    pub ihr_disabled: bool,
    pub bounce: bool,
    pub bounced: bool,
    pub src: MsgAddress, // it's MsgAddressInt in tlb, but in fact it can be at least MsgAddressNone
    pub dst: MsgAddress, // the same
    pub value: CurrencyCollection,
    pub ihr_fee: Coins,
    pub fwd_fee: Coins,
    pub created_lt: u64,
    pub created_at: u32,
}

#[derive(Clone, Debug, PartialEq, TLB)]
#[tlb(prefix = 0b11, bits_len = 2)]
pub struct CommonMsgInfoExtOut {
    pub src: MsgAddressInt,
    pub dst: MsgAddressExt,
    pub created_lt: u64,
    pub created_at: u32,
}

impl Default for CommonMsgInfoInt {
    fn default() -> Self {
        CommonMsgInfoInt {
            ihr_disabled: false,
            bounce: false,
            bounced: false,
            src: MsgAddressIntStd {
                anycast: None,
                workchain: -1,
                address: TonHash::ZERO,
            }
            .into(),
            dst: MsgAddressIntStd {
                anycast: None,
                workchain: -1,
                address: TonHash::ZERO,
            }
            .into(),
            value: CurrencyCollection::new(0u32),
            ihr_fee: Coins::ZERO,
            fwd_fee: Coins::ZERO,
            created_lt: 0,
            created_at: 0,
        }
    }
}

impl Default for CommonMsgInfo {
    fn default() -> Self { CommonMsgInfoInt::default().into() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_tlb_common_msg_info_enum_derive() -> anyhow::Result<()> {
        let mut common_msg_info = CommonMsgInfo::Int(CommonMsgInfoInt {
            ihr_disabled: false,
            bounce: false,
            bounced: false,
            src: MsgAddress::Int(MsgAddressInt::Std(MsgAddressIntStd {
                anycast: None,
                workchain: 0,
                address: TonHash::ZERO,
            })),
            dst: MsgAddress::NONE,
            value: CurrencyCollection::new(0u32),
            ihr_fee: Coins::ZERO,
            fwd_fee: Coins::ZERO,
            created_lt: 0,
            created_at: 0,
        });
        assert!(common_msg_info.as_int().is_some());
        assert!(common_msg_info.as_ext_in().is_none());
        assert!(common_msg_info.as_ext_in_mut().is_none());

        let int = common_msg_info.as_int().unwrap();
        let common_msg_info_from = CommonMsgInfo::from(int.clone());
        assert_eq!(common_msg_info_from, common_msg_info);
        Ok(())
    }
}
