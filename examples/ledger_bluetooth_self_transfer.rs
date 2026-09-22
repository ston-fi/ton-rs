//! Sends 0.01 TON to a funded, deployed V4R2 testnet Ledger wallet itself.
//! Run manually with the TON app open; approval spends network fees.
use anyhow::{Context, ensure};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use ton::{
    block_tlb::{CommonMsgInfoInt, Msg},
    lite_client::{LiteClient, LiteReqParams},
    net_config::TonNetConfig,
    ton_core::{
        cell::TonCell,
        traits::tlb::TLB,
        types::tlb_core::{EitherRefLayout, TLBCoins, TLBEitherRef},
    },
    ton_wallet::{WalletV4Data, WalletVersion},
};
use ton_ledger::{
    app::AddressOptions, derivation_path::DerivationPath, ton_ledger_wallet::TonLedgerWallet,
    transports::ble::BleTransport,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let devices = BleTransport::discover(Duration::from_secs(10)).await?;
    ensure!(devices.len() == 1, "connect exactly one Ledger device");
    let transport = BleTransport::connect(devices.into_iter().next().context("no Ledger found")?).await?;
    let mut wallet = TonLedgerWallet::builder(WalletVersion::V4R2)
        .with_transport(transport)
        .with_derivation_path(DerivationPath::Ton {
            account: 0,
            testnet: true,
        })
        .build()
        .await?;
    wallet.confirm_address(AddressOptions::default().with_testnet(true)).await?;
    println!("Wallet: {}", wallet.address().to_base64(false, true, true));

    let mut config = TonNetConfig::new_default(false)?;
    config.lite_endpoints.truncate(1);
    let client = LiteClient::builder()?
        .with_mainnet(false)?
        .with_net_config(config)
        .with_default_req_params(LiteReqParams::new(0, 0, 5000))
        .build()?;
    let block = client.get_mc_info().await?;
    let account = client.get_account_state(wallet.address(), block.last.seqno, None).await?;
    let data = WalletV4Data::from_cell(account.get_data().context("fund and deploy the wallet first")?)?;

    let transfer = Msg {
        info: CommonMsgInfoInt {
            bounce: false,
            ..CommonMsgInfoInt::new(wallet.address().to_msg_address_int().into(), TLBCoins::new(10_000_000))
        }
        .into(),
        init: None,
        body: TLBEitherRef::new_with_layout(TonCell::empty().clone(), EitherRefLayout::ToCell),
    }
    .to_cell()?;
    let expire_at =
        u32::try_from((SystemTime::now().duration_since(UNIX_EPOCH)? + Duration::from_secs(600)).as_secs())?;
    let message = wallet.create_ext_in_msg(vec![transfer], data.seqno, expire_at, false).await?;
    let status = client.send_msg(message.to_boc()?, Some(LiteReqParams::new(0, 0, 5000))).await?;
    println!("Broadcast acknowledged ({status}); transaction inclusion is not yet confirmed.");
    Ok(())
}
