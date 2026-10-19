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
mod tests {
    use super::*;
    use crate::contracts::tep::jetton::jetton_provide_wallet_address_msg::JettonProvideWalletAddressMsg;
    use ton_core::types::TonAddress;
    #[test]
    fn literal_addresses_and_owner_reference_round_trip() -> TonCoreResult<()> {
        let zero = MsgAddress::Int(TonAddress::ZERO.to_msg_address_int());
        assert_ne!(zero, MsgAddress::NONE);
        for address in [MsgAddress::NONE, zero] {
            for include in [false, true] {
                let request = JettonProvideWalletAddressMsg {
                    query_id: u64::MAX,
                    owner_address: address.clone(),
                    include_address: include,
                };
                let cell = request.to_cell()?;
                assert_eq!(JettonProvideWalletAddressMsg::from_cell(&cell)?.to_cell()?, cell);
                let reply = JettonTakeWalletAddressMsg {
                    query_id: 3,
                    wallet_address: address.clone(),
                    owner_address: include.then(|| TLBRef::new(address.clone())),
                };
                let cell = reply.to_cell()?;
                assert_eq!(JettonTakeWalletAddressMsg::from_cell_strict(&cell)?, reply);
            }
        }
        Ok(())
    }
    #[test]
    fn rejects_malformed_root_and_owner_tails() -> TonCoreResult<()> {
        let reply = JettonTakeWalletAddressMsg {
            query_id: 3,
            wallet_address: MsgAddress::NONE,
            owner_address: None,
        };
        let cell = reply.to_cell()?;
        let mut root = TonCell::builder();
        root.write_cell(&cell)?;
        root.write_bit(false)?;
        assert!(JettonTakeWalletAddressMsg::from_cell_strict(&root.build()?).is_err());
        let mut owner = TonCell::builder();
        MsgAddress::NONE.write(&mut owner)?;
        owner.write_bit(false)?;
        let mut root = TonCell::builder();
        JettonTakeWalletAddressMsg::write_prefix(&mut root)?;
        3u64.write(&mut root)?;
        MsgAddress::NONE.write(&mut root)?;
        Some(TLBRef::new(owner.build()?)).write(&mut root)?;
        assert!(JettonTakeWalletAddressMsg::from_cell_strict(&root.build()?).is_err());
        assert!(JettonTakeWalletAddressMsg::from_cell_strict(TonCell::empty()).is_err());
        let request = JettonProvideWalletAddressMsg {
            query_id: 3,
            owner_address: MsgAddress::NONE,
            include_address: false,
        };
        assert!(JettonTakeWalletAddressMsg::from_cell_strict(&request.to_cell()?).is_err());
        Ok(())
    }
}
