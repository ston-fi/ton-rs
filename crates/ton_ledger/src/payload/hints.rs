//! Ordered Ledger field mappings and complete hint encoding.
mod dns;
mod vesting;
use super::{exact, message::SupportedMessage, tlb::*};
use crate::{
    error::{TonLedgerError, TonLedgerResult},
    protocol,
    signing::SigningPolicy,
};
use ton::contracts::tep::{
    jetton::{jetton_burn_msg::JettonBurnMsg, jetton_transfer_msg::JettonTransferMsg},
    nft::nft_transfer_msg::NFTTransferMsg,
};
use ton::ton_core::{cell::TonCell, errors::TonCoreError};

pub(super) struct Hint {
    pub id: u32,
    pub data: Vec<u8>,
}

pub(super) trait LedgerHintEncode {
    fn encode_hint(&self, policy: SigningPolicy) -> TonLedgerResult<Hint>;
}

pub(super) fn encode(cell: &TonCell, policy: SigningPolicy) -> TonLedgerResult<Vec<u8>> {
    let hint = exact::<SupportedMessage>(cell).and_then(|message| message.encode_hint(policy));
    match hint {
        Ok(Hint { id, data }) => {
            let length = u16::try_from(data.len()).map_err(|_| TonLedgerError::Invalid("hint exceeds 65535 bytes"))?;
            let mut output = vec![1];
            output.extend(id.to_be_bytes());
            output.extend(length.to_be_bytes());
            output.extend(data);
            Ok(output)
        },
        Err(_) if policy == SigningPolicy::AllowOpaque => Ok(vec![0]),
        Err(TonLedgerError::Cell(TonCoreError::TLBEnumOutOfOptions { .. })) => Err(TonLedgerError::OpaquePayload),
        Err(error) => Err(error),
    }
}

// Unsupported nested record types are opaque; truncated input remains a cell error.
fn unsupported_record(error: TonLedgerError) -> TonLedgerError {
    match error {
        TonLedgerError::Cell(TonCoreError::TLBWrongPrefix {
            bits_exp, bits_left, ..
        }) if bits_left >= bits_exp => TonLedgerError::OpaquePayload,
        TonLedgerError::Cell(TonCoreError::TLBEnumOutOfOptions { .. }) => TonLedgerError::OpaquePayload,
        error => error,
    }
}

impl LedgerHintEncode for Comment {
    fn encode_hint(&self, _policy: SigningPolicy) -> TonLedgerResult<Hint> {
        let mut parser = self.content.parser();
        let bit_count = parser.data_bits_left()?;
        if bit_count % 8 != 0 {
            return Err(TonLedgerError::OpaquePayload);
        }
        let data = parser.read_bits(bit_count)?;
        parser.ensure_empty()?;
        protocol::printable(&data, 120)?;
        Ok(Hint { id: 0, data })
    }
}

// Field order is the firmware wire contract. Adapters own validation and policy;
// this macro only sequences their calls and propagates errors.
macro_rules! impl_ledger_hint {
    ($ty:ty, $hint_id:expr, { $( $field:ident => $encoder:ident ),* $(,)? }) => {
        impl LedgerHintEncode for $ty {
            fn encode_hint(&self, policy: SigningPolicy) -> TonLedgerResult<Hint> {
                let mut output = Vec::new();
                $( super::encoding::$encoder(&mut output, &self.$field, policy)?; )*
                Ok(Hint { id: $hint_id, data: output })
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
