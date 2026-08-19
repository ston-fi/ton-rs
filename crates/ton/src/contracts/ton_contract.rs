use crate::block_tlb::{FromTVMStack, TVMStack};
use crate::contracts::TVMGetMethodID;
use crate::contracts::contract_client::ContractClient;
use crate::errors::{TonError, TonResult};
use std::future::Future;
use std::sync::Arc;
use tokio::sync::OnceCell;
use ton_core::traits::emulator_provider::{EmulatorContractState, EmulatorGetMethodRequest};
use ton_core::traits::state_provider::ContractState;
use ton_core::traits::tlb::TLB;
use ton_core::types::{TonAddress, TxLTHash};

pub trait TonContract: Send + Sync + Sized {
    // derive implementation automatically using ton_contract! macro (see below)
    type ContractDataT: TLB;
    /// Creates a contract wrapper without loading its state.
    fn new(client: &ContractClient, address: &TonAddress, tx_id: Option<TxLTHash>) -> Self;
    /// Creates a contract wrapper with an already loaded state.
    fn from_state(client: ContractClient, state: Arc<ContractState>) -> Self;
    /// Returns the contract state, loading it on the first call when necessary.
    fn load_state(&self) -> impl Future<Output = TonResult<&Arc<ContractState>>> + Send;
    /// Returns custom state when loaded, or the address and transaction for provider-side resolution.
    fn get_emulator_contract_state(&self) -> EmulatorContractState;
    fn get_client(&self) -> &ContractClient;

    fn emulate_get_method<'a, M, T: FromTVMStack>(
        &'a self,
        method: M,
        stack: &'a TVMStack,
    ) -> impl Future<Output = TonResult<T>> + Send + 'a
    where
        M: Into<TVMGetMethodID> + Send + 'a,
    {
        async move {
            let method_id = method.into().to_id();
            let stack_boc = Arc::new(stack.to_boc()?);
            let request = match self.get_emulator_contract_state() {
                EmulatorContractState::Address { address, tx_id } => {
                    EmulatorGetMethodRequest::new_with_address(address, tx_id, method_id, stack_boc)
                }
                EmulatorContractState::Custom(state) => {
                    EmulatorGetMethodRequest::new_with_state(state, method_id, stack_boc)
                }
                _ => {
                    return Err(TonError::EmulatorUnexpectedResponse(
                        "unsupported EmulatorContractState variant".to_string(),
                    ));
                }
            };
            let response = self.get_client().emulate_get_method(request).await?;
            T::from_stack_boc(response.stack_boc)
        }
    }

    fn load_parsed_data(&self) -> impl Future<Output = TonResult<Self::ContractDataT>> + Send {
        async move {
            let state = self.load_state().await?;
            match state.data_boc.as_ref() {
                Some(data_boc) => Ok(TLB::from_boc(data_boc.to_owned())?),
                None => Err(TonError::TonContractNotFull {
                    address: state.address,
                    tx_id: Some(state.last_tx_id.clone()),
                    missing_field: "data".to_string(),
                }),
            }
        }
    }
}

/// Lazy contract identity and cached state used by [`ton_contract!`].
#[doc(hidden)]
pub struct LazyTonContractState {
    address: TonAddress,
    tx_id: Option<TxLTHash>,
    state: OnceCell<Arc<ContractState>>,
}

#[doc(hidden)]
impl LazyTonContractState {
    pub fn new(address: TonAddress, tx_id: Option<TxLTHash>) -> Self {
        Self {
            address,
            tx_id,
            state: OnceCell::new(),
        }
    }

    pub fn from_state(state: Arc<ContractState>) -> Self {
        Self {
            address: state.address,
            tx_id: Some(state.last_tx_id.clone()),
            state: OnceCell::new_with(Some(state)),
        }
    }

    pub async fn get_or_load(&self, client: &ContractClient) -> TonResult<&Arc<ContractState>> {
        self.state.get_or_try_init(|| client.load_state(&self.address, self.tx_id.as_ref())).await
    }

    pub fn get_emulator_contract_state(&self) -> EmulatorContractState {
        match self.state.get() {
            Some(state) => EmulatorContractState::Custom(state.as_ref().clone()),
            None => EmulatorContractState::Address {
                address: self.address,
                tx_id: self.tx_id.clone(),
            },
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
            state: $crate::contracts::LazyTonContractState,
        }

        impl $crate::contracts::TonContract for $name {
            type ContractDataT = $DATATYPE;
            fn new(
                client: &$crate::contracts::ContractClient,
                address: &$crate::ton_core::types::TonAddress,
                tx_id: Option<$crate::ton_core::types::TxLTHash>,
            ) -> Self {
                Self { client: client.clone(), state: $crate::contracts::LazyTonContractState::new(*address, tx_id) }
            }
            fn from_state(client: $crate::contracts::ContractClient, state: std::sync::Arc<$crate::ton_core::traits::state_provider::ContractState>) -> Self {
                Self { client, state: $crate::contracts::LazyTonContractState::from_state(state) }
            }
            fn load_state(&self) -> impl std::future::Future<Output = $crate::errors::TonResult<&std::sync::Arc<$crate::ton_core::traits::state_provider::ContractState>>> + Send {
                self.state.get_or_load(&self.client)
            }
            fn get_emulator_contract_state(&self) -> $crate::ton_core::traits::emulator_provider::EmulatorContractState {
                self.state.get_emulator_contract_state()
            }
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
