use std::marker::PhantomData;
use ton_core::cell::TonCell;
use ton_core::traits::tlb::TLB;
use crate::contracts::ContractClient;
use crate::contracts::TonContract;
use crate::contracts::contract_methods::TonWalletMethods;
use crate::ton_contract;
use crate::ton_core::traits::contract_provider::TonContractState;

// ton_contract!(TonWalletContract: TonWalletMethods);
pub struct TonWalletContract<T: TLB> {
    client: ContractClient,
    state: std::sync::Arc<TonContractState>,
    _phantom: PhantomData<T>,
}
impl<T:TLB> TonContract for TonWalletContract<T> {
    type DataType = TonCell;
    fn from_state(client: ContractClient, state: std::sync::Arc<TonContractState>) -> Self { Self { client, state, _phantom: Default::default() } }
    fn get_client(&self) -> &ContractClient { &self.client }
    fn get_state(&self) -> &std::sync::Arc<TonContractState> { &self.state }
}
::ton::__impl_traits_for_contract!( TonWalletContract < TonCell > : TonWalletMethods );