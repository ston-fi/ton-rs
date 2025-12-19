use ton_core::types::TonAddress;
use ton_macros::TVMType;

#[derive(Debug, Clone, PartialEq, Eq, TVMType)]
#[tvm_type(ensure_empty = true)]
pub struct GetWalletAddressResult {
    pub address: TonAddress,
}
