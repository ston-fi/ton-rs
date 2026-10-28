//! Jetton transfer request message.

use ton_core::TLB;
use ton_core::cell::TonCell;
use ton_core::traits::tlb::TLB;
use ton_core::types::tlb_core::TLBCoins;
use ton_core::types::tlb_core::{MsgAddress, MsgAddressInt, TLBEitherRef, TLBRef};

///```raw
/// transfer#f8a7ea5 query_id:uint64 amount:(VarUInteger 16) destination:MsgAddress
/// response_destination:MsgAddress custom_payload:(Maybe ^Cell)
/// forward_ton_amount:(VarUInteger 16) forward_payload:(Either Cell ^Cell)
/// = InternalMsgBody;
/// ```

#[derive(Clone, Debug, PartialEq, TLB)]
#[tlb(prefix = 0xf8a7ea5, bits_len = 32)]
pub struct JettonTransferMsg<CustomPayloadT: TLB = TonCell, ForwardPayloadT: TLB = TonCell> {
    pub query_id: u64,                                  // arbitrary number to identify the transfer
    pub amount: TLBCoins,                               // amount of transferred jettons in elementary units
    pub dst: MsgAddress,                                // new jetton owner
    pub response_dst: MsgAddress, // address where to send a response with confirmation of a successful transfer and the rest of the incoming message Toncoins.
    pub custom_payload: Option<TLBRef<CustomPayloadT>>, // optional custom data (which is used by either sender or receiver jetton ton_wallet for inner logic).
    pub forward_ton_amount: TLBCoins, // the amount of nano-tons to be sent to the destination address.
    pub forward_payload: TLBEitherRef<ForwardPayloadT>, // data will be sent to the destination address "as is".
}

impl<CustomPayload: TLB, ForwardPayloadT: TLB> JettonTransferMsg<CustomPayload, ForwardPayloadT> {
    /// Creates a transfer with query ID zero, no response/custom payload, and zero forwarded TON.
    pub fn new<C: Into<TLBCoins>>(dst: MsgAddressInt, amount: C, payload: ForwardPayloadT) -> Self {
        JettonTransferMsg {
            query_id: 0,
            amount: amount.into(),
            dst: dst.into(),
            response_dst: MsgAddress::NONE,
            custom_payload: None,
            forward_ton_amount: TLBCoins::ZERO,
            forward_payload: TLBEitherRef::new(payload),
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/contracts/tep/jetton/test_jetton_transfer_msg.rs"]
mod tests;
