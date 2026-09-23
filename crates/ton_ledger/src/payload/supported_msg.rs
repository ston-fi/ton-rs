//! Supported payloads are selected by their existing TLB prefixes.
use super::{
    hints::{Hint, LedgerHintEncode},
    tlb::*,
};
use crate::{error::TonLedgerResult, signing::SigningPolicy};
use ton::{
    contracts::tep::jetton::{jetton_burn_msg::JettonBurnMsg, jetton_transfer_msg::JettonTransferMsg},
    ton_core::TLB,
};

#[derive(TLB)]
pub(super) enum LedgerSupportedMsg {
    Comment(Comment),
    JettonTransfer(JettonTransferMsg),
    NftTransfer(NftTransfer),
    JettonBurn(JettonBurnMsg),
    Whitelist(Whitelist),
    Withdraw(Withdraw),
    Validator(Validator),
    TonstakersDeposit(TonstakersDeposit),
    Vote(Vote),
    DnsChangeRecord(DnsChangeRecord),
    Swap(Swap),
    WhalesDeposit(WhalesDeposit),
    WhalesWithdraw(WhalesWithdraw),
    Vesting(Vesting),
}

impl LedgerHintEncode for LedgerSupportedMsg {
    fn encode_hint(&self, policy: SigningPolicy) -> TonLedgerResult<Hint> {
        match self {
            Self::Comment(message) => message.encode_hint(policy),
            Self::JettonTransfer(message) => message.encode_hint(policy),
            Self::NftTransfer(message) => message.0.encode_hint(policy),
            Self::JettonBurn(message) => message.encode_hint(policy),
            Self::Whitelist(message) => message.encode_hint(policy),
            Self::Withdraw(message) => message.encode_hint(policy),
            Self::Validator(message) => message.encode_hint(policy),
            Self::TonstakersDeposit(message) => message.encode_hint(policy),
            Self::Vote(message) => message.encode_hint(policy),
            Self::DnsChangeRecord(message) => message.encode_hint(policy),
            Self::Swap(message) => message.encode_hint(policy),
            Self::WhalesDeposit(message) => message.encode_hint(policy),
            Self::WhalesWithdraw(message) => message.encode_hint(policy),
            Self::Vesting(message) => message.encode_hint(policy),
        }
    }
}
