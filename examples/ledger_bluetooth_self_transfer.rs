//! Sends 0.01 TON to a funded, deployed V4R2 testnet Ledger wallet itself.
//! Run manually with the TON app open; approval spends network fees.
use anyhow::Context;
use std::{
    io::{self, Write},
    process::ExitCode,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
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
    app::AddressOptions,
    derivation_path::DerivationPath,
    ton_ledger_wallet::TonLedgerWallet,
    transports::ble::{BleDeviceInfo, BleTransport},
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    match transfer().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error:#}");
            ExitCode::FAILURE
        },
    }
}

async fn transfer() -> anyhow::Result<()> {
    println!("Testnet · V4R2 · account 0 · self-transfer 0.01 TON (plus network fees).");
    println!("Unlock your Ledger, enable Bluetooth, and open the TON app.");
    if prompt("Press Enter to scan, or q to quit: ")?.is_none() {
        return Ok(());
    }
    let Some(device) = select_device().await? else { return Ok(()) };
    println!("Connecting… Accept pairing on your Ledger if prompted.");
    let transport = BleTransport::connect(device)
        .await
        .context("Could not connect; check pairing and close other Ledger connections")?;
    let mut wallet = TonLedgerWallet::builder(WalletVersion::V4R2)
        .with_transport(transport)
        .with_derivation_path(DerivationPath::Ton {
            account: 0,
            testnet: true,
        })
        .build()
        .await
        .context("Could not open the wallet; unlock the Ledger and open the TON app")?;
    println!("Confirm the wallet address on your Ledger.");
    wallet.confirm_address(AddressOptions::default().with_testnet(true)).await?;
    println!("Wallet: {}", wallet.address().to_base64(false, true, true));

    let mut config = TonNetConfig::new_default(false)?;
    config.lite_endpoints.truncate(1);
    let client = LiteClient::builder()?
        .with_mainnet(false)?
        .with_net_config(config)
        .with_default_req_params(LiteReqParams::new(0, 0, 5000))
        .build()?;
    println!("Reading the deployed wallet's state from testnet…");
    let block = client.get_mc_info().await.context("Could not reach a testnet lite server")?;
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
    println!("Review and approve the 0.01 TON self-transfer on your Ledger.");
    let message = wallet.create_ext_in_msg(vec![transfer], data.seqno, expire_at, false).await?;
    println!("Broadcasting the approved transaction…");
    let status = client
        .send_msg(message.to_boc()?, Some(LiteReqParams::new(0, 0, 5000)))
        .await
        .context("Broadcast result is uncertain; check wallet history before sending again")?;
    println!("Broadcast acknowledged ({status}); transaction inclusion is not yet confirmed.");
    Ok(())
}

async fn select_device() -> anyhow::Result<Option<BleDeviceInfo>> {
    loop {
        println!("Scanning for Ledger devices (10-second scan budget, plus up to 2 seconds to stop scanning)…");
        let mut devices = match BleTransport::discover(Duration::from_secs(10)).await {
            Ok(devices) => devices,
            Err(error) => {
                eprintln!("Bluetooth scan failed: {error}");
                Vec::new()
            },
        };
        if devices.is_empty() {
            println!("No Ledger available. Check Bluetooth on both devices and keep the Ledger nearby and unlocked.");
            println!(
                "On macOS, allow Bluetooth for your terminal/IDE in System Settings → Privacy & Security → Bluetooth."
            );
            if prompt("Press Enter to rescan, or q to quit: ")?.is_none() {
                return Ok(None);
            }
            continue;
        }
        for (index, device) in devices.iter().enumerate() {
            println!("  {}. {} ({})", index + 1, device.name.as_deref().unwrap_or("Ledger"), device.id);
        }
        loop {
            let Some(choice) = prompt("Select a device number (Enter = 1), r to rescan, or q to quit: ")? else {
                return Ok(None);
            };
            if choice.eq_ignore_ascii_case("r") {
                break;
            }
            let selected = if choice.is_empty() { Some(1) } else { choice.parse::<usize>().ok() };
            if let Some(index) = selected.filter(|index| (1..=devices.len()).contains(index)) {
                return Ok(Some(devices.remove(index - 1)));
            }
            println!("Choose a number from 1 to {}.", devices.len());
        }
    }
}

fn prompt(message: &str) -> io::Result<Option<String>> {
    print!("{message}");
    io::stdout().flush()?;
    let mut input = String::new();
    if io::stdin().read_line(&mut input)? == 0 || input.trim().eq_ignore_ascii_case("q") {
        return Ok(None);
    }
    Ok(Some(input.trim().to_owned()))
}
