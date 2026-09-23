# ton-rs agent guide

## Repository

This workspace contains public Rust libraries for TON:

- `ton_core`: cells, addresses, TLB primitives, and provider traits.
- `ton`: block and contract types, wallets, clients, and optional emulators.
- `ton_ledger`: exclusive Ledger sessions, USB/BLE transports and verified signing.
- `ton_macros`: derives and attributes used by the other libraries.
- `examples` and `benchmarks`: consumers, not reusable API crates.

Example filenames and Cargo target names start with the crate they demonstrate:
`ton_ledger_`, `ton_core_`, or `ton_`. External comparison examples use their
actual crate prefix, such as `tonlib_core_`.

Use the `rust-library-review` workflow for public API, dependency, feature,
serialization, documentation, or release changes.

## Start here

Read the nearest guide before editing:

- [ton_core](crates/ton_core/AGENTS.md): cells, addresses, TLB and provider traits.
- [ton](crates/ton/AGENTS.md): clients, contracts and emulators.
- [Wallets](crates/ton/src/ton_wallet/AGENTS.md): software wallet identity and signing.
- [ton_ledger](crates/ton_ledger/AGENTS.md): hardware wallet protocol and lifecycle.
- [ton_macros](crates/ton_macros/AGENTS.md): macro expansion and diagnostics.

The root README owns workspace onboarding; crate READMEs and Rustdoc own consumer
contracts. Keep reusable invariants here and in the nearest guide, rather than
requiring agents to reconstruct them from implementation history. Examples live
in the separate `examples` package; compile them without running live transfers.

## Boundaries

Keep provider-neutral types in `ton_core`. Networking, contract wrappers, wallet
schemas/software signing, and native tonlib adapters belong in `ton`. Ledger
sessions, transport ownership and verified device signing belong in `ton_ledger`. Procedural macros must emit
paths that work when dependencies are renamed.

Do not add convenience re-exports or parallel constructors. Prefer the existing
module-qualified paths and one canonical construction path.

## Public API

- Mark open errors, enums, returned state, responses, and result records
  `#[non_exhaustive]` so variants and fields can grow compatibly.
- Keep fixed TLB/TL wire records exhaustive when callers must construct them and
  adding a field would change serialization. Document this exception near the
  type or in its crate guide.
- A non-exhaustive struct must have a constructor, builder, or another supported
  construction path.
- Keep Rustdoc short and behavioral. Document errors, panics, ownership,
  timeouts, caching, and feature requirements when relevant.
- Public enum additions, serialized names, feature changes, and dependency types
  are compatibility-sensitive even when code still compiles.

Any public surface change must review the applicable `AGENTS.md`, README,
Rustdoc, examples, tests, package include rules, and changelog.

## TON invariants

- Cell limits are 1023 data bits and four references.
- TLB prefixes, field order, reference layout, and signed-message layout are wire
  contracts. Never change them as cleanup.
- `TVMStack` pops from the end; serialized stack order is reversed while reading.
- Wallet code and initial data determine the address. Signing layout differs by
  wallet version.
- Provider state requested for a transaction must describe that exact
  transaction. Do not silently substitute latest state.

## Features

- Default `ton` builds provider-neutral contract APIs.
- `lite-client` enables the ADNL lite client.
- `tonlibjson` enables native tonlib clients and emulators and also enables
  `lite-client`.
- `ton_core/serde` enables serialization for core types.
- `ton_ledger` defaults to USB `hid`; `ble` is opt-in; no-default builds require a custom transport.

Keep features additive and ensure production imports are explicitly enabled by
the package that uses them.

## Validation

Fast checks:

```bash
cargo test -p ton_core
cargo test -p ton --lib
cargo test -p ton_macros
cargo test -p ton_ledger --all-features
cargo +nightly fmt --check
```

Full public-library checks:

```bash
cargo test --workspace --all-features
cargo test -p ton --doc --all-features
cargo check -p examples --examples --all-features
cargo hack check --rust-version --workspace --all-targets --all-features --ignore-private
cargo +nightly fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features
cargo package --list -p ton
cargo package --list -p ton_core
cargo package --list -p ton_macros
cargo package --list -p ton_ledger
```

Use `cargo semver-checks` against the latest matching release tag when public
surface changes; `ton_ledger` 0.1.0 is an initial release with no prior baseline.
Release automation lives in `.release-plz.toml` and `.github/workflows/release.yml`.
Before publication, verify the package with `cargo publish --dry-run -p <crate>`;
`cargo package --list` alone does not build it. Run live network tests only when requested or when the change
depends on current chain behavior.
