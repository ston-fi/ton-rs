//! TEP-89 asynchronous Jetton wallet-address request.
use ton_core::{TLB, types::tlb_core::MsgAddress};

/// `provide_wallet_address#2c76b973 query_id:uint64 owner_address:MsgAddress include_address:Bool`.
///
/// The raw address preserves literal none separately from standard internal zero.
/// This fixed wire record is exhaustive for direct construction. Parsing does not
/// authenticate the recipient, enforce its gas rule, or prove that a reply exists.
/// A recipient-specific prefix parser may allow data its classified contract ignores.
#[derive(Debug, Clone, PartialEq, TLB)]
#[tlb(prefix = 0x2c76b973, bits_len = 32)]
pub struct JettonProvideWalletAddressMsg {
    pub query_id: u64,
    pub owner_address: MsgAddress,
    pub include_address: bool,
}
