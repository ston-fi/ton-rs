use crate::block_tlb::Coins;
use ton_lib_core::cell::{TonCell, TonCellRef};
use ton_lib_core::types::tlb_core::TLBEitherRef;
use ton_lib_core::types::TonAddress;
use ton_lib_core::TLB;

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
    pub custom_payload: Option<TonCellRef>,
    pub forward_ton_amount: Coins, // the amount of nanotons to be sent to the destination address.
    pub forward_payload: TLBEitherRef<TonCell>, // optional custom data that should be sent to the destination address.
}

impl NFTTransferMsg {
    pub fn new(new_owner: &TonAddress) -> Self {
        NFTTransferMsg {
            query_id: 0,
            new_owner: new_owner.clone(),
            response_dst: TonAddress::ZERO,
            custom_payload: None,
            forward_ton_amount: Coins::ZERO,
            forward_payload: TLBEitherRef::new(TonCell::EMPTY),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::block_tlb::Coins;
    use crate::tep::nft::nft_transfer_msg::NFTTransferMsg;
    use std::str::FromStr;
    use ton_lib_core::cell::TonCell;
    use ton_lib_core::traits::tlb::TLB;
    use ton_lib_core::types::tlb_core::TLBEitherRef;
    use ton_lib_core::types::TonAddress;

    #[test]
    fn test_nft_transfer_msg() -> anyhow::Result<()> {
        let nft_transfer_msg = NFTTransferMsg::from_boc_hex("b5ee9c7201010101006f0000d95fcc3d140000000000000000800e20aaf07ad251d1800fe45e3af334769b7b2069d3ab2ea6c9ee0f73dfd072a21000a1b4b24b6a66313f3e0b49d095f3e8f4294af504b3a0f7b99290129f3aaafcc47312d0040544f4e506c616e65747320676966742077697468206c6f76658")?;
        let payload =
            TonCell::from_boc_hex("b5ee9c7201010101001c00003440544f4e506c616e65747320676966742077697468206c6f7665")?;

        let expected = NFTTransferMsg {
            query_id: 0,
            new_owner: TonAddress::from_str("0:71055783d6928e8c007f22f1d799a3b4dbd9034e9d5975364f707b9efe839510")?,
            response_dst: TonAddress::from_str("0:286d2c92da998c4fcf82d274257cfa3d0a52bd412ce83dee64a404a7ceaabf31")?,
            custom_payload: None,
            forward_ton_amount: Coins::new(10000000u64),
            forward_payload: TLBEitherRef::new(payload),
        };

        assert_eq!(nft_transfer_msg, expected);

        let serialized = nft_transfer_msg.to_boc()?;
        let parsed_back = NFTTransferMsg::from_boc(&serialized)?;
        assert_eq!(parsed_back, expected);
        Ok(())
    }
}
