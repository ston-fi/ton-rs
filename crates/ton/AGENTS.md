# `ton` crate guide

`ton` is the public high-level crate. It owns TON block schemas, contract
wrappers, wallet message construction, lite-client networking, and optional
tonlib emulation.

Use `rust-library-review` for every non-trivial change.

## Public paths

- `block_tlb`: protocol wire types. Preserve prefixes, order, and layout.
- `contracts`: provider-neutral wrappers and TEP-specific modules.
- `ton_wallet`: keys, wallet data, code, signing, and external messages.
- `lite_client`: enabled by `lite-client`.
- `tl_client` and `emulators`: enabled by `tonlibjson`.

Prefer module-qualified APIs such as
`contracts::tep::jetton::jetton_master_contract::JettonMasterContract`. Do not
restore removed glob re-exports.

Open errors, enums, get-method results, decoded responses, and snapshots should
be `#[non_exhaustive]`. Fixed wire-format request/data structs stay exhaustive
when callers build them directly and a field addition would change their BOC or
JSON schema. Provide construction APIs before making a struct non-exhaustive.

`ContractClient` owns state caching and composes separate `StateProvider` and
`EmulationProvider` implementations. State-cache refresh needs Tokio. Native
emulator library caches belong to `TLEmulationProvider`, not `ContractClient`.

Errors must remain typed; preserve emulator exit codes and raw diagnostics.
Avoid `unwrap`, `expect`, and panic-driven production paths.

`contracts::tep::metadata::meta_loader::MetaLoader` uses the IPFS Foundation's best-effort public gateway by
default. Production consumers should configure a self-hosted or dedicated gateway through the builder.

Downstream setup:

```toml
[dependencies]
ton = "0.4"
```

Enable `lite-client` for native ADNL access or `tonlibjson` for tonlib clients
and emulators. Custom providers need neither feature.

Run the root fast/full validation commands. For crate-only iteration, start with
`cargo test -p ton --lib --all-features` and strict crate Rustdoc/Clippy.

## Downstream helpers to check before writing your own

Paths below are relative to this crate. Read the resolved `ton_core` and
`ton_macros` packages' `AGENTS.md` for address adapters, `TLB` attributes
(`bits_len`, `adapter`, `ensure_empty`), and typed stack/get-method macros.
With only a `ton` dependency, core types and derives are available through the
existing `ton::ton_core` path; dictionaries belong in `ton::tlb_adapters`.

### Dictionaries

Keep application fields as `std::collections::HashMap` and select the wire
adapter with `#[tlb(adapter = "...")]`. All names below are public through
`ton::tlb_adapters`; implementation files are in `src/tlb_adapters/dict/`.
The attribute is `tlb`, not the legacy `tlb_derive` spelling in some older
dictionary comments.

- `TLBHashMapE<KA, VA>::new(key_bits_len)` implements `HashmapE`: a presence bit
  and, when nonempty, a referenced dictionary. An empty map writes a zero bit.
- `TLBHashMap<KA, VA>::new(key_bits_len)` implements the inline dictionary
  without a presence bit and cannot write an empty map. Choose from the schema,
  not just whether the current Rust map happens to be empty.
- Keys: `DictKeyAdapterUint<T>`, `DictKeyAdapterInt<KEY_BITS_LEN, T>`,
  `DictKeyAdapterTonHash`, `DictKeyAdapterMsgAddress`,
  `DictKeyAdapterTonAddress`, and `DictKeyAdapterString`. Match key encoding
  and width to the schema. For signed keys, the const width must match the
  dictionary constructor's width. Address key adapters have their own codecs;
  do not assume they use `InternalAddress` semantics.
- Values: `DictValAdapterTLB<T>` uses `T: TLB` (including `TLBRef<T>` for `^T`),
  while `DictValAdapterNum<T, BITS_LEN>` encodes a fixed-width number.
- Implement `DictKeyAdapter` or `DictValAdapter` only for a missing encoding;
  reuse the dictionary tree/label implementation. `BinTree<VA>` is also
  available for the distinct shard binary-tree format.

```rust
use std::collections::HashMap;
use ton::tlb_adapters::{DictKeyAdapterInt, DictValAdapterNum, TLBHashMapE};
use ton::ton_core::TLB;
use ton::ton_core::traits::tlb::TLB;

#[derive(Debug, PartialEq, TLB)]
#[tlb(ensure_empty = true)]
struct Counters {
    #[tlb(adapter = "TLBHashMapE::<DictKeyAdapterInt<24, i32>, DictValAdapterNum<u64, 64>>::new(24)")]
    by_index: HashMap<i32, u64>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let counters = Counters { by_index: HashMap::from([(-1, 7), (2, 9)]) };
    assert_eq!(Counters::from_cell(&counters.to_cell()?)?, counters);
    Ok(())
}
```

For real schemas see `src/block_tlb/state_init.rs` and for boundary fixtures
see `src/tests/test_dictionary.rs`. The repository's
`examples/ton_types_tlb_build_dict_large.rs` demonstrates dictionary assembly.

### Contracts, standard messages, and metadata

- Use `ton_contract!(Name<Storage>: Methods)` and `#[ton_methods]` before
  implementing `TonContract` and stack packing manually. The `ton_macros` guide
  documents naming overrides and `FromTVMStack` strictness flags.
- Use `contracts::ContractClient::builder(state_provider, emulation_provider)`
  for custom provider integration. Core `StateProvider`/`EmulationProvider`
  traits let wrappers work without native features. `TonContract::new` is
  synchronous and lazy; `load_state`/`load_parsed_data` perform loading.
- `with_default_caches()` on the client configures state caches and needs an
  active Tokio runtime. Native emulator library caches are configured separately
  on `emulators::tl_emulation_provider::TLEmulationProvider` (`tonlibjson`).
- Search `src/block_tlb/` for existing message, transaction, state-init and TVM
  types; search `src/contracts/tep/` for Jetton/NFT/SBT messages, wrappers and
  result records before copying schemas or opcodes into another project.
- `contracts::tep::snake_data::SnakeData` handles byte-aligned snake data;
  `contracts::tep::metadata` owns metadata content/fields and `MetaLoader`.
  Check these before implementing cell-chain string parsing or metadata loading.
- Software wallet identity/message signing belongs to `ton_wallet::TonWallet`;
  reuse its version-specific implementation. Hardware signing belongs to
  `ton_ledger`, which has its own guide and transport lifecycle.

Compile downstream examples with the features actually enabled by the consumer.
Use deterministic schema/ABI fixtures; do not run network or transfer examples
as a codec smoke test.

## Dictionary invariants

Signed dictionary key adapters must reject values outside their declared signed
width before converting to two's complement. Dictionary readers bound every
label by the remaining key width, including inside forks, and accept all valid
short, long, and same label encodings, including zero-length labels. Invalid
keys and labels return typed errors rather than panicking or aliasing keys.

## Jetton discovery

TEP-89 request/reply wire records live in their named Jetton modules. Preserve
raw nullable addresses and the absent/present owner reference. Strict reply
parsing must validate that reference without changing global TLBRef semantics.
Do not place master discovery requests in the wallet message enum or equate
wire parsing with source authentication or request correlation.
