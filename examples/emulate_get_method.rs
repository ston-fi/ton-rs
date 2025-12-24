#[cfg(feature = "tonlibjson")]
mod example {
    use std::str::FromStr;
    use ton::block_tlb::TVMStack;
    use ton::contracts::tl_provider::TLProvider;
    use ton::contracts::{ContractClient, TonContract};
    use ton::emulators::tvm_emulator::TVMGetMethodID;
    use ton::errors::TonResult;
    use ton::net_config::TonNetConfig;
    use ton::tep::tvm_result::GetJettonDataResult;
    use ton::tl_client::TLClient;
    use ton::ton_contract;
    use ton_core::types::TonAddress;
    use ton_core::{TLB, ton_methods};

    ton_contract!(StonfiPool<StonFiPoolData>);
    // macros expands to:
    // pub struct StonfiPool {
    //     client: ::ton::contracts::ContractClient,
    //     state: std::sync::Arc<::ton::ton_core::traits::contract_provider::TonContractState>,
    // }
    // impl ::ton::contracts::TonContract for StonfiPool {
    //     type ContractDataT = StonFiPoolData;
    //     fn from_state(client: ::ton::contracts::ContractClient, state: std::sync::Arc<::ton::ton_core::traits::contract_provider::TonContractState>) -> Self {
    //         Self { client, state }
    //     }
    //     fn get_state(&self) -> &std::sync::Arc<::ton::ton_core::traits::contract_provider::TonContractState> { &self.state }
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

        let provider = TLProvider::new(tl_client);
        let ctr_cli = ContractClient::builder(provider)?.build()?;

        let address = TonAddress::from_str("EQBSUY4UWGJFAps0KwHY4tpOGqzU41DZhyrT8OuyAWWtnezy")?;

        // Emulation using predefined implementation of TonContract
        let pool = StonfiPool::new(&ctr_cli, &address, None).await?;
        let jetton_data = pool.get_jetton_data().await?;
        let pool_data = pool.get_parsed_data().await?;
        println!("[predefined] jetton_data result: {:?}", jetton_data);
        println!("[predefined] pool_data result: {:?}", pool_data);

        // Emulation using contract contract_client directly
        let state = ctr_cli.get_contract(&address, None).await?;
        let method_id = TVMGetMethodID::from("get_jetton_data").to_id();
        let emul_result = ctr_cli.emul_get_method(&state, method_id, TVMStack::EMPTY_BOC.to_owned(), None).await?;
        let jetton_data = emul_result.stack_parsed()?;
        println!("[arbitrary] jetton_data_result stack len: {:?}", jetton_data.len());
        Ok(())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    #[cfg(feature = "tonlibjson")]
    example::real_main().await?;
    Ok(())
}
