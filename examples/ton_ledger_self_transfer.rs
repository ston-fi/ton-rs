//! Sends 0.01 TON to a funded, deployed V4R2 mainnet Ledger wallet itself.
//! Optionally includes a random opaque payload to exercise blind signing.
//! Prefers USB; scans Bluetooth when no USB Ledger is connected.
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
    ton_ledger_wallet::{
        TonLedgerWallet,
        config::{AddressOptions, DerivationPath, SigningPolicy},
    },
    transports::{
        ble::{BleDeviceInfo, BleTransport},
        hid::HidTransport,
    },
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
    println!("Mainnet · V4R2 · account 0 · self-transfer 0.01 TON (plus network fees).");
    let Some(policy) = select_signing_policy()? else { return Ok(()) };
    let payload = transfer_payload(policy)?;
    let Some(mut wallet) = connect_wallet(policy).await? else { return Ok(()) };

    println!("Confirm the wallet address on your Ledger.");
    wallet.confirm_address(AddressOptions::default().with_testnet(false)).await?;
    println!("Wallet: {}", wallet.address().to_base64(true, true, true));

    let client = mainnet_client()?;
    println!("Reading the deployed wallet's state from mainnet…");
    let block = client.get_mc_info().await.context("Could not reach a mainnet lite server")?;
    let account = client.get_account_state(wallet.address(), block.last.seqno, None).await?;
    let wallet_data = WalletV4Data::from_cell(account.get_data().context("fund and deploy the wallet first")?)?;

    let transfer = Msg {
        info: CommonMsgInfoInt {
            bounce: false,
            ..CommonMsgInfoInt::new(wallet.address().to_msg_address_int().into(), TLBCoins::new(10_000_000))
        }
        .into(),
        init: None,
        body: payload,
    }
    .to_cell()?;
    let expires_at = SystemTime::now() + Duration::from_secs(600);
    let expire_at = u32::try_from(expires_at.duration_since(UNIX_EPOCH)?.as_secs())?;
    println!("Review and approve the 0.01 TON self-transfer on your Ledger.");
    let message = wallet.create_ext_in_msg(transfer, wallet_data.seqno, expire_at, false).await?;
    println!("Broadcasting the approved transaction…");
    let status = client
        .send_msg(message.to_boc()?, Some(LiteReqParams::new(0, 0, 5000)))
        .await
        .context("Broadcast result is uncertain; check wallet history before sending again")?;
    println!("Broadcast acknowledged ({status}); transaction inclusion is not yet confirmed.");
    Ok(())
}

fn select_signing_policy() -> io::Result<Option<SigningPolicy>> {
    loop {
        let Some(choice) = prompt("Enter = plain transfer, b = random blind-signing payload, q = quit: ")? else {
            return Ok(None);
        };
        if choice.is_empty() {
            return Ok(Some(SigningPolicy::ClearOnly));
        }
        if choice.eq_ignore_ascii_case("b") {
            println!("Enable blind signing in the Ledger TON app settings before continuing.");
            println!("The payload will be displayed by hash, not as a readable message.");
            return Ok(Some(SigningPolicy::AllowOpaque));
        }
        println!("Choose Enter, b, or q.");
    }
}

fn transfer_payload(policy: SigningPolicy) -> anyhow::Result<TLBEitherRef<TonCell>> {
    if policy == SigningPolicy::ClearOnly {
        return Ok(TLBEitherRef::new_with_layout(TonCell::empty().clone(), EitherRefLayout::ToCell));
    }
    let random_bytes = rand::random::<[u8; 32]>();
    let mut builder = TonCell::builder();
    // An unrecognized opcode guarantees the random bytes cannot become a clear-signing hint.
    builder.write_num(&0xdead_beefu32, 32)?;
    builder.write_bits(random_bytes, 256)?;
    let payload = builder.build()?;
    println!("Payload bytes: deadbeef{}", hex::encode(random_bytes));
    println!("Payload cell hash: {}", hex::encode(payload.cell_hash()?.as_slice()));
    Ok(TLBEitherRef::new_with_layout(payload, EitherRefLayout::ToRef))
}

async fn connect_wallet(policy: SigningPolicy) -> anyhow::Result<Option<TonLedgerWallet>> {
    println!("Unlock your Ledger and open the TON app. Connect a USB cable if available.");
    if prompt("Press Enter to connect, or q to quit: ")?.is_none() {
        return Ok(None);
    }
    println!("Checking for a USB Ledger…");
    let mut devices = HidTransport::discover(Duration::from_secs(10))
        .await
        .context("Could not discover USB Ledgers; check USB permissions and the cable")?;
    anyhow::ensure!(devices.len() <= 1, "Multiple USB Ledgers found; leave only the intended device connected");
    let builder = TonLedgerWallet::builder(WalletVersion::V4R2);
    let builder = if let Some(device) = devices.pop() {
        println!("Connecting via USB…");
        let transport = HidTransport::connect(device)
            .await
            .context("Could not connect via USB; check the cable and close other Ledger connections")?;
        builder.with_transport(transport)
    } else {
        println!("No USB Ledger found. Enable Bluetooth on your Ledger to scan.");
        if prompt("Press Enter to scan Bluetooth, or q to quit: ")?.is_none() {
            return Ok(None);
        }
        let Some(device) = select_bluetooth_device().await? else { return Ok(None) };
        println!("Connecting via Bluetooth… Accept pairing on your Ledger if prompted.");
        let transport = BleTransport::connect(device)
            .await
            .context("Could not connect; check pairing and close other Ledger connections")?;
        builder.with_transport(transport)
    };
    let wallet = builder
        .with_signing_policy(policy)
        .with_derivation_path(DerivationPath::Ton {
            account: 0,
            testnet: false,
        })
        .build()
        .await
        .context("Could not open the wallet; unlock the Ledger and open the TON app")?;
    Ok(Some(wallet))
}

fn mainnet_client() -> anyhow::Result<LiteClient> {
    let mut config = TonNetConfig::new_default(true)?;
    config.lite_endpoints.truncate(1);
    Ok(LiteClient::builder()?
        .with_mainnet(true)?
        .with_net_config(config)
        .with_default_req_params(LiteReqParams::new(0, 0, 5000))
        .build()?)
}

async fn select_bluetooth_device() -> anyhow::Result<Option<BleDeviceInfo>> {
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
