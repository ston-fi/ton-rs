# ton_ledger maintainer guide

## Scope and navigation

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

## Signing and lifecycle invariants

Preserve exact TON cells: compare local reconstruction and device hashes and
verify Ed25519 before returning a signature. Never normalize caller layout,
omit extra currencies, guess token identities, or infer nested clear display.
Fixed private TLB records are exhaustive wire schemas. Extensible public enums,
errors and returned records are non-exhaustive with construction paths as needed.
A session must remain dirty across the entire chunk operation and after dropped
futures/timeouts. No automatic retry or reconnection. Backends own and close their
workers/streams; they cannot promise to cancel the device's approval screen.

Use fixtures under tests/fixtures, pinned firmware source, README and the plan
in docs/plans/ton_ledger.md. Accept all TON app 2.x.x versions under the assumed
SemVer compatibility policy; reject other majors. The source-validated baseline
remains 2.9.1. Changes to wire encoding or signing rules require new independent
vectors; accepting a version does not establish validation of that release.
Update docs, fixtures, feature matrix, package contents and changelog together. Never invoke the funded example
without explicit live-transfer authorization.

## Validation

Fast checks:
```
cargo test -p ton_ledger --no-default-features
cargo test -p ton_ledger --all-features
cargo check -p examples --example ton_ledger_self_transfer --features ledger-ble
cargo clippy -p ton_ledger --all-targets --all-features -- -D warnings
cargo +nightly fmt --check
```
Before release, also run:

```sh
cargo test -p ton_ledger
cargo test -p ton_ledger --no-default-features --features ble
cargo +1.94 hack check -p ton_ledger --feature-powerset --all-targets
RUSTDOCFLAGS="-D warnings -D missing_docs" cargo doc -p ton_ledger --no-deps --all-features
cargo package --list -p ton_ledger
cargo publish --dry-run -p ton_ledger
```

Compile a temporary consumer of the extracted `.crate` with default, no-default,
BLE-only and combined features. Version `0.1.0` is the initial release; there is
no published baseline for a SemVer comparison. Release-plz owns subsequent
version/changelog updates through the workspace `.release-plz.toml`.

The crate enables `missing_docs`: document public types, variants, fields,
methods and generated setters. Explain units, defaults, limits, I/O, approval,
errors and recovery where meaningful. Private comments should explain invariants
and non-obvious ownership rather than narrate the code.

A maintainer reported a working physical-device test on 2026-09-23; model,
transport and operation details were not recorded. Do not turn that report into
blanket device/platform/recovery claims. Record model, OS, firmware/app version,
transport, tested operations and observed outcome for future hardware acceptance.

Reuse published ton APIs without changing the ton crate.
Keep wallet assembly private here; compare its bytes against TonWallet vectors.

## Code map and extension points

- `ton_ledger_wallet.rs`: wallet identity, address confirmation and signing entrypoints.
- `ton_ledger_wallet/{builder,config,app,proof,data}.rs`: configuration and public records.
- `protocol/client.rs`: APDU session state, chunking and verification.
- `protocol/{apdu,derivation_path,proof,data}.rs`: wire encoders and digest construction.
- `protocol/payload.rs` and `protocol/payload/`: exact-cell validation and display hints.
- `transports/{hid,ble,framing}.rs`: worker ownership, native backends and packet framing.
- `_test_*.rs` beside these modules: deterministic regression tests.
- `tests/fixtures/README.md`: fixture generation and pinned-source provenance.

The full design is `docs/plans/ton_ledger.md` at the repository root (not included
in the published package). Keep package-local guidance sufficient on its own.

Hint encoding stays private to this crate. Define ordered field mappings with
`impl_ledger_hint!` in `protocol/payload/hints.rs` using the existing TON message types.
Keep primitive checks and signing-policy handling in `protocol/payload/encoding.rs`;
the private `LedgerSupportedMsg` enum derives TLB and selects each message by its
prefix. Exact-cell validation precedes encoding. Keep comment, DNS and vesting
hint rules on their typed messages. The private NFT adapter preserves standard
zero addresses instead of normalizing them to addr_none. Do not add derives or
traversal to other crates. Ledger hint IDs are distinct from TLB opcodes.
Verify mappings against the independent firmware fixtures and payload-policy tests.

## Transport maintenance

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
