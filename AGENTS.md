# ton-rs agent guide

## Repository

This workspace contains public Rust libraries for TON:

- `ton_core`: cells, addresses, TLB primitives, and provider traits.
- `ton`: block and contract types, wallets, clients, and optional emulators.
- `ton_macros`: derives and attributes used by the two libraries.
- `examples` and `benchmarks`: consumers, not reusable API crates.

Use the `rust-library-review` workflow for public API, dependency, feature,
serialization, documentation, or release changes.

## Boundaries

Keep provider-neutral types in `ton_core`. Networking, contract wrappers, wallet
logic, and native tonlib adapters belong in `ton`. Procedural macros must emit
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

Keep features additive and ensure production imports are explicitly enabled by
the package that uses them.

## Validation

Fast checks:

```bash
cargo test -p ton_core
cargo test -p ton --lib
cargo test -p ton_macros
cargo +nightly fmt --check
```

Full public-library checks:

```bash
cargo test --workspace --all-features
cargo test -p ton --doc --all-features
cargo test -p ton --examples --all-features
cargo +nightly fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features
cargo package --list -p ton
cargo package --list -p ton_core
cargo package --list -p ton_macros
```

Use `cargo semver-checks` against the latest matching release tag when public
surface changes. Run live network tests only when requested or when the change
depends on current chain behavior.
