use crate::block_tlb::{TVMStack, TVMType};
use crate::contracts::contract_client::ContractClient;
use crate::emulators::tvm_emulator::TVMGetMethodID;
use crate::errors::{TonError, TonResult};
use std::sync::Arc;
use ton_core::traits::contract_provider::TonContractState;
use ton_core::traits::tlb::TLB;
use ton_core::types::{TonAddress, TxLTHash};

#[async_trait::async_trait]
pub trait TonContract: Send + Sync + Sized {
    // derive implementation automatically using ton_contract! macro (see below)
    type ContractDataT: TLB;
    fn from_state(client: ContractClient, state: Arc<TonContractState>) -> Self;
    fn get_state(&self) -> &Arc<TonContractState>;
    fn get_client(&self) -> &ContractClient;

    async fn new(client: &ContractClient, address: &TonAddress, tx_id: Option<TxLTHash>) -> TonResult<Self> {
        let state = client.get_contract(address, tx_id.as_ref()).await?;
        Ok(Self::from_state(client.clone(), state))
    }

    async fn emulate_get_method<M, T: TVMType>(
        &self,
        method: M,
        stack: &TVMStack,
        mc_seqno: Option<i32>,
    ) -> TonResult<T>
    where
        M: Into<TVMGetMethodID> + Send,
    {
        let method_id = method.into().to_id();
        let response =
            self.get_client().emulate_get_method(self.get_state(), method_id, stack.to_boc()?, mc_seqno).await?;
        T::from_stack_boc(response.stack_boc()?)
    }

    async fn get_parsed_data(&self) -> TonResult<Self::ContractDataT> {
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

/// Check usage examples in the tests module below
#[macro_export]
macro_rules! ton_contract {
    // no traits -> forward without `:`
    ($name:ident) => {
        $crate::ton_contract!($name<$crate::ton_core::cell::TonCell>);
    };
    // with traits -> forward the traits repetition (must match at least one)
    ($name:ident : $($traits:tt)+) => {
        $crate::ton_contract!($name<$crate::ton_core::cell::TonCell> : $($traits)+);
    };
    // primary implementation
    ($name:ident < $DATATYPE:ty > $( : $($traits:tt)+ )? ) => {
        pub struct $name {
            client: $crate::contracts::ContractClient,
            state: std::sync::Arc<$crate::ton_core::traits::contract_provider::TonContractState>,
        }

        impl $crate::contracts::TonContract for $name {
            type ContractDataT = $DATATYPE;
            fn from_state(client: $crate::contracts::ContractClient, state: std::sync::Arc<$crate::ton_core::traits::contract_provider::TonContractState>) -> Self {
                Self { client, state }
            }
            fn get_state(&self) -> &std::sync::Arc<$crate::ton_core::traits::contract_provider::TonContractState> { &self.state }
            fn get_client(&self) -> &$crate::contracts::ContractClient { &self.client }
        }

        $(
            $crate::__impl_traits_for_contract!($name<$DATATYPE> : $($traits)+);
        )?
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __impl_traits_for_contract {
    ($name:ident<$DATATYPE:ty>) => {
        // Base case: no traits to implement
    };
    // Single trait for a named type with its datatype
    ($name:ident<$DATATYPE:ty> : $trait:path) => {
        impl $trait for $name {}
    };

    // Multiple traits separated by commas — recurse while preserving <$DATATYPE>
    ($name:ident<$DATATYPE:ty> : $trait:path , $($rest:tt)+) => {
        impl $trait for $name {}
        $crate::__impl_traits_for_contract!($name<$DATATYPE> : $($rest)+);
    };
}

#[cfg(test)]
mod tests {
    use ton_macros::TLB;

    #[test]
    #[allow(unused)] // we just check it compiles
    fn test_ton_contract_macro() {
        ton_contract!(MyContract1);

        trait MyTrait1 {}
        ton_contract!(MyContract2: MyTrait1);

        trait MyTrait2 {}
        ton_contract!(MyContract3: MyTrait1, MyTrait2);

        #[derive(TLB)]
        pub struct MyContract4Data;
        ton_contract!(MyContract4<MyContract4Data>);

        #[derive(TLB)]
        pub struct MyContract5Data;
        ton_contract!(MyContract5<MyContract5Data>: MyTrait1, MyTrait2);
    }
}
