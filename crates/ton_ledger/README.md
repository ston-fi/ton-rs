# ton_ledger

A Ledger-backed TON wallet with V3R2/V4R2 message construction, USB HID,
optional Bluetooth, and custom transports. Requires native Tokio with its time
driver enabled. Rust 1.94 or later. No network provider is owned by the wallet.

The public API has three modules:

| Module | Contents |
| --- | --- |
| `ton_ledger_wallet` | `TonLedgerWallet` and its `builder`, `config`, `app`, `proof`, and `data` modules. |
| `transports` | The `Transport` extension trait and feature-gated `hid`/`ble` backends. |
| `error` | Wallet results and typed wallet/transport errors. |

`ton_ledger_wallet::config` owns `DerivationPath`, `SigningPolicy` and
`AddressOptions`. Proof/data modules expose requests and verified results;
firmware encoding, hashing, hints and session machinery are private.

```rust,no_run
use ton::ton_wallet::WalletVersion;
use ton_ledger::ton_ledger_wallet::TonLedgerWallet;
use ton_ledger::ton_ledger_wallet::config::AddressOptions;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let mut wallet = TonLedgerWallet::builder(WalletVersion::V4R2).build().await?;
wallet.confirm_address(AddressOptions::default()).await?;
println!("{}", wallet.address());
# Ok(())
# }
```

Construct one internal `Msg` with IHR disabled, no source, no extra currencies,
zero fees/timestamps, and a standard workchain 0/-1 destination. Empty bodies
must be inline; present payloads and state init must be references. No layout is
silently rewritten. Then use `create_ext_in_body(expiry, seqno, vec![message])`,
`sign_ext_in_body(&body).await`, and `create_ext_in_msg_from_body(signed, deploy)`;
`create_ext_in_msg(...).await` composes these. Send mode defaults to 3. Existing
wallet TLB records can express supported custom modes. Signed cells are locally
verified; they are not transaction IDs or proof of inclusion.

For example, construct an empty-body transfer with explicit inline placement.
The caller supplies the destination, current seqno and expiry:

```rust,no_run
use ton::block_tlb::{CommonMsgInfoInt, Msg};
use ton::ton_core::{
    cell::TonCell,
    traits::tlb::TLB,
    types::{TonAddress, tlb_core::{EitherRefLayout, TLBCoins}},
};
use ton_ledger::{error::TonLedgerResult, ton_ledger_wallet::TonLedgerWallet};

async fn transfer(
    wallet: &mut TonLedgerWallet,
    destination: TonAddress,
    seqno: u32,
    expire_at: u32,
) -> TonLedgerResult<TonCell> {
    let mut message = Msg::new(
        CommonMsgInfoInt::new(destination.to_msg_address_int().into(), TLBCoins::new(10_000_000)),
        TonCell::empty().clone(),
    );
    message.body.layout = EitherRefLayout::ToCell;
    wallet.create_ext_in_msg(vec![message.to_cell()?], seqno, expire_at, false).await
}
```

For a nonempty payload, supply its cell as the body and use
`EitherRefLayout::ToRef`. Attach state init with `TLBEitherRef::new_ref(state_init)`
and explicitly select `SigningPolicy::AllowOpaque` on the wallet builder.

The default path is `44'/607'/0'/0'/0'/0'`. Use
`with_derivation_path(DerivationPath::Ton { account, testnet })`; workchain -1
selects chain component 255. `Custom(Vec<u32>)` accepts 3–10 unhardened indexes
with prefix 44/607 and hardens each exactly once. Testnet key derivation and
friendly-address display flags are independent; neither provides transaction
network separation. Wallet IDs preserve all 32 bits through `i32`.

Wallet operations return `error::TonLedgerResult<T>`, using the typed
`error::TonLedgerError`. Transport implementations use `error::TransportError`.

## Features and platforms

- Default `hid`: `hidapi` 2; `HidTransport::discover(timeout)` and
  `connect(device)` support explicit selection. Default build selects exactly one.
- `ble`: `btleplug` 0.13; `BleTransport::discover(timeout)` and `connect(device)`.
  Service UUIDs identify normal-mode Nano X, Stax, Flex and Nano Gen5 interfaces.
  Device model identifiers are not a claim of physical-device validation.
- No default features: supply `with_transport(impl transports::Transport)`; otherwise
  `build()` returns `MissingTransport`. The trait exchanges short APDUs including
  response status bytes. `Send` futures are supported; `Sync` is not required.

macOS uses native HID/CoreBluetooth. Grant Bluetooth permission to the terminal
or host application; packaged applications need an NSBluetoothAlwaysUsageDescription.
Linux requires pkg-config, libudev development headers for HID, and libdbus-1
headers/BlueZ for BLE, plus OS device permissions. Windows uses native HID and
WinRT BLE. Only the local macOS build has been checked during implementation;
Linux CI is configured to build the feature combinations. No browser/WASM or no_std support.

Discovery and connection are bounded. Request/approval defaults are 10/180
seconds, configurable on the builder. HID and BLE connect budgets are 10/20
seconds. BLE discovery can add up to two seconds to stop scanning before returning;
connection cleanup takes up to four seconds in the background. Blocking OS HID enumeration or
writes cannot be forcibly interrupted; timed-out callers return, and workers
release resources after the OS call returns. HID reads poll at most every 50 ms.
Drop releases session ownership. There is no automatic reconnect, retry or
USB/BLE fallback. Cancellation or uncertain I/O poisons the wallet; drop it,
resolve any device prompt, reconnect and build again. Reconnecting does not
cancel a pending prompt on the device. Custom transports must enforce exclusive
physical-device access for the whole wallet lifetime.

BLE rejects duplicate connections to the same backend device in this process
with `TransportError::DeviceBusy`, including while an old worker is cleaning up.
The lease covers cloned and rediscovered handles. It does not coordinate other
applications or USB access to the same physical Ledger; close those sessions first.

## Signing scope

The compatibility profile is the LedgerHQ TON app **2.9.1**, pinned at
`849962d6378567f5aaf708b116292ccf7b17a97c`. Other version strings fail explicitly.
Version checking does not attest a firmware binary, and does not infer support
from `version >= minimum`. V5, multiple messages and extra currencies are rejected.
The APDU payload limit is 255 bytes; chunked transaction/data payloads total at
most 510 bytes. No key export, app installation, background device discovery or
balance cache is provided.

All 14 baseline hint families are recognized: ASCII comments, Jetton transfers,
NFT transfers, Jetton burns, whitelist additions, nominator withdrawals and
validator changes, Tonstakers deposits, proposal votes, DNS updates, bridge swaps,
TonWhales deposits/withdrawals, and vesting messages with comments. Recognition
preserves exact cells and rejects trailing data. Known token registry IDs are
not inferred from transaction fields; generic Jetton amounts have no asserted
ticker/decimals. `SigningPolicy::ClearOnly` is the default. State init, unknown
payloads, unknown DNS records, and opaque nested custom/forward fields require
`AllowOpaque`; the device displays hashes and may require blind signing enabled.
A recognized outer opcode does not make its nested content clear.

Coverage is by hint family, not every firmware-supported variant. The encoder
uses generic Jetton hints rather than the firmware's optional hardcoded token
registry. It also currently rejects zero TonWhales query IDs in clear mode,
although the pinned firmware parser accepts them; `AllowOpaque` falls back to
hash-only signing for those payloads. Exact layout and policy restrictions above
still apply. Other firmware versions and hardware behavior are not validated.

A private TLB enum parses supported payloads and validates exact cell layouts.
Hint encoding uses ordered field mappings over the existing TON message types.
Callers keep constructing TEP messages and passing their cells to the wallet; no additional derive or Ledger-specific message struct is required.

`get_address_proof` reconstructs the TON proof digest from the bound address,
UTF-8 domain, timestamp and payload. Domain/payload are each at most 128 bytes
and must fit one APDU including path/specifiers (216 combined bytes for six
components). `sign_data` supports Ledger's legacy plaintext and app-data
schemas, not TON Connect signData. Plaintext is at most 120 printable ASCII
bytes. App-data requires address or domain; domain is at most 126 printable ASCII
bytes. Signatures cover schema BE32, timestamp BE64 and cell hash, directly.

## Manual USB/Bluetooth self-transfer

From this repository:

```sh
cargo run -p examples --example ton_ledger_bluetooth_self_transfer --features ledger-ble
```

The example prefers a connected USB Ledger and scans Bluetooth only when no USB
Ledger is found. USB discovery/connection errors stop the example; if multiple USB
Ledgers are connected, leave only the intended one connected. The `ledger-ble`
example feature enables both HID and BLE. Bluetooth scanning lists nearby Ledgers
and lets you choose one or rescan. Enter `q`
at a prompt to quit. It confirms the address on the device, reads the seqno,
signs **10,000,000 nanotons to itself**, and broadcasts through one mainnet
endpoint without retries. It uses account zero, V4R2 and workchain zero.
Fund and deploy that wallet before running it; network fees reduce its balance.
The printed acknowledgement does not confirm transaction inclusion.

CI compiles the example without running it. Hardware pairing, approval,
disconnect behavior and a live self-transfer require manual acceptance.
