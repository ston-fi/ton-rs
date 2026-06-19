use ton_core::types::TonAddress;
use ton_macros::FromTVMStack;

#[derive(Debug, Clone, PartialEq, Eq, FromTVMStack)]
pub struct GetWalletAddressResult {
    pub address: TonAddress,
}
