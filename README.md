# ton-rs

Set of general-purpose rust libraries to interact with [TON](https://ton.org/) blockchain.

[![CI](https://github.com/sild/ton_lib_rs/actions/workflows/build.yml/badge.svg)](https://github.com/sild/ton_lib_rs/actions/workflows/build.yml)
[![Crates.io](https://img.shields.io/crates/v/ton_lib.svg)](https://crates.io/crates/ton_lib)
[![codecov](https://codecov.io/gh/sild/ton_lib_rs/branch/main/graph/badge.svg)](https://codecov.io/gh/sild/ton_lib_rs)

This crate is heavily based on the [tonlib-rs](https://github.com/ston-fi/tonlib-rs) repository and also uses [tonlib-sys](https://github.com/ston-fi/tonlib-sys) underneath for the [tonlibjson_client](ton/src/clients/tonlibjson) implementation.

## ton_macros

- TLB Derive - Automatically derive TLB trait for your types based on it's members
- TonContract Derive - Automatically derive TonContract trait for your contract wrappers

## ton_core
- [TonCell](ton_core/src/cell/ton_cell.rs)
- [TonAddress](ton_core/src/types/ton_address.rs)
- [TLB](ton_core/src/traits/tlb.rs) - Trait allows you read/write arbitrary objects in BOC format
- [Types](ton_core/src/types) - Few basic types, common and stable enough to be in core

## ton
- `tonlibjson` feature: Disabled by default. Enable it if you need `TLClient`, `Emulator` or `TonContract` functionality.
- Use `TON_NET_CONF_MAINNET_PATH` or `TON_NET_CONF_TESTNET_PATH` env variables to override `netconfig.json` and use your own TON nodes.
- [TLBAdapters](ton/src/tlb_adapters.rs) - Allows you to work with rust types like HashMap, and still serialize it properly for TON
- [BlockTLB](ton/src/block_tlb.rs) - Bunch of types to interact with raw blockchain data (However it's not fully covered)
- [TonWallet](ton/src/ton_wallet.rs) - Wrapper of wallet to sign and create external messages
- [TLClient](ton/src/tl_client.rs) - Using `tonlibjson` to interact with TON network
- [TonContract](ton/src/contracts/ton_contract.rs) - Use it to get data or execute methods on TON contracts


## Getting started
Examples can be found in `examples` folder (feel free to add your own).

### Basic usage
```rust
// Build and read custom cells
fn main() -> anyhow::Result<()> {
    use ton_lib::cell::ton_cell::TonCell;
    let mut builder = TonCell::builder();
    builder.write_bits([1,2,3], 24).unwrap();
    let cell = builder.build().unwrap();
    assert_eq!(cell.data, vec![1, 2, 3]);
    let mut parser = cell.parser();
    let data = parser.read_bits(24).unwrap();
    assert_eq!(data, [1, 2, 3]);
}

// describe TLB type:
#[derive(Debug, Clone, PartialEq, TLBDerive)]
#[tlb_derive(ensure_empty = true)]
pub struct StateInit {
    #[tlb_derive(bits_len = 5)]
    pub split_depth: Option<u8>,
    pub tick_tock: Option<TickTock>,
    pub code: Option<TonCellRef>,
    pub data: Option<TonCellRef>,
    #[tlb_derive(adapter = "DictRef::<DictKeyAdapterTonHash, DictValAdapterTLB, _, _>::new(256)")]
    pub library: LibsDict,
}

fn main() {
    let boc_hex = "b5ee9c720102160100030400020134020100510000082f29a9a31738dd3a33f904d35e2f4f6f9af2d2f9c563c05faa6bb0b12648d5632083ea3f89400114ff00f4a413f4bcf2c80b03020120090404f8f28308d71820d31fd31fd31f02f823bbf264ed44d0d31fd31fd3fff404d15143baf2a15151baf2a205f901541064f910f2a3f80024a4c8cb1f5240cb1f5230cbff5210f400c9ed54f80f01d30721c0009f6c519320d74a96d307d402fb00e830e021c001e30021c002e30001c0039130e30d03a4c8cb1f12cb1fcbff08070605000af400c9ed54006c810108d718fa00d33f305224810108f459f2a782106473747270748018c8cb05cb025005cf165003fa0213cb6acb1f12cb3fc973fb000070810108d718fa00d33fc8542047810108f451f2a782106e6f746570748018c8cb05cb025006cf165004fa0214cb6a12cb1fcb3fc973fb0002006ed207fa00d4d422f90005c8ca0715cbffc9d077748018c8cb05cb0222cf165005fa0214cb6b12ccccc973fb00c84014810108f451f2a702020148130a0201200c0b0059bd242b6f6a2684080a06b90fa0218470d4080847a4937d29910ce6903e9ff9837812801b7810148987159f31840201200e0d0011b8c97ed44d0d70b1f8020158120f02012011100019af1df6a26840106b90eb858fc00019adce76a26840206b90eb85ffc0003db29dfb513420405035c87d010c00b23281f2fff274006040423d029be84c6002e6d001d0d3032171b0925f04e022d749c120925f04e002d31f218210706c7567bd22821064737472bdb0925f05e003fa403020fa4401c8ca07cbffc9d0ed44d0810140d721f404305c810108f40a6fa131b3925f07e005d33fc8258210706c7567ba923830e30d03821064737472ba925f06e30d1514008a5004810108f45930ed44d0810140d720c801cf16f400c9ed540172b08e23821064737472831eb17080185005cb055003cf1623fa0213cb6acb1fcb3fc98040fb00925f03e2007801fa00f40430f8276f2230500aa121bef2e0508210706c7567831eb17080185004cb0526cf1658fa0219f400cb6917cb1f5260cb3f20c98040fb0006";
    let state_init = StateInit::from_boc_hex(boc_hex).unwrap();
}

// create & use TLClient
pub(crate) async fn main() -> anyhow::Result<()> {
    let mainnet = true; // or false for testnet
    let archive_only = true; // set to true if you want to use archive nodes only
    init_logging();
    let mut config = match mainnet {
        true => TLClientConfig::new_mainnet(archive_only),
        false => TLClientConfig::new_testnet(archive_only),
    };
    config.connections_count = 10;
    config.retry_strategy.retry_count = 10;
    let client = TLClient::new(config).await?;
    sys_tonlib_set_verbosity_level(0);

    let mc_info = tl_client.get_mc_info().await?;
    let block = tl_client.lookup_mc_block(mc_info.last.seqno - 100).await?;
    Ok(())
}
```

## Contribution

If you face with some unclear parts or bugs, your can add a new example or improve documentation.

If you implemented some general feature, please make sure it's covered by unit tests (except `TLClient` part, for sure it's hard to cover it by unit tests)
