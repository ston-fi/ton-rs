use crate::block_tlb::TVMStack;
use crate::contracts::client::ContractClient;
use crate::emulators::tvm_emulator::TVMGetMethodID;
use crate::errors::TonError;
use std::sync::Arc;
use ton_lib_core::traits::contract_provider::TonContractState;
use ton_lib_core::traits::tlb::TLB;
use ton_lib_core::types::{TonAddress, TxLTHash};

pub struct ContractCtx {
    pub client: ContractClient,
    pub address: TonAddress,
    pub state: Arc<TonContractState>,
}

#[async_trait::async_trait]
pub trait TonContract: Send + Sync + Sized {
    fn ctx(&self) -> &ContractCtx;
    fn from_ctx(ctx: ContractCtx) -> Self;

    async fn new(client: &ContractClient, address: TonAddress, tx_id: Option<TxLTHash>) -> Result<Self, TonError> {
        let state = client.get_contract(&address, tx_id.as_ref()).await?;
        Self::from_state(client.clone(), address, state)
    }

    fn from_state(client: ContractClient, address: TonAddress, state: Arc<TonContractState>) -> Result<Self, TonError> {
        Ok(Self::from_ctx(ContractCtx { client, address, state }))
    }

    async fn get_state(&self) -> Result<&Arc<TonContractState>, TonError> { Ok(&self.ctx().state) }

    async fn emulate_get_method<M>(&self, method: M, stack: &TVMStack) -> Result<Vec<u8>, TonError>
    where
        M: Into<TVMGetMethodID> + Send,
    {
        let ctx = self.ctx();
        let method_id = method.into().to_id();
        let stack_boc = stack.to_boc()?;
        let response = ctx.client.emulate_get_method(&ctx.state, method_id, &stack_boc).await?;
        response.stack_boc()
    }

    async fn get_parsed_data<D: TLB>(&self) -> Result<D, TonError> {
        let state = self.get_state().await?;
        match &state.data_boc {
            Some(data_boc) => Ok(D::from_boc(data_boc)?),
            None => Err(TonError::TonContractNoData {
                address: state.address.clone(),
                tx_id: Some(state.last_tx_id.clone()),
            }),
        }
    }
}
