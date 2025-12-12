use crate::block_tlb::TVMStack;
use crate::contracts::contract_client::ContractClient;
use crate::emulators::tvm_emulator::TVMGetMethodID;
use crate::errors::TonError;
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

    async fn new(client: &ContractClient, address: &TonAddress, tx_id: Option<TxLTHash>) -> Result<Self, TonError> {
        let state = client.get_contract(address, tx_id.as_ref()).await?;
        Ok(Self::from_state(client.clone(), state))
    }

    async fn emulate_get_method<M, T: TLB>(
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
        Ok(T::from_boc(response.stack_boc()?)?)
    }

    async fn get_parsed_data<D: TLB>(&self) -> Result<D, TonError> {
        let state = self.get_state();
        match state.data_boc.as_ref() {
            Some(data_boc) => Ok(D::from_boc(data_boc.to_owned())?),
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
    // Simple: no generics
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

    // Simple with trait impls
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

        // Expand each trait via helper (accepts a type)
        $crate::__impl_traits_for_contract!($name : $($traits)+);
    };

    // --- Concrete type: Name<SomeType> where SomeType is a concrete type path
    ($name:ident < $data_ty:ty >) => {
        pub struct $name {
            client: ContractClient,
            state: std::sync::Arc<TonContractState>,
            data: Option<$data_ty>,
        }

        impl TonContract for $name {
            fn from_state(client: ContractClient, state: std::sync::Arc<TonContractState>) -> Self {
                let data = state.data_boc.as_ref().and_then(|boc| <$data_ty as ::ton_core::traits::tlb::TLB>::from_boc(boc.to_owned()).ok());
                Self{client, state, data}
            }
            fn get_client(&self) -> &ContractClient { &self.client }
            fn get_state(&self) -> &std::sync::Arc<TonContractState> { &self.state }
        }

        #[allow(dead_code)]
        impl $name {
            pub async fn get_parsed_data(&self) -> Result<$data_ty, $crate::errors::TonError> {
                <Self as TonContract>::get_parsed_data::<$data_ty>(self).await
            }
        }
    };

    // Concrete type with trait impls
    ($name:ident < $data_type:ty > : $($traits:tt)+) => {
        pub struct $name {
            client: ContractClient,
            state: std::sync::Arc<TonContractState>,
            data: Option<$data_type>,
        }

        impl TonContract for $name {
            fn from_state(client: ContractClient, state: std::sync::Arc<TonContractState>) -> Self {
                let data = state.data_boc.as_ref().and_then(|boc| <$data_type as ::ton_core::traits::tlb::TLB>::from_boc(boc.to_owned()).ok());
                Self{client, state, data}
            }
            fn get_client(&self) -> &ContractClient { &self.client }
            fn get_state(&self) -> &std::sync::Arc<TonContractState> { &self.state }
        }

        #[allow(dead_code)]
        impl $name {
            pub async fn get_parsed_data(&self) -> Result<$data_type, $crate::errors::TonError> {
                <Self as TonContract>::get_parsed_data::<$data_type>(self).await
            }
        }

        $crate::__impl_traits_for_contract!($name : $($traits)+);
    };

    // --- Generic identifier: Name<T> where T is a type parameter that should implement TLB
    ($name:ident < $type_ident:ident >) => {
        pub struct $name<$type_ident: ::ton_core::traits::tlb::TLB> {
            client: ContractClient,
            state: std::sync::Arc<TonContractState>,
            data: Option<$type_ident>,
        }

        impl<$type_ident: ::ton_core::traits::tlb::TLB + Send + Sync> TonContract for $name<$type_ident> {
            fn from_state(client: ContractClient, state: std::sync::Arc<TonContractState>) -> Self {
                let data = state.data_boc.as_ref().and_then(|boc| <$type_ident as ::ton_core::traits::tlb::TLB>::from_boc(boc.to_owned()).ok());
                Self{client, state, data}
            }
            fn get_client(&self) -> &ContractClient { &self.client }
            fn get_state(&self) -> &std::sync::Arc<TonContractState> { &self.state }
        }

        impl<$type_ident: ::ton_core::traits::tlb::TLB + Send + Sync> $name<$type_ident> {
            pub async fn get_parsed_data(&self) -> Result<$type_ident, $crate::errors::TonError> {
                <Self as TonContract>::get_parsed_data::<$type_ident>(self).await
            }
        }
    };

    // Generic identifier with trait impls
    ($name:ident < $type_ident:ident > : $($traits:tt)+) => {
        #[allow(dead_code)]
        pub struct $name<$type_ident: ::ton_core::traits::tlb::TLB> {
            client: ContractClient,
            state: std::sync::Arc<TonContractState>,
            data: Option<$type_ident>,
        }

        impl<$type_ident: ::ton_core::traits::tlb::TLB + Send + Sync> TonContract for $name<$type_ident> {
            fn from_state(client: ContractClient, state: std::sync::Arc<TonContractState>) -> Self {
                let data = state.data_boc.as_ref().and_then(|boc| <$type_ident as ::ton_core::traits::tlb::TLB>::from_boc(boc.to_owned()).ok());
                Self{client, state, data}
            }
            fn get_client(&self) -> &ContractClient { &self.client }
            fn get_state(&self) -> &std::sync::Arc<TonContractState> { &self.state }
        }

        #[allow(dead_code)]
        impl<$type_ident: ::ton_core::traits::tlb::TLB + Send + Sync> $name<$type_ident> {
            pub async fn get_parsed_data(&self) -> Result<$type_ident, $crate::errors::TonError> {
                <Self as TonContract>::get_parsed_data::<$type_ident>(self).await
            }
        }

        // Implement requested traits for the parametrized type
        $crate::__impl_traits_for_contract!($name<$type_ident> : $($traits)+);
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __impl_traits_for_contract {
    ($name_ty:ty : $trait:path) => {
        impl $trait for $name_ty {}
    };

    ($name_ty:ty : $trait:path , $($rest:tt)+) => {
        impl $trait for $name_ty {}
        $crate::__impl_traits_for_contract!($name_ty : $($rest)+);
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use ton_core::TLB;

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
        ton_contract!(MyContract5<MyContract5Data>: MyTrait1);
        #[derive(TLB)]
        struct MyContract6Data;
        ton_contract!(MyContract6<MyContract6Data>: MyTrait1, MyTrait2);
    }
}
