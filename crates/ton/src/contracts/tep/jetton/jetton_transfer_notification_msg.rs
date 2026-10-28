//! Jetton transfer notification message.

use ton_core::TLB;
use ton_core::cell::TonCell;
use ton_core::traits::tlb::TLB;
use ton_core::types::TonAddress;
use ton_core::types::tlb_core::TLBCoins;
use ton_core::types::tlb_core::TLBEitherRef;

/// ```raw
/// transfer_notification#7362d09c query_id:uint64 amount:(VarUInteger 16)
/// sender:MsgAddress forward_payload:(Either Cell ^Cell)
/// = InternalMsgBody;
/// ```
#[derive(Clone, Debug, PartialEq, TLB)]
#[tlb(prefix = 0x7362d09c, bits_len = 32, ensure_empty = true)]
pub struct JettonTransferNotificationMsg<ForwardPayloadT: TLB = TonCell> {
    pub query_id: u64,                                  // should be equal with request's query_id
    pub amount: TLBCoins,                               // amount of transferred jettons
    pub sender: TonAddress,                             // is address of the previous owner of transferred jettons
    pub forward_payload: TLBEitherRef<ForwardPayloadT>, //  optional custom data that should be sent to the destination address.
}

impl<T: TLB> JettonTransferNotificationMsg<T> {
    /// Creates a notification with query ID zero and an automatically laid-out payload.
    pub fn new<C: Into<TLBCoins>>(amount: C, sender: TonAddress, forward_payload: T) -> Self {
        JettonTransferNotificationMsg {
            query_id: 0,
            amount: amount.into(),
            sender,
            forward_payload: TLBEitherRef::new(forward_payload),
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/contracts/tep/jetton/test_jetton_transfer_notification_msg.rs"]
mod tests;
