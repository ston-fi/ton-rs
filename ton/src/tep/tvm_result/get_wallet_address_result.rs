use ton_core::types::TonAddress;
use ton_macros::FromTVMStack;

#[derive(Debug, Clone, PartialEq, Eq, FromTVMStack)]
#[from_tvm_stack(ensure_empty = true)]
pub struct GetWalletAddressResult {
    pub address: TonAddress,
}
