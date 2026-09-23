# ton-rs

Set of general-purpose rust libraries to interact with [TON](https://ton.org/) blockchain.

[![CI](https://github.com/ston-fi/ton-rs/actions/workflows/build.yml/badge.svg)](https://github.com/ston-fi/ton-rs/actions/workflows/build.yml)
[![Crates.io](https://img.shields.io/crates/v/ton.svg)](https://crates.io/crates/ton)

This crate is heavily based on the [tonlib-rs](https://github.com/ston-fi/tonlib-rs) repository and also uses [tonlib-sys](https://github.com/ston-fi/tonlib-sys) underneath for the [tonlibjson_client](crates/ton/src/tl_client.rs) implementation.

## ton_macros

- `TLB` Derive macros: Automatically derive TLB trait for your types based on it's members
- Native `Enum` support using TLBPrefix: Automatically match underlying variant by it's prefix (check [ton_core_enum.rs](examples/ton_core_enum.rs) example). Provides powerful enums, but use them carefully; read the [Enum with TLB macros](#enum-with-tlb-macros) chapter.
- `#[ton_methods]`: Generate async get-method implementations for contract traits or impl blocks, with optional block-level `name_format` conversion and per-method exact names.

## ton_core
- `serde` feature: provides few mods to ser/de core types, check [ton_core/src/serde.rs](crates/ton_core/src/serde.rs). Disabled by default.
- [EmulationProvider](crates/ton_core/src/traits/emulation_provider.rs) - Provider-neutral interface used to execute TVM get methods
- [TonCell](crates/ton_core/src/cell/ton_cell.rs)
- [TonAddress](crates/ton_core/src/types/ton_address.rs)
- [TLB](crates/ton_core/src/traits/tlb.rs) - Trait allows you read/write arbitrary objects in BOC format
- [Types](crates/ton_core/src/types) - Few basic types, common and stable enough to be in core

## ton
- `ton_contract!`: Generate a `TonContract` wrapper type and optionally implement method traits for it.
- `lite-client` feature: Disabled by default. Enable it for the ADNL-based `LiteClient`; it also enables the networking dependencies required by that client.
- `tonlibjson` feature: Disabled by default. Enable it for the native `TLClient`, emulator implementations, `TLStateProvider`, and `ton::emulators::tl_emulation_provider::TLEmulationProvider`. `ContractClient` and `TonContract` can be used without it by supplying custom providers.
  This feature includes `lite-client` because `TLClient` uses it to refresh the network configuration's init block.
- `WalletVersion` and `LiteNodeFilter` serialize using their Rust variant names; `TVMGetMethodID` serializes as an integer or string according to its variant.
- Use `TON_NET_CONF_MAINNET_PATH` or `TON_NET_CONF_TESTNET_PATH` env variables to override `netconfig.json` and use your own TON nodes.
- [TLBAdapters](crates/ton/src/tlb_adapters.rs) - Allows you to work with rust types like HashMap, and still serialize it properly for TON
- [BlockTLB](crates/ton/src/block_tlb.rs) - Bunch of types to interact with raw blockchain data (However it's not fully covered)
- [TonWallet](crates/ton/src/ton_wallet.rs) - Wrapper of wallet to sign and create external messages
- [TLClient](crates/ton/src/tl_client.rs) - Using `tonlibjson` to interact with TON network
- [TonContract](crates/ton/src/contracts/ton_contract.rs) - Use it with `ContractClient::builder(state_provider, emulation_provider)` to get data or execute methods on TON contracts
- Standard Jetton, NFT, SBT, and TON wallet contract wrappers live under `contracts::tep`, grouped into public modules by standard and implementation. For example, use `contracts::tep::jetton::jetton_master_contract::JettonMasterContract` or `contracts::tep::ton_wallet::TonWalletContract`.
- `contracts::tep::metadata::meta_loader::MetaLoader` resolves `ipfs://` metadata through the IPFS Foundation's best-effort public gateway by default. Production applications should configure their own gateway with `MetaLoader::builder().with_ipfs_base_url(...)`.

`ContractClient::builder(...).with_default_caches()` configures state caches. When using the native adapter, configure emulator library caches independently with `ton::emulators::tl_emulation_provider::TLEmulationProvider::with_default_caches()`.
State caches require an active Tokio runtime when the client is built and start a background refresh task. Dropping the client does not cancel an in-flight provider call; initial sequence discovery also keeps retrying provider errors until it succeeds.
`TonContract::new()` synchronously stores the contract address and optional transaction ID. `load_state()` and `load_parsed_data()` load and retain state; emulation uses retained state when available. Otherwise, it lets the emulation provider resolve the address and transaction ID unless `EmulationProvider::requires_resolved_state()` returns `true`, in which case the configured state provider loads it first.

`Mnemonic` clears its owned words and password on drop, and `KeyPair` clears its
secret key bytes. Caller-owned mnemonic strings and copies read from the public
`KeyPair::secret_key` field remain the caller's responsibility.

## Ledger wallets

[`ton_ledger`](crates/ton_ledger/README.md) provides V3R2/V4R2 signing through
USB HID (default), optional Bluetooth, or an exclusive custom transport.
Signatures and exact-message hashes are checked locally. The library owns no
network provider. Its firmware profile is pinned to TON app 2.9.1; hardware
acceptance remains separate from deterministic tests. A manual
[USB-first self-transfer example](examples/ton_ledger_bluetooth_self_transfer.rs)
shows how to sign and broadcast 0.01 TON to the same deployed mainnet wallet. See the crate README for platform setup and limits.

## Rust version

The minimum supported Rust version (MSRV) is 1.94. CI verifies every published
crate, target, and feature against the declared MSRV and checks weekly whether
the latest dependency releases still support it. MSRV increases are released
as minor compatibility changes.


## Getting started
Examples can be found in [examples](examples) folder (feel free to add your own)

- [ton_emulate_get_method](examples/ton_emulate_get_method.rs): network-backed contract emulation; requires `--features tonlibjson`.
- [ton_transfer](examples/ton_transfer.rs): signs and broadcasts a transfer; requires `--features tonlibjson` and deliberate account/recipient configuration.
- [ton_ledger_bluetooth_self_transfer](examples/ton_ledger_bluetooth_self_transfer.rs): USB-first Ledger signing with Bluetooth fallback; requires `--features ledger-ble` and spends mainnet fees.

Run examples through `cargo run -p examples --example <name> --features <features>`.
Compilation is separate from execution; do not run transfer examples as smoke tests.

### Basic usage

Build and read a cell with `ton_core`:

```rust
use ton_core::cell::TonCell;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = TonCell::builder();
    builder.write_bits([1, 2, 3], 24)?;
    let cell = builder.build()?;
    let mut parser = cell.parser();
    assert_eq!(parser.read_bits(24)?, [1, 2, 3]);
    parser.ensure_empty()?;
    Ok(())
}
```

Derive a TLB codec and round-trip a typed record:

```rust
use ton_core::TLB;
use ton_core::traits::tlb::TLB;

#[derive(Debug, PartialEq, TLB)]
#[tlb(prefix = 0xc4, bits_len = 8)]
struct GlobalVersion {
    version: u32,
    capabilities: u64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let original = GlobalVersion { version: 1, capabilities: 0 };
    let cell = original.to_cell()?;
    assert_eq!(GlobalVersion::from_cell(&cell)?, original);
    Ok(())
}
```

### Enum with TLB macros
TLB macros can derive TLB for enums. You can define enums with a common prefix or with no common prefix.
Enums without a common prefix are tricky: if you embed such an enum into another enum, its variants are effectively inlined into the outer enum.
```rust
use ton_core::TLB;

#[derive(TLB)]
#[tlb(prefix = 0b010, bits_len = 3)]
struct Variant1(u8);

#[derive(TLB)]
#[tlb(prefix = 0b011, bits_len = 3)]
struct Variant2(u8);

#[derive(TLB)]
enum InnerEnum { // No common prefix
    Variant1(Variant1), // Prefix = 0b010
    Variant2(Variant2), // Prefix = 0b011
}

#[derive(TLB)]
#[tlb(prefix = 0b1, bits_len = 1)] // Common prefix 
enum OuterEnum { 
    OuterVariant1(u16), // Prefix overall = 0b101
    OuterVariant2(InnerEnum),
}
```
This is effectively parsed as:
```rust
enum OuterEnum {
    OuterVariant1(u16), // Prefix overall = 0b101
    Variant1(u8),       // Prefix overall = 0b1010
    Variant2(u8),       // Prefix overall = 0b1011
}
```
Be careful with null (zero-length) prefixes. A null prefix acts like a wildcard; during parsing, variants are tried in declaration order, so a null-prefix variant placed earlier can consume the input before later variants are considered. See tests in [ton_core/src/traits/tlb/test_tlb_enum.rs](crates/ton_core/src/traits/tlb/test_tlb_enum.rs) for the shadowing and the safe-prefix example.

## Contribution

Repository and crate-specific guidance lives in [`AGENTS.md`](AGENTS.md).
Public API changes must preserve wire formats, document compatibility impact,
and keep one supported construction path. Open enums and returned records use
`#[non_exhaustive]`; fixed TLB records remain exhaustive when their fields are
the serialization contract.

If you face with some unclear parts or bugs, your can add a new example or improve documentation.

If you implemented some general feature, please make sure it's covered by tests (unit tests if possible)
