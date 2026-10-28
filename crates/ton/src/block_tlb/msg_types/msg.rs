use crate::block_tlb::msg_types::common_msg_info::CommonMsgInfo;
use crate::block_tlb::*;
use ton_core::TLB;
use ton_core::cell::{TonCell, TonHash};
use ton_core::errors::TonCoreError;
use ton_core::traits::tlb::TLB;
use ton_core::types::tlb_core::*;

// https://github.com/ton-blockchain/ton/blob/050a984163a53df16fb03f66cc445c34bfed48ed/crypto/block/block.tlb#L157
// Use TonAddress::from_msg_address / TonAddress::to_msg_address to operate with User-friendly addresses
#[derive(Debug, Clone, PartialEq, TLB)]
pub struct Msg<Body: TLB = TonCell> {
    pub info: CommonMsgInfo,
    pub init: Option<TLBEitherRef<StateInit>>,
    pub body: TLBEitherRef<Body>,
}

impl<Body: TLB> Msg<Body> {
    /// Creates a message without state init and stores the body in a reference.
    pub fn new<T: Into<CommonMsgInfo>>(info: T, body: Body) -> Self {
        Self {
            info: info.into(),
            init: None,
            body: TLBEitherRef::new_with_layout(body, EitherRefLayout::ToRef),
        }
    }

    pub fn src(&self) -> MsgAddress {
        match &self.info {
            CommonMsgInfo::Int(info) => info.src.clone(),
            CommonMsgInfo::ExtIn(info) => info.src.clone().into(),
            CommonMsgInfo::ExtOut(info) => info.src.clone().into(),
        }
    }

    pub fn dst(&self) -> MsgAddress {
        match &self.info {
            CommonMsgInfo::Int(info) => info.dst.clone(),
            CommonMsgInfo::ExtIn(info) => info.dst.clone().into(),
            CommonMsgInfo::ExtOut(info) => info.dst.clone().into(),
        }
    }

    pub fn created_at(&self) -> Option<u32> {
        match &self.info {
            CommonMsgInfo::Int(info) => Some(info.created_at),
            CommonMsgInfo::ExtIn(_) => None,
            CommonMsgInfo::ExtOut(info) => Some(info.created_at),
        }
    }

    pub fn created_lt(&self) -> Option<u64> {
        match &self.info {
            CommonMsgInfo::Int(info) => Some(info.created_lt),
            CommonMsgInfo::ExtIn(_) => None,
            CommonMsgInfo::ExtOut(info) => Some(info.created_lt),
        }
    }

    pub fn state_init(&self) -> Option<&StateInit> {
        self.init.as_ref().map(|init| &init.value)
    }

    pub fn cell_hash_normalized(&self) -> Result<TonHash, TonCoreError> {
        match &self.info {
            CommonMsgInfo::ExtIn(_) => {
                let mut msg_normalized = Msg {
                    info: self.info.clone(),
                    init: self.init.clone(),
                    body: TLBEitherRef::new_with_layout(self.body.value.to_cell()?, self.body.layout),
                };
                let CommonMsgInfo::ExtIn(info) = &mut msg_normalized.info else { unreachable!() };
                info.src = MsgAddressExt::NONE;
                match &mut info.dst {
                    MsgAddressInt::Std(addr) => addr.anycast = None,
                    MsgAddressInt::Var(addr) => addr.anycast = None,
                    _ => return Err(TonCoreError::Custom("unsupported internal address layout".to_string())),
                }
                info.import_fee = TLBCoins::ZERO;
                msg_normalized.init = None;
                msg_normalized.body.layout = EitherRefLayout::ToRef;
                msg_normalized.cell_hash()
            },
            _ => self.cell_hash(),
        }
    }
}

#[cfg(test)]
#[path = "../../tests/block_tlb/msg_types/test_msg.rs"]
mod tests;
