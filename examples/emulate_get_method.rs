#[cfg(feature = "tonlibjson")]
mod example {
    use std::str::FromStr;
    use ton::contracts::tep::jetton::jetton_master_contract::GetJettonDataResult;
    use ton::contracts::{ContractClient, TonContract};
    use ton::emulators::{TLEmulatorProvider, emulator_pool::EmulatorPool};
    use ton::errors::TonResult;
    use ton::net_config::TonNetConfig;
    use ton::tl_client::{TLClient, TLStateProvider};
    use ton::ton_contract;
    use ton_core::types::TonAddress;
    use ton_core::{TLB, ton_methods};

    ton_contract!(StonfiPool<StonFiPoolData>);
    // macros expands to:
    // pub struct StonfiPool {
    //     client: ::ton::contracts::ContractClient,
    //     state: ::ton::contracts::LazyTonContractState,
    // }
    // impl ::ton::contracts::TonContract for StonfiPool {
    //     type ContractDataT = StonFiPoolData;
    //     fn new(client: &::ton::contracts::ContractClient, address: &TonAddress, tx_id: Option<TxLTHash>) -> Self {
    //         Self { client: client.clone(), state: ::ton::contracts::LazyTonContractState::new(*address, tx_id) }
    //     }
    //     fn from_state(client: ::ton::contracts::ContractClient, state: std::sync::Arc<::ton::ton_core::traits::state_provider::ContractState>) -> Self {
    //         Self { client, state: ::ton::contracts::LazyTonContractState::from_state(state) }
    //     }
    //     fn load_state(&self) -> impl std::future::Future<Output = ::ton::errors::TonResult<&std::sync::Arc<::ton::ton_core::traits::state_provider::ContractState>>> + Send {
    //         self.state.get_or_load(&self.client)
    //     }
    //     fn get_emulator_contract_state(&self) -> ::ton::ton_core::traits::emulator_provider::EmulatorContractState {
    //         self.state.get_emulator_contract_state()
    //     }
    //     fn get_client(&self) -> &::ton::contracts::ContractClient { &self.client }
    // }

    #[derive(Debug, Clone, TLB)]
    pub struct StonFiPoolData {
        address: TonAddress,
    }

    #[ton_methods]
    impl StonfiPool {
        async fn get_jetton_data(&self) -> TonResult<GetJettonDataResult>;
    }

    pub async fn real_main() -> anyhow::Result<()> {
        let tl_client = TLClient::builder()?.with_net_config(&TonNetConfig::new_default(false)?)?.build().await?;

        let state_provider = TLStateProvider::new(tl_client.clone());
        let emulator_provider = TLEmulatorProvider::new(tl_client, EmulatorPool::builder()?.build()?);
        let ctr_cli = ContractClient::builder(state_provider, emulator_provider)?.build()?;

        let address = TonAddress::from_str("EQBSUY4UWGJFAps0KwHY4tpOGqzU41DZhyrT8OuyAWWtnezy")?;

        // Emulation using predefined implementation of TonContract
        let pool = StonfiPool::new(&ctr_cli, &address, None);
        let jetton_data = pool.get_jetton_data().await?;
        let pool_data = pool.load_parsed_data().await?;
        println!("[predefined] jetton_data result: {:?}", jetton_data);
        println!("[predefined] pool_data result: {:?}", pool_data);
        Ok(())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    #[cfg(feature = "tonlibjson")]
    example::real_main().await?;
    Ok(())
}
