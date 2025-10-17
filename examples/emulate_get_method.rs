#[cfg(feature = "tonlibjson")]
mod example {
    use std::str::FromStr;
    use ton_lib::block_tlb::TVMStack;
    use ton_lib::contracts::tl_provider::TLProvider;
    use ton_lib::contracts::{ContractClient, TonContract};
    use ton_lib::emulators::tvm_emulator::TVMGetMethodID;
    use ton_lib::errors::TonError;
    use ton_lib::net_config::TonNetConfig;
    use ton_lib::tl_client::TLClient;
    use ton_lib::ton_contract;
    use ton_lib_core::traits::contract_provider::TonContractState;
    use ton_lib_core::traits::tlb::TLB;
    use ton_lib_core::types::TonAddress;

    ton_contract!(StonfiPool);

    impl StonfiPool {
        async fn get_jetton_data(&self) -> Result<TVMStack, TonError> {
            let boc = self.emulate_get_method("get_jetton_data", &TVMStack::EMPTY).await?;
            Ok(TVMStack::from_boc(boc)?)
        }

        async fn get_pool_data(&self) -> Result<TVMStack, TonError> {
            let boc = self.emulate_get_method("get_pool_data", &TVMStack::EMPTY).await?;
            Ok(TVMStack::from_boc(boc)?)
        }
    }

    pub async fn real_main() -> anyhow::Result<()> {
        let tl_client = TLClient::builder()?.with_net_config(&TonNetConfig::new_default(false)?)?.build().await?;

        let provider = TLProvider::new(tl_client);
        let ctr_cli = ContractClient::builder(provider).build()?;

        let address = TonAddress::from_str("EQBSUY4UWGJFAps0KwHY4tpOGqzU41DZhyrT8OuyAWWtnezy")?;

        // Emulation using predefined implementation of TonContract
        let pool = StonfiPool::new(&ctr_cli, &address, None).await?;
        let jetton_data = pool.get_jetton_data().await?;
        let pool_data = pool.get_pool_data().await?;
        println!("[predefined] jetton_data_result stack len: {:?}", jetton_data.len());
        println!("[predefined] pool_data_result stack len: {:?}", pool_data.len());

        // Emulation using contract contract_client directly
        let state = ctr_cli.get_contract(&address, None).await?;
        let method_id = TVMGetMethodID::from("get_jetton_data").to_id();
        let emul_result = ctr_cli.emulate_get_method(&state, method_id, TVMStack::EMPTY_BOC).await?;
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
