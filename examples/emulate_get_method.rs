#[cfg(feature = "tonlibjson")]
mod example {
    use std::str::FromStr;
    use ton_lib::block_tlb::TVMStack;
    use ton_lib::contracts::tl_provider::TLProvider;
    use ton_lib::contracts::ContractCtx;
    use ton_lib::contracts::{ContractClient, ContractClientConfig, TonContract};
    use ton_lib::emulators::tvm_emulator::TVMGetMethodID;
    use ton_lib::errors::TonError;
    use ton_lib::tl_client::{TLClient, TLClientConfig};
    use ton_lib_core::ton_contract;
    use ton_lib_core::traits::tlb::TLB;
    use ton_lib_core::types::TonAddress;

    #[ton_contract]
    struct StonfiPool;
    impl StonfiPool {
        async fn get_jetton_data(&self) -> Result<TVMStack, TonError> {
            let boc = self.emulate_get_method("get_jetton_data", &TVMStack::EMPTY).await?;
            Ok(TVMStack::from_boc(&boc)?)
        }

        async fn get_pool_data(&self) -> Result<TVMStack, TonError> {
            let boc = self.emulate_get_method("get_pool_data", &TVMStack::EMPTY).await?;
            Ok(TVMStack::from_boc(&boc)?)
        }
    }

    pub async fn real_main() -> anyhow::Result<()> {
        let tl_config = TLClientConfig::new_mainnet(false);
        let tl_client = TLClient::new(tl_config).await?;
        let provider = TLProvider::new(tl_client.clone());
        let ctr_cfg = ContractClientConfig::new_no_cache(Default::default());
        let ctr_cli = ContractClient::new(ctr_cfg, provider)?;

        let address = TonAddress::from_str("EQBSUY4UWGJFAps0KwHY4tpOGqzU41DZhyrT8OuyAWWtnezy")?;

        // Emulation using predefined implementation of TonContract
        let pool = StonfiPool::new(&ctr_cli, address.clone(), None).await?;
        let jetton_data = pool.get_jetton_data().await?;
        let pool_data = pool.get_pool_data().await?;
        println!("[predefined] jetton_data_result stack len: {:?}", jetton_data.len());
        println!("[predefined] pool_data_result stack len: {:?}", pool_data.len());

        // Emulation using contract client directly
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
