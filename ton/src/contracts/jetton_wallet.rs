use crate::contracts::methods::get_wallet_data::GetWalletData;
use crate::contracts::ton_contract::ContractCtx;
use ton_lib_core::ton_contract;

#[ton_contract]
pub struct JettonWallet;
impl GetWalletData for JettonWallet {}
