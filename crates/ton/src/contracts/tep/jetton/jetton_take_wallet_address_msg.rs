//! TEP-89 asynchronous Jetton wallet-address response.
use ton_core::{
    TLB,
    cell::TonCell,
    errors::TonCoreResult,
    traits::tlb::TLB as TLBTrait,
    types::tlb_core::{MsgAddress, TLBRef},
};

/// `take_wallet_address#d1735400 query_id:uint64 wallet_address:MsgAddress owner_address:(Maybe ^MsgAddress)`.
///
/// Absence of the owner reference differs from a present reference containing
/// literal none. The wallet can itself be literal none. This exhaustive record
/// describes wire data, not source authentication or request correlation.
///
/// ```
/// use ton::contracts::tep::jetton::jetton_take_wallet_address_msg::JettonTakeWalletAddressMsg;
/// use ton::ton_core::{traits::tlb::TLB, types::tlb_core::MsgAddress};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let reply = JettonTakeWalletAddressMsg {
///     query_id: 7, wallet_address: MsgAddress::NONE, owner_address: None,
/// };
/// let cell = reply.to_cell()?;
/// assert_eq!(JettonTakeWalletAddressMsg::from_cell_strict(&cell)?, reply);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, TLB)]
#[tlb(prefix = 0xd1735400, bits_len = 32)]
pub struct JettonTakeWalletAddressMsg {
    pub query_id: u64,
    pub wallet_address: MsgAddress,
    pub owner_address: Option<TLBRef<MsgAddress>>,
}
impl JettonTakeWalletAddressMsg {
    /// Reads the complete root and included owner reference without ignoring tails.
    ///
    /// # Errors
    /// Returns a typed cell error for malformed/truncated fields, a wrong opcode,
    /// or trailing bits/references in either the root or included owner cell.
    pub fn from_cell_strict(cell: &TonCell) -> TonCoreResult<Self> {
        let mut parser = cell.parser();
        Self::verify_prefix(&mut parser)?;
        let query_id = u64::read(&mut parser)?;
        let wallet_address = MsgAddress::read(&mut parser)?;
        let owner = Option::<TLBRef<TonCell>>::read(&mut parser)?;
        parser.ensure_empty()?;
        let owner_address = owner
            .map(|cell| -> TonCoreResult<_> {
                let mut parser = cell.parser();
                let address = MsgAddress::read(&mut parser)?;
                parser.ensure_empty()?;
                Ok(TLBRef::new(address))
            })
            .transpose()?;
        Ok(Self {
            query_id,
            wallet_address,
            owner_address,
        })
    }
}

#[cfg(test)]
#[path = "../../../tests/contracts/tep/jetton/test_jetton_take_wallet_address_msg.rs"]
mod tests;
