use log::LevelFilter;
use log4rs::append::console::{ConsoleAppender, Target};
use log4rs::config::{Appender, Root};
use log4rs::Config;
use std::sync::Once;
use std::time::Duration;
use ton_lib::lite_client::client::LiteClient;
use ton_lib::lite_client::config::LiteClientConfig;
use ton_lib::net_config::TonNetConfig;

static LOG: Once = Once::new();

pub(crate) fn init_logging() {
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

#[cfg(feature = "tonlibjson")]
pub(crate) async fn make_tl_client(mainnet: bool, archive_only: bool) -> anyhow::Result<ton_lib::tl_client::TLClient> {
    init_logging();
    log::info!("Initializing tl_client with mainnet={mainnet}...");
    let mut config = match mainnet {
        true => ton_lib::tl_client::TLClientConfig::new_mainnet(archive_only),
        false => ton_lib::tl_client::TLClientConfig::new_testnet(archive_only),
    };
    config.connections_count = 10;
    config.retry_strategy.retry_count = 10;
    let client = ton_lib::tl_client::TLClient::new(config).await?;
    ton_lib::sys_utils::sys_tonlib_set_verbosity_level(0);
    Ok(client)
}

pub async fn make_lite_client(mainnet: bool) -> anyhow::Result<LiteClient> {
    init_logging();
    log::info!("initializing lite_client with mainnet={mainnet}...");
    let mut config = LiteClientConfig::new(&TonNetConfig::get_json(mainnet))?;
    config.default_req_params.retries_count = 20;
    config.default_req_params.retry_waiting = Duration::from_millis(200);
    Ok(LiteClient::new(config)?)
}
