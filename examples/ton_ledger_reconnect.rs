//! Reads wallet addresses before and after replacing a USB Ledger in one process.
//! Connect the first Ledger and open its TON app before running this example.
//! Does not sign or broadcast transactions; no funded wallet is required.
use std::{error::Error, io, time::Duration};
use ton::ton_wallet::WalletVersion;
use ton_ledger::ton_ledger_wallet::TonLedgerWallet;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    print_address().await?;

    println!("Replace the USB Ledger and open the TON app on the replacement.");
    // Exceed Tokio's default 10-second blocking-thread idle timeout. Before
    // the fix for #231, macOS HIDAPI could retain that retired thread's run loop.
    tokio::time::sleep(Duration::from_secs(11)).await;

    println!("Press Enter when the replacement Ledger is ready.");
    if io::stdin().read_line(&mut String::new())? == 0 {
        return Err(io::Error::from(io::ErrorKind::UnexpectedEof).into());
    }

    print_address().await?;
    Ok(())
}

async fn print_address() -> Result<(), Box<dyn Error>> {
    let wallet = TonLedgerWallet::builder(WalletVersion::V4R2).build().await?;
    println!("Address: {}", wallet.address());
    drop(wallet);
    Ok(())
}
