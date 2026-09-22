//! Ordered Ledger field mappings for existing TON message types.
use super::tlb::*;
use crate::{error::TonLedgerResult, signing::SigningPolicy};
use ton::contracts::tep::{
    jetton::{jetton_burn_msg::JettonBurnMsg, jetton_transfer_msg::JettonTransferMsg},
    nft::nft_transfer_msg::NFTTransferMsg,
};

pub(super) trait LedgerHintEncode {
    const HINT_ID: u32;

    fn encode_hint(&self, policy: SigningPolicy) -> TonLedgerResult<Vec<u8>>;
}

// Field order is the firmware wire contract. Adapters own validation and policy;
// this macro only sequences their calls and propagates errors.
macro_rules! impl_ledger_hint {
    ($ty:ty, $hint_id:expr, { $( $field:ident => $encoder:ident ),* $(,)? }) => {
        impl LedgerHintEncode for $ty {
            const HINT_ID: u32 = $hint_id;

            fn encode_hint(&self, policy: SigningPolicy) -> TonLedgerResult<Vec<u8>> {
                let mut output = Vec::new();
                $( super::encoding::$encoder(&mut output, &self.$field, policy)?; )*
                Ok(output)
            }
        }
    };
}

impl_ledger_hint!(JettonTransferMsg, 1, {
    query_id => query_id,
    amount => coins,
    dst => address,
    response_dst => address,
    custom_payload => custom_payload,
    forward_ton_amount => coins,
    forward_payload => forward_payload,
});

impl_ledger_hint!(NFTTransferMsg, 2, {
    query_id => query_id,
    new_owner => ton_address,
    response_dst => ton_address,
    custom_payload => custom_payload,
    forward_ton_amount => coins,
    forward_payload => forward_payload,
});

impl_ledger_hint!(JettonBurnMsg, 3, {
    query_id => query_id,
    amount => coins,
    response_dst => address,
    custom_payload => burn_custom_payload,
});

impl_ledger_hint!(Whitelist, 4, {
    query => query_id,
    address => address,
});

impl_ledger_hint!(Withdraw, 5, {
    query => query_id,
    amount => coins,
});

impl_ledger_hint!(Validator, 6, {
    query => query_id,
    address => address,
});

impl_ledger_hint!(TonstakersDeposit, 7, {
    query => query_id,
    app => trailing_app_id,
});

impl_ledger_hint!(Vote, 8, {
    query => query_id,
    address => address,
    expiration => uint48,
    vote => boolean,
    confirm => boolean,
});

impl_ledger_hint!(Swap, 10, {
    query => query_id,
    id => hash,
});

impl_ledger_hint!(WhalesDeposit, 11, {
    query => positive_query_id,
    gas => coins,
});

impl_ledger_hint!(WhalesWithdraw, 12, {
    query => positive_query_id,
    gas => coins,
    amount => coins,
});
