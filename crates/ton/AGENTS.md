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
