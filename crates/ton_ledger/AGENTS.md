# ton_ledger maintainer guide

Own the exclusive Ledger session, native transports and verified signing here.
Networking/history/broadcasting belong in callers, including the examples crate.
Public paths are module-qualified; no convenience re-exports. The only supported
transport extension is `traits::Transport`, using Send async futures and complete
APDUs with status bytes. Build through `TonLedgerWallet::builder(version)`.

Preserve exact TON cells: compare local reconstruction and device hashes and
verify Ed25519 before returning a signature. Never normalize caller layout,
omit extra currencies, guess token identities, or infer nested clear display.
Fixed private TLB records are exhaustive wire schemas. Extensible public enums,
errors and returned records are non-exhaustive with construction paths as needed.
A session must remain dirty across the entire chunk operation and after dropped
futures/timeouts. No automatic retry or reconnection. Backends own and close their
workers/streams; they cannot promise to cancel the device's approval screen.

Use fixtures under tests/fixtures, pinned firmware source, README and the plan
in docs/plans/ton_ledger.md. Changing firmware rules requires new independent
vectors; version numbers alone are insufficient. Update docs, fixtures, feature
matrix, package contents and changelog together. Never invoke the funded example
without explicit live-transfer authorization.

Fast checks:
```
cargo test -p ton_ledger --no-default-features
cargo test -p ton_ledger --all-features
cargo check -p examples --example ledger_bluetooth_self_transfer --features ledger-ble
cargo clippy -p ton_ledger --all-targets --all-features -- -D warnings
cargo +nightly fmt --check
```
Also check hid-only and ble-only, Rust 1.94, strict Rustdoc, package contents and
an external consumer. Reuse published ton APIs without changing the ton crate.
Keep wallet assembly private here; compare its bytes against TonWallet vectors.

Hint encoding stays private to this crate. Define ordered field mappings with
`impl_ledger_hint!` in `payload/hints.rs` using the existing TON message types.
Keep primitive checks and signing-policy handling in `payload/encoding.rs`;
recognition and exact-cell validation precede encoding. Comment, DNS and vesting
formats retain explicit encoders. Do not add derives or traversal to other crates.
Use each message type's `TLB::PREFIX` for dispatch; Ledger hint IDs are separate
firmware values and must not be substituted with TLB opcodes.
Verify mappings against the independent firmware fixtures and payload-policy tests.
