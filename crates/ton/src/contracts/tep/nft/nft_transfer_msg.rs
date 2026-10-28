//! NFT transfer request message.

use ton_core::TLB;
use ton_core::cell::TonCell;
use ton_core::types::TonAddress;
use ton_core::types::tlb_core::TLBCoins;
use ton_core::types::tlb_core::{TLBEitherRef, TLBRef};

/// Creates a body for jetton transfer according to TL-B schema:
///
/// ```raw
/// transfer#5fcc3d14
///   query_id:uint64
///   new_owner:MsgAddress
///   response_destination:MsgAddress
///   custom_payload:(Maybe ^Cell)
///   forward_amount:(VarUInteger 16)
///   forward_payload:(Either Cell ^Cell)
/// = InternalMsgBody;
/// ```
#[derive(Clone, Debug, PartialEq, TLB)]
#[tlb(prefix = 0x5fcc3d14, bits_len = 32, ensure_empty = true)]
pub struct NFTTransferMsg {
    pub query_id: u64,
    pub new_owner: TonAddress,    // address of the new owner of the NFT item.
    pub response_dst: TonAddress, //  address where to send a response with confirmation of a successful transfer and the rest of the incoming message coins.
    pub custom_payload: Option<TLBRef<TonCell>>,
    pub forward_ton_amount: TLBCoins, // the amount of nanotons to be sent to the destination address.
    pub forward_payload: TLBEitherRef<TonCell>, // optional custom data that should be sent to the destination address.
}

impl NFTTransferMsg {
    /// Creates a transfer with query ID zero, zero response address/amount, and empty payloads.
    pub fn new(new_owner: &TonAddress) -> Self {
        NFTTransferMsg {
            query_id: 0,
            new_owner: *new_owner,
            response_dst: TonAddress::ZERO,
            custom_payload: None,
            forward_ton_amount: TLBCoins::ZERO,
            forward_payload: TLBEitherRef::new(TonCell::empty().to_owned()),
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/contracts/tep/nft/test_nft_transfer_msg.rs"]
mod tests;
