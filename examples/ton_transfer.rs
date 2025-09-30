use anyhow::Ok;

#[cfg(feature = "tonlibjson")]
mod example {
    use log::LevelFilter;
    use log4rs::append::console::{ConsoleAppender, Target};
    use log4rs::config::{Appender, Root};
    use log4rs::Config;
    use std::sync::Once;
    use std::time::Duration;
    use ton_lib::block_tlb::{Coins, CommonMsgInfoInt, Msg};
    use ton_lib::block_tlb::{CommonMsgInfo, CurrencyCollection};
    use ton_lib::clients::tl_client::tl::client::TLClientTrait;
    use ton_lib::clients::tl_client::TLClient;
    use ton_lib::clients::tl_client::TLClientConfig;
    use ton_lib::contracts::contract_client::{ContractClient, ContractClientConfig};
    use ton_lib::contracts::tl_provider::TLProvider;
    use ton_lib::contracts::{TonContract, TonWalletContract, TonWalletMethods};
    use ton_lib::sys_utils::sys_tonlib_set_verbosity_level;
    use ton_lib::wallet::WalletVersion;
    use ton_lib::wallet::{Mnemonic, TonWallet};
    use ton_lib_core::cell::TonCell;
    use ton_lib_core::traits::tlb::TLB;
    use ton_lib_core::types::tlb_core::{MsgAddress, TLBEitherRef};

    // Transaction: https://testnet.tonviewer.com/transaction/3771a86dd5c5238ac93e7f125817379c7a9d1321c79b27ac5e6b2b2d34749af1
    // How external and internal messages work: https://docs.ton.org/v3/guidelines/smart-contracts/howto/wallet#-external-and-internal-messages
    /* Plan:
        - Ton transfer (We will use wallet v4)
            - make an internal message with empty sell. It will signal that it is transfer
            - make an correct external message, and put there an internal message
            - send message to ton blockchain
    */
    static LOG: Once = Once::new();

    fn init_logging() {
        LOG.call_once(|| {
            let stderr = ConsoleAppender::builder()
                .target(Target::Stderr)
                .encoder(Box::new(log4rs::encode::pattern::PatternEncoder::new(
                    "{d(%Y-%m-%d %H:%M:%S%.6f)} {T:>15.15} {h({l:>5.5})} {t}:{L} - {m}{n}",
                )))
                .build();

            let config = Config::builder()
                .appender(Appender::builder().build("stderr", Box::new(stderr)))
                .build(Root::builder().appender("stderr").build(LevelFilter::Info))
                .unwrap();

            log4rs::init_config(config).unwrap();
        })
    }

    async fn make_tl_client(mainnet: bool, archive_only: bool) -> anyhow::Result<TLClient> {
        init_logging();
        log::info!("Initializing tl_client with mainnet={mainnet}, archive_only={archive_only}...");
        let mut config = match mainnet {
            true => TLClientConfig::new_mainnet(archive_only),
            false => TLClientConfig::new_testnet(archive_only),
        };
        config.connections_count = 10;
        config.retry_strategy.retry_count = 10;
        let client = TLClient::new(config).await?;
        sys_tonlib_set_verbosity_level(0);
        Ok(client)
    }

    pub async fn real_main() -> anyhow::Result<()> {
        // ---------- Wallet initialization ----------
        let mnemonic_str = std::env::var("MNEMONIC_STR")?;
        let key_pair = Mnemonic::from_str(&mnemonic_str, None)?.to_key_pair()?;
        // To create w5 wallet for testnet, use TonWallet::new_with_params with WALLET_V5R1_DEFAULT_ID_TESTNET wallet_id
        let wallet = TonWallet::new(WalletVersion::V4R2, key_pair)?;

        // Make testnet client
        let tl_client = make_tl_client(false, false).await?;
        let provider = TLProvider::new(tl_client.clone());
        let ctr_config = ContractClientConfig::new_no_cache(Duration::from_millis(100));
        let ctr_cli = ContractClient::new(ctr_config, provider)?;

        // ---------- Building transfer_msg ----------
        let transfer_msg = Msg {
            info: CommonMsgInfo::Int(CommonMsgInfoInt {
                ihr_disabled: false,
                bounce: false,
                bounced: false,
                src: MsgAddress::NONE,
                dst: MsgAddress::Int(wallet.address.to_msg_address_int()),
                value: CurrencyCollection::new(50010u128),
                ihr_fee: Coins::ZERO,
                fwd_fee: Coins::ZERO,
                created_lt: 0,
                created_at: 0,
            }),
            init: None,
            body: TLBEitherRef::new(TonCell::EMPTY),
        };

        let expired_at_time = std::time::SystemTime::now() + Duration::from_secs(600);
        let expire_at = expired_at_time.duration_since(std::time::UNIX_EPOCH)?.as_secs() as u32;

        // Get current wallet seqno
        let wallet_ctr = TonWalletContract::new(&ctr_cli, wallet.address.clone(), None).await?;
        let seqno = wallet_ctr.seqno().await?;

        let ext_in_msg = wallet.create_ext_in_msg(vec![transfer_msg.to_cell_ref()?], seqno, expire_at, false)?;
        // Transaction: https://testnet.tonviewer.com/transaction/3771a86dd5c5238ac93e7f125817379c7a9d1321c79b27ac5e6b2b2d34749af1
        let _msg_hash = tl_client.send_msg(ext_in_msg.to_boc()?).await?;

        Ok(())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    #[cfg(feature = "tonlibjson")]
    example::real_main().await?;

    Ok(())
}
