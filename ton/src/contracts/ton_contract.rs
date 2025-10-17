use crate::block_tlb::TVMStack;
use crate::contracts::contract_client::ContractClient;
use crate::emulators::tvm_emulator::TVMGetMethodID;
use crate::errors::TonError;
use std::sync::Arc;
use ton_lib_core::traits::contract_provider::TonContractState;
use ton_lib_core::traits::tlb::TLB;
use ton_lib_core::types::{TonAddress, TxLTHash};

#[async_trait::async_trait]
pub trait TonContract: Send + Sync + Sized {
    // derive implementation automatically using ton_contract! macro
    fn from_state(client: ContractClient, state: Arc<TonContractState>) -> Self;
    fn get_state(&self) -> &Arc<TonContractState>;
    fn get_client(&self) -> &ContractClient;

    async fn new(client: &ContractClient, address: &TonAddress, tx_id: Option<TxLTHash>) -> Result<Self, TonError> {
        let state = client.get_contract(address, tx_id.as_ref()).await?;
        Ok(Self::from_state(client.clone(), state))
    }

    async fn emulate_get_method<M>(&self, method: M, stack: &TVMStack) -> Result<Vec<u8>, TonError>
    where
        M: Into<TVMGetMethodID> + Send,
    {
        let method_id = method.into().to_id();
        let stack_boc = stack.to_boc()?;
        let response = self.get_client().emulate_get_method(self.get_state(), method_id, &stack_boc).await?;
        response.stack_boc()
    }

    async fn get_parsed_data<D: TLB>(&self) -> Result<D, TonError> {
        let state = self.get_state();
        match state.data_boc.as_ref() {
            Some(data_boc) => Ok(D::from_boc(data_boc.to_owned())?),
            None => Err(TonError::TonContractNoData {
                address: state.address.clone(),
                tx_id: Some(state.last_tx_id.clone()),
            }),
        }
    }
}

#[macro_export]
macro_rules! ton_contract {
    ($name:ident) => {
        pub struct $name {
            client: ContractClient,
            state: std::sync::Arc<TonContractState>,
        }

        impl TonContract for $name {
            fn from_state(client: ContractClient, state: std::sync::Arc<TonContractState>) -> Self { Self{client, state} }
            fn get_client(&self) -> &ContractClient { &self.client }
            fn get_state(&self) -> &std::sync::Arc<TonContractState> { &self.state }
        }
    };
    ($name:ident : $($traits:tt)+) => {
        pub struct $name {
            client: ContractClient,
            state: std::sync::Arc<TonContractState>,
        }

        impl TonContract for $name {
            fn from_state(client: ContractClient, state: std::sync::Arc<TonContractState>) -> Self { Self{client, state} }
            fn get_client(&self) -> &ContractClient { &self.client }
            fn get_state(&self) -> &std::sync::Arc<TonContractState> { &self.state }
        }

        // Expand each trait separated by '+'
        $crate::__impl_traits_for_contract!($name : $($traits)+);
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __impl_traits_for_contract {
    // Single trait
    ($name:ident : $trait:path) => {
        impl $trait for $name {}
    };

    // Multiple traits separated by '+'
    ($name:ident : $trait:path , $($rest:tt)+) => {
        impl $trait for $name {}
        $crate::__impl_traits_for_contract!($name : $($rest)+);
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[allow(unused)]
    fn test_ton_contract_macro() {
        ton_contract!(MyContract1);

        trait MyTrait1 {}
        ton_contract!(MyContract2: MyTrait1);

        trait MyTrait2 {}
        ton_contract!(MyContract3: MyTrait1, MyTrait2);
    }
}
