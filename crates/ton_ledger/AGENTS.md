# ton_ledger maintainer guide

Own the exclusive Ledger session, native transports and verified signing here.
Networking/history/broadcasting belong in callers, including the examples crate.
Public paths are module-qualified; no convenience re-exports. The only supported
transport extension is `transports::Transport`, using Send async futures and complete
APDUs with status bytes. Build through `TonLedgerWallet::builder(version)`.

Keep exactly three public root modules: `ton_ledger_wallet`, `transports`, and
`error`. Wallet configuration lives in `ton_ledger_wallet::config`; app, proof
and data modules contain public requests/results. Firmware codecs, derivation
encoding, proof digests, hints and session state belong in private `protocol`.
Transport framing stays private under `transports`. Do not restore legacy root
modules or add aliases/re-exports for the pre-release import paths.

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
cargo check -p examples --example ton_ledger_bluetooth_self_transfer --features ledger-ble
cargo clippy -p ton_ledger --all-targets --all-features -- -D warnings
cargo +nightly fmt --check
```
Also check hid-only and ble-only, Rust 1.94, strict Rustdoc, package contents and
an external consumer. Reuse published ton APIs without changing the ton crate.
Keep wallet assembly private here; compare its bytes against TonWallet vectors.

Hint encoding stays private to this crate. Define ordered field mappings with
`impl_ledger_hint!` in `protocol/payload/hints.rs` using the existing TON message types.
Keep primitive checks and signing-policy handling in `protocol/payload/encoding.rs`;
the private `LedgerSupportedMsg` enum derives TLB and selects each message by its
prefix. Exact-cell validation precedes encoding. Keep comment, DNS and vesting
hint rules on their typed messages. The private NFT adapter preserves standard
zero addresses instead of normalizing them to addr_none. Do not add derives or
traversal to other crates. Ledger hint IDs are distinct from TLB opcodes.
Verify mappings against the independent firmware fixtures and payload-policy tests.

BLE negotiation uses tag 0x08 and the packet size at offset 5, as Ledger's host
transports do. Do not fix the intervening header bytes: newer device SDKs also
encode the size there. Ignore unrelated setup notifications within the existing
connection deadline; preserve errors for truncated or undersized size replies.

Acquire the BLE device lease from the backend peripheral ID before spawning a
connection worker, never from the editable display ID. Keep it in the worker
through disconnect cleanup, including failed or cancelled setup. Duplicate
connections fail with `TransportError::DeviceBusy`; the lease is process-local.
BLE discovery has a separate two-second stop-scan allowance after its scan budget.
Keep proof/data wallet transcript tests alongside the independent codec vectors:
they must check request bytes, signature preimages and dirty-session rejection.

HID also requires a process-local lease keyed by the private `CString` OS path,
never the editable or lossy display ID. Acquire it before spawning the worker
and retain it through failed/cancelled setup, blocking OS calls and handle
destruction. Caller cancellation must not release a worker's ownership early.

BLE leases become non-releasable before the first backend operation. Only a
confirmed successful disconnect permits reuse. Failed, timed-out or cancelled
cleanup must quarantine the backend ID until process restart; do not release
ownership based only on a timeout or a disconnected-state snapshot. Keep cleanup
bounded and test success, failure, timeout and worker cancellation.
