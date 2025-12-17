use crate::block_tlb::TVMStack;
use crate::contracts::contract_client::ContractClient;
use crate::emulators::tvm_emulator::TVMGetMethodID;
use crate::errors::TonError;
use crate::tep::tvm_results::TVMResult;
use std::sync::Arc;
use ton_core::traits::contract_provider::TonContractState;
use ton_core::traits::tlb::TLB;
use ton_core::types::{TonAddress, TxLTHash};

#[async_trait::async_trait]
pub trait TonContract: Send + Sync + Sized {
    // derive implementation automatically using ton_contract! macro
    fn from_state(client: ContractClient, state: Arc<TonContractState>) -> Self;
    fn get_state(&self) -> &Arc<TonContractState>;
    fn get_client(&self) -> &ContractClient;
    type ContractData: TLB;

    async fn new(client: &ContractClient, address: &TonAddress, tx_id: Option<TxLTHash>) -> Result<Self, TonError> {
        let state = client.get_contract(address, tx_id.as_ref()).await?;
        Ok(Self::from_state(client.clone(), state))
    }

    async fn emulate_get_method<M, T: TVMResult>(
        &self,
        method: M,
        stack: &TVMStack,
        mc_seqno: Option<i32>,
    ) -> Result<T, TonError>
    where
        M: Into<TVMGetMethodID> + Send,
    {
        let method_id = method.into().to_id();
        let response =
            self.get_client().emulate_get_method(self.get_state(), method_id, stack.to_boc()?, mc_seqno).await?;
        T::from_stack_boc(response.stack_boc()?)
    }

    async fn get_parsed_data(&self) -> Result<Self::ContractData, TonError> {
        let state = self.get_state();
        match state.data_boc.as_ref() {
            Some(data_boc) => Ok(TLB::from_boc(data_boc.to_owned())?),
            None => Err(TonError::TonContractNotFull {
                address: state.address.clone(),
                tx_id: Some(state.last_tx_id.clone()),
                missing_field: "data".to_string(),
            }),
        }
    }
}

#[macro_export]
macro_rules! ton_contract {
    ($name:ident) => {
        pub struct $name<T: TLB> {
            client: ContractClient,
            state: std::sync::Arc<TonContractState>,
            _phantom: std::marker::PhantomData<T>,
        }

        impl TonContract for $name<TonCell> {
            type ContractData = TonCell;
            fn from_state(client: ContractClient, state: std::sync::Arc<TonContractState>) -> Self {
                Self { client, state, _phantom: std::marker::PhantomData }
            }
            fn get_client(&self) -> &ContractClient { &self.client }
            fn get_state(&self) -> &std::sync::Arc<TonContractState> { &self.state }
        }
    };
    ($name:ident $( : $($traits:tt)+ )? ) => {
        pub struct $name<T: TLB> {
            client: ContractClient,
            state: std::sync::Arc<TonContractState>,
            _phantom: std::marker::PhantomData<T>,
        }

        impl TonContract for $name<TonCell> {
            type ContractData = TonCell;
            fn from_state(client: ContractClient, state: std::sync::Arc<TonContractState>) -> Self {
                Self { client, state, _phantom: std::marker::PhantomData }
            }
            fn get_client(&self) -> &ContractClient { &self.client }
            fn get_state(&self) -> &std::sync::Arc<TonContractState> { &self.state }
        }

        $(
            $crate::__impl_traits_for_contract!($name<TonCell> : $($traits)+);
        )?
    };
    ($name:ident < $DATATYPE:ty > $( : $($traits:tt)+ )? ) => {
        pub struct $name<T: TLB> {
            client: ContractClient,
            state: std::sync::Arc<TonContractState>,
            _phantom: std::marker::PhantomData<T>,
        }

        impl TonContract for $name<$DATATYPE> {
            type ContractData = $DATATYPE;
            fn from_state(client: ContractClient, state: std::sync::Arc<TonContractState>) -> Self {
                Self { client, state, _phantom: std::marker::PhantomData }
            }
            fn get_client(&self) -> &ContractClient { &self.client }
            fn get_state(&self) -> &std::sync::Arc<TonContractState> { &self.state }
        }

        $(
            $crate::__impl_traits_for_contract!($name<$DATATYPE> : $($traits)+);
        )?
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __impl_traits_for_contract {
    // Single trait for a named type with its datatype
    ($name:ident<$DATATYPE:ty> : $trait:path) => {
        impl $trait for $name<$DATATYPE> {}
    };

    // Multiple traits separated by commas — recurse while preserving <$DATATYPE>
    ($name:ident<$DATATYPE:ty> : $trait:path , $($rest:tt)+) => {
        impl $trait for $name<$DATATYPE> {}
        $crate::__impl_traits_for_contract!($name<$DATATYPE> : $($rest)+);
    };
}

#[cfg(test)]
mod tests {
    use crate::contracts::{ContractClient, TonContract};
    use ton_core::cell::TonCell;
    use ton_core::traits::contract_provider::TonContractState;
    use ton_core::traits::tlb::TLB;
    use ton_macros::TLB;

    #[test]
    #[allow(unused)]
    fn test_ton_contract_macro() {
        ton_contract!(MyContract1);
        trait MyTrait1 {}
        ton_contract!(MyContract2: MyTrait1);
        trait MyTrait2 {}
        ton_contract!(MyContract3: MyTrait1, MyTrait2);

        #[derive(TLB)]
        struct MyContract4Data;
        ton_contract!(MyContract4<MyContract4Data>);

        #[derive(TLB)]
        struct MyContract5Data;
        ton_contract!(MyContract5<MyContract5Data>: MyTrait1, MyTrait2);
    }
}
