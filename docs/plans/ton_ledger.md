# ton_ledger implementation plan

Status: implemented and locally validated. Hardware/live-transfer acceptance remains manual.
Date: 2026-09-22. Implementation: 2026-09-23.
Repository baseline: `c2f507c90759562c477f1d4e505966c6648b2853`, branch `ton_ledger`.

This document is the implementation contract and ordered work plan for a new
public crate named `ton_ledger`, in `crates/ton_ledger`. Implementation does not authorize running a live transfer. Checkboxes below
track implementation; physical-device validation is reported separately.

## 1. Outcome and scope

Provide a high-level `TonLedgerWallet` with a usage pattern similar to `TonWallet`:
construct a wallet, create an unsigned external-message body, sign it, and wrap
it in an external message suitable for existing TON providers. Construction
uses a builder rather than copying `TonWallet` constructors.

Required capabilities:

- `TonLedgerWallet::builder(WalletVersion)` with defaults and `with_` setters.
- Built-in USB HID transport by default and optional Bluetooth Low Energy.
- User-provided transport through a public trait in `transports.rs`.
- An enum selecting conventional TON account derivation or a custom path.
- Wallet V3R2/V4R2, public-key acquisition, address derivation and confirmation.
- One-message transaction signing, recognized payload hints, explicit opaque
  payload policy, address proofs, legacy Ledger data signing and app settings.
- Reuse of `ton`, `ton_core` through its re-export, and existing TLB derives.
- A runnable Bluetooth example that signs and broadcasts an internal transfer
  of exactly **10,000,000 nanotons (0.01 TON) to the same Ledger wallet**.

Out of scope: V5 signing, multi-message Ledger transactions, key export,
mnemonics, account/balance caches, provider ownership inside `TonLedgerWallet`,
automatic app installation, background device managers, generic signer/plugin
frameworks, and experimental extra-currency signing. Extra currencies require
a separately validated firmware profile; do not silently omit them.

## 2. Evidence and compatibility baseline

Use these pinned sources as the starting references:

- [TypeScript client ce292b1](https://github.com/ton-community/ton-ledger-ts/tree/ce292b1ac136c3b0f973704a9ef1d89f5201e1d3),
  package version `7.4.0-pre.0`.
- [LedgerHQ firmware 849962d](https://github.com/LedgerHQ/app-ton/tree/849962d6378567f5aaf708b116292ccf7b17a97c),
  declaring app version `2.9.1`.
- [Ledger device identifiers](https://developers.ledger.com/docs/device-interaction/dmk-ts/references/identifiers).
- [Ledger transport architecture](https://developers.ledger.com/docs/device-interaction/dmk-ts/integration/how_to/transports).
- [TON proof specification](https://docs.ton.org/applications/ton-connect/how-to/ton-proof).
- [btleplug](https://github.com/deviceplug/btleplug) as a candidate native BLE backend.

Firmware source and independent signing vectors take precedence over stale
tables or TypeScript output. Known source issues to avoid copying:

1. TypeScript locks each APDU rather than the complete signing operation.
2. Payload conversion can change inline/reference layout and therefore hashes.
3. Proof verification does not compare against a locally reconstructed digest.
4. `writeUint48` uses incorrect JavaScript shifts; a timestamp of `1700000000`
   reproduces an out-of-range failure.
5. Signing responses do not receive complete structural validation.

The TON upstream default firmware branch is an older snapshot. The inspected
LedgerHQ version supports all 14 hint IDs; its transaction parser does not
support extra currencies. The TON experimental `ec-pre` branch declares 2.4.0
and does support them. Consequently, `version >= minimum` alone cannot prove
all capabilities. Pin fixture provenance, maintain explicit supported-feature
rules, and report unvalidated firmware as such. Do not claim broad old-version
or device compatibility from one source snapshot.

## 3. Public entities and ownership

| Entity | Responsibility |
| --- | --- |
| `ton_ledger_wallet::TonLedgerWallet` | Own immutable wallet identity and one exclusive device session; expose wallet operations. |
| `ton_ledger_wallet::builder::Builder` | Configure wallet, derivation, transport, policy and timeouts; validate and connect in `build().await`. |
| `transports::Transport` | Public extension point exchanging complete APDU byte sequences. |
| `transports::hid::HidTransport` | Native USB discovery, selection, framing, exchange and connection lifecycle. |
| `transports::ble::BleTransport` | Native BLE discovery, selection, GATT framing, notifications and connection lifecycle. |
| `ton_ledger_wallet::config::DerivationPath` | Select a conventional TON account or explicit custom components. |
| `ton_ledger_wallet::config::SigningPolicy` | Explicit policy for unknown payloads and opaque nested fields. |
| `ton_ledger_wallet::proof::ProofRequest`, `AddressProof` | Proof inputs and locally verified output. |
| `ton_ledger_wallet::data::LedgerDataRequest`, `SignedData` | Legacy Ledger plaintext/app-data inputs and verified output. |
| `ton_ledger_wallet::app::AppInfo`, `AppSettings` | App identity/version and blind-signing/expert-mode settings. |
| `error::{TonLedgerError, TonLedgerResult}`, `TransportError` | Typed domain/session errors and backend failures preserving their source. |

`TonLedgerWallet` is non-generic and owns `Box<dyn Transport>` internally. It is
not `Clone`: a device session must not be duplicated implicitly. Read-only
accessors expose version, address, wallet ID, public key and derivation path;
callers cannot mutate identity independently of the cached public key/address.

`protocol::client::Client`, APDU commands, capability rules, payload hints and prepared
signing state remain private. Normal users do not construct a separate account,
transaction request or prepared transaction entity.

Use `#[non_exhaustive]` for extensible enums, errors and returned records.
Requests with private/non-exhaustive state need a documented construction path.
Fixed TLB wire records may remain exhaustive, following repository guidance.
Use module-qualified paths without additional convenience re-exports.

## 4. Builder contract and defaults

Follow existing `ContractClient` and `MetaLoader` builders: a dedicated
`Builder`, `derive_setters::Setters`, and
`#[setters(prefix = "with_", strip_option)]`. Skip setters for the required
wallet version and custom transport field; write `with_transport` manually to
box the implementation. Keep builder state private and `pub(super)` only where
needed for construction.

`TonLedgerWallet::builder(version)` is infallible and performs no I/O.
`build(self)` is async and fallible. It validates configuration before device
discovery, resolves transport, verifies the TON app/capabilities, obtains the
public key and derives the wallet address/state init. It does not prompt for a
transaction or automatically switch/install the device app.

| Setting | Default / setter |
| --- | --- |
| Version | Required; initially V3R2/V4R2 only. |
| Transport | USB HID; `with_transport(impl Transport + 'static)` overrides it. |
| Path | `DerivationPath::Ton { account: 0, testnet: false }`; `with_derivation_path(...)`. |
| Workchain | `0`; `with_workchain(...)`; initially only `0` and `-1`. |
| Wallet ID | Existing `ton` default; `with_wallet_id(...)`; preserve all 32 wire bits. |
| Signing policy | Reject unknown/opaque payload signing unless explicitly permitted; `with_signing_policy(...)`. |
| Request timeout | 10 seconds; `with_request_timeout(Duration)`. |
| Approval timeout | 180 seconds; `with_approval_timeout(Duration)`. |

Timeout defaults are library policy, not firmware limits. They are positive,
configurable budgets. Discovery has its own bounded timeout. Do not restart a
full timeout on every fragment to create an unbounded operation.

```rust,ignore
let mut wallet = TonLedgerWallet::builder(WalletVersion::V4R2)
    .build()
    .await?;

let transport = BleTransport::connect(device).await?;
let mut wallet = TonLedgerWallet::builder(WalletVersion::V4R2)
    .with_transport(transport)
    .with_derivation_path(DerivationPath::Ton {
        account: 0,
        testnet: true,
    })
    .build()
    .await?;
```

These snippets specify intended usage; executable imports/examples are added
during implementation.

## 5. Derivation path

```rust,ignore
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DerivationPath {
    Ton { account: u32, testnet: bool },
    Custom(Vec<u32>),
}
```

`Default` selects mainnet account zero. The conventional path is
`44'/607'/network'/chain'/account'/0'`, where network is 0/1 and chain is 0/255
for workchain 0/-1. Resolve it from final builder settings during `build()` so
setter order cannot produce a stale path. Wallet version does not alter the
reserved final component. Workchain and friendly address flags are separate
concepts from derivation; a custom path is used exactly as supplied.

`Custom` contains unhardened component indexes, each below `0x80000000`.
Validate prefix, length and component bounds before adding the hardened bit
once. Six components are the conventional scheme, not a universal firmware
requirement. Resolve the SDK path bound against the chosen firmware/build;
never rely on an unchecked one-byte length. Testnet derivation is a key-choice
convention, not cryptographic transaction-network separation.

## 6. Wallet API and exact-message semantics

Mirror the corresponding `TonWallet` method argument order and `TonCell`
input/result types, except that Ledger creation accepts a single internal
message cell rather than a vector:

| Method | Behavior |
| --- | --- |
| `create_ext_in_body(&self, expire_at, seqno, int_msg)` | Synchronous construction using existing wallet TLBs and default send mode 3. |
| `sign_ext_in_body(&mut self, &body)` | Async Ledger signing and verified signature attachment. |
| `create_ext_in_msg_from_body(&self, signed_body, add_state_init)` | Synchronous external message construction. |
| `create_ext_in_msg(&mut self, int_msg, seqno, expire_at, add_state_init)` | Async composition of the preceding operations. |
| `confirm_address(&mut self, options)` | Confirm the bound wallet identity on device; distinguish display flags from raw address. |
| `get_address_proof(&mut self, request, options)` | Request, reconstruct and verify proof. |
| `sign_data(&mut self, request, timestamp)` | Sign the legacy data scheme with explicit timestamp. |
| `app_info`, `settings` | Typed app inspection without exposing APDU details. |

Message creation accepts exactly one internal message as a `TonCell` and validates the supported
wallet version before prompting. Signing additionally validates subwallet ID,
V4 opcode zero, send mode and all fields the firmware reconstructs. A manually
built body can express supported custom send modes through existing wallet TLB
records; do not duplicate the high-level API just to expose modes.

The device reconstructs a constrained internal message: IHR disabled, no source,
not bounced, zero fees/timestamps, supported standard destination, and the
specific state-init/body layout. Reject input that cannot be represented
exactly. In particular, do not normalize a caller's cell silently. Preserve
absent versus present-empty payload semantics; force reference placement only
when constructing a new message under an explicit documented contract.

Before I/O, decode through existing TLB types, fully consume the input, encode
the APDU request, and independently reconstruct the expected unsigned body.
Require hash equality with the supplied body. After device approval, require
the returned hash to match and verify Ed25519 against this wallet's public key.
Only then attach the signature and return a signed body. A reconnect or changed
device must re-establish key identity before signing; no stale-key reuse.

The API returns signed cells, not a misleading on-chain transaction ID. It owns
no network client and does not fetch seqno, estimate fees or broadcast.

## 7. Reuse existing ton APIs

Reuse:

- `TonCell`, `TonHash`, `TonAddress`, `BoC`, checked builders, hash and depth.
- `WalletVersion::get_code`, `WalletV3Data`, `WalletV4Data` and `StateInit`.
- `WalletV3ExtMsgBody`, `WalletV4ExtMsgBody` and the existing message schemas.
- `Msg`, `CommonMsgInfoInt`, `CommonMsgInfoExtIn`, `CurrencyCollection`.
- `JettonTransferMsg`, `JettonBurnMsg`, `NFTTransferMsg` and `SnakeData`.
- `TLBRef`, `TLBEitherRef`, explicit `EitherRefLayout` and `#[derive(TLB)]`.

The current `TonWallet` requires `KeyPair`. Do not create dummy private keys
or modify the ton crate. Build public-key initial data, attach V3/V4 signatures,
and wrap external envelopes privately within ton_ledger using existing public
TON schemas. Compare addresses and message bytes against software wallet vectors.

The existing high-level unsigned-body helper uses mode 3; lower-level Ledger
verification must still use the explicit modes decoded from supplied bodies.

## 8. TLB payload recognition and display hints

Keep two separate representations: actual TON cells and the firmware's compact
byte-oriented hints. TLB handles on-chain records; a small checked byte codec
handles APDUs and hints. A hash/depth hint is not a fabricated pruned cell.

Reuse existing public Jetton/NFT types. Missing recognized on-chain bodies can
initially be crate-private TLB records in `protocol/payload/tlb.rs`; do not add entire
public contract families to `ton` just for Ledger display recognition. New
publicly reusable TON schemas should only be promoted with a concrete consumer
need and the applicable library review.

Encode the eleven fixed-field hint families with a private `impl_ledger_hint!`
macro in `ton_ledger/src/protocol/payload/hints.rs`. Each invocation lists the existing
message type, hint ID and fields in firmware order with named byte adapters.
Adapters in `protocol/payload/encoding.rs` own primitive validation and signing policy.
A private `LedgerSupportedMsg` enum derives TLB and owns opcode dispatch. Parse and
round-trip the complete cell before encoding. Typed comments, DNS records and
vesting messages retain explicit hint rules. A private NFT wrapper preserves
standard zero addresses using the existing NFTTransferMsg fields. No public extension
trait, duplicate TEP structs, or changes to ton/ton_core/ton_macros are needed.
Existing independent fixtures must remain byte-for-byte identical.

Implement all 14 baseline hint families:

| ID | Payload |
| --- | --- |
| 0 | Printable ASCII comment |
| 1 | Jetton transfer |
| 2 | NFT transfer |
| 3 | Jetton burn, including supported inline custom bytes |
| 4 | Whitelist addition |
| 5 | Single-nominator withdrawal |
| 6 | Single-nominator validator change |
| 7 | Tonstakers deposit |
| 8 | Proposal vote |
| 9 | DNS record change |
| 10 | Token-bridge swap payment |
| 11 | TonWhales deposit |
| 12 | TonWhales withdrawal |
| 13 | Vesting message with comment |

Use `ensure_empty = true` on complete new records and explicit parser checks
for reused types. Validate referenced subrecords too. Use custom adapters only
for actual wire exceptions: e.g. an optional trailing app ID without a presence
bit cannot be represented by ordinary `Option<T>` serialization.

Preserve inline/reference distinctions and compare hashes rather than relying
on semantic equality. For a recognized layout that hints cannot reproduce,
reject or use the original cell only under explicit opaque-signing permission.
Never silently rewrite or strip hints. Require separate recognition of opaque
nested fields; a recognized Jetton opcode does not prove full clear display.

Keep firmware token IDs and their address mappings private and source-pinned.
Do not trust caller-supplied ticker/decimals or guess device registry indexes.
Validate all numbers, printable byte constraints, lengths, workchains and
on-chain field widths before any device request.

## 9. Transport contract and built-in implementations

The object-safe async trait belongs directly in `transports.rs`. It exchanges one
complete short APDU and returns the complete response including status bytes.
The intended shape is `exchange(&mut self, command: &[u8], timeout: Duration)`
returning `Result<Vec<u8>, TransportError>`, using the repository's standard
`#[async_trait]` approach. Define `Transport: Send` and keep operation futures
`Send` for native Tokio consumers; do not require `Sync` for an exclusively
owned transport. Backends must uphold these bounds through safe ownership,
using a dedicated worker for a non-movable blocking handle where needed. Do not
use unsafe thread-safety promises or silently switch to local-only futures.

Transport errors retain backend sources and distinguish timeout, disconnect,
permissions and malformed transport frames. App status words belong to the
protocol client. Document exclusive access: custom transports must not allow
other users to interleave commands on the same device during a wallet operation.

USB HID:

- Select a maintained backend after checking license, Rust 1.94 compatibility,
  target support, HID report framing and cancellation behavior.
- Discover Ledger application interfaces and use the published identifiers.
- Default discovery selects exactly one compatible device; return a useful
  no-device/ambiguous-device error otherwise. Explicit selection is available
  through discovery and `HidTransport::connect(device)`.
- Isolate blocking operations from async execution with bounded reads and
  explicit worker ownership/shutdown if a dedicated worker is needed.

Bluetooth:

- Prefer `btleplug` for native OS access after the same dependency preflight.
- Expose bounded discovery returning typed device descriptors and
  `BleTransport::connect(device)`. Do not select by advertised name alone.
- Support the verified normal-mode service/characteristic IDs for the selected
  device models; bootloader availability is not a signing connection.
- Subscribe to notifications, handle Ledger packet-size negotiation, frame and
  reassemble APDUs with checked sequence numbers/lengths and bounded buffers.
- BLE fragmentation and TON APDU chunking are different layers.
- Pairing and permissions use the OS facilities; report actionable errors.
- Own notification tasks and handles; closing/dropping the session terminates
  readers/workers and releases connections without leaking background work.
- Acquire a process-local lease by backend peripheral ID before starting setup;
  hold it through disconnect cleanup and reject duplicate connections with
  `TransportError::DeviceBusy`. Other processes and USB sessions are outside
  this lease's scope.
- Discovery's scan budget excludes a bounded two-second stop-scan allowance;
  document that allowance in the public API and interactive example.

Both backends use the same protocol/signature tests. Mark an in-flight session
dirty before awaiting I/O; timeout, dropped futures or uncertain transport
failure leave it unusable until recovery/reconnection. Do not automatically
retry signing or silently fall back between USB and BLE. Session recovery must
not claim to cancel a device prompt: the TON app has no cancel instruction.

## 10. Protocol, limits, proof and legacy data

- CLA `E0`; version `03`, name `04`, key/address `05`, transaction `06`, proof
  `08`, legacy data `09`, settings `0A`. Check the TON app identity explicitly.
- Parse 32-byte public keys and 98-byte signing data responses strictly:
  `64 || signature[64] || 32 || hash[32]`, plus the separate status word.
- Keep unknown status words in typed errors. Distinguish user denial,
  unsupported command and disabled blind signing.
- Transaction/data chunking: path-only packet `P2=03`, intermediate `02`, final
  `00`, at most 255 data bytes each and 510 accumulated payload bytes for the
  baseline firmware. Lock the whole operation, including preliminary requests.
- Encode Ledger addresses, lengths and variable integers explicitly. APDU
  length prefixes/booleans are not their TLB equivalents. Checked 48-bit writes
  and zero VarUInt encoding need independent vectors.
- Address/proof flags: testnet `01`, masterchain `02`, wallet specifiers `04`.
  A non-display key request uses `P2=0`. Validate response-derived address
  against the bound wallet identity when confirming it.
- Proof uses one APDU. For path length n, wallet-specifier presence s, domain
  bytes d and payload bytes p: `1 + 4n + 5s + 1 + d + 8 + p <= 255`, with
  `d <= 128`, `p <= 128`. Six components plus specifiers allow `d+p <= 216`.
- Reconstruct proof digest locally using raw wallet address, UTF-8 domain,
  timestamp and payload. Timestamp is BE on the APDU but LE in the TON proof
  preimage; domain length is LE and workchain is signed BE. Verify the returned
  hash and signature. Display testnet flags do not bind a proof to a network.
- Legacy plaintext schema `754bf91b`: at most 120 printable ASCII bytes for the
  baseline. App-data schema `54b58535`: address and/or domain required, domain
  at most 126 ASCII bytes, correctly encoded domain labels and data/ext refs.
- Legacy data signatures cover `schema_BE4 || timestamp_BE8 || cell_hash32`.
  Do not label this API as current TON Connect `signData` compatibility.

## 11. USB-first self-transfer example

Add a short, copy-pastable `examples/ton_ledger_self_transfer.rs` executable
with `required-features = ["ledger-ble"]`. Enable both HID and BLE and use the existing pure-Rust LiteClient. Prefer a single
connected USB Ledger; scan Bluetooth only when USB discovery returns no devices.
Report USB discovery/connection errors without falling back; reject multiple USB
devices with instructions to leave only the intended device connected.

```sh
cargo run -p examples --example ton_ledger_self_transfer --features ledger-ble
```

Keep the transfer flow straight-line: select a discovered device, build a V4R2 mainnet account-zero
wallet, confirm its address, read the deployed wallet's seqno, construct a
0.01 TON self-transfer, sign, and broadcast without automatic retries.
Offer a plain transfer by default or an explicit blind-signing test: an unknown
`0xdeadbeef` opcode plus 32 random bytes in a referenced payload, using
`SigningPolicy::AllowOpaque`. Print bytes and cell hash before device approval
and explain that the TON app must enable blind signing.
Keep Bluetooth discovery interactive: list names and IDs, select by number, rescan, or quit.
Explain empty scans and permission failures; print progress before device approval.
Require a funded, deployed wallet. State that fees reduce its balance and
broadcast acknowledgement does not prove inclusion. Omit CLI parsing, history
verification, deployment management and example-specific tests. CI compiles the
example; running it requires explicit live-transfer authorization.

## 12. Dependencies, features and file layout

Add `crates/ton_ledger` as a workspace member and a workspace dependency where
useful. Depend on `ton` with no optional TON networking features enabled by the
library. Core and macro paths resolve through `ton::ton_core`; direct
`ton_core`/`ton_macros` dependencies are unnecessary. This favors reuse over a
minimal dependency tree; `ton` still has unconditional Tokio/HTTP dependencies.

Use existing workspace versions of `derive_setters`, `async-trait`,
`ed25519-dalek`, `sha2`, `thiserror`, and runtime utilities as required. Evaluate
new HID/BLE dependencies for licensing, maintained API, MSRV and platform
requirements before choosing versions. Do not claim no_std or browser/WASM
support as a byproduct of the transport abstraction.

Features: `default = ["hid"]`; independent additive `hid` and `ble`; both can
be enabled together. With neither enabled, custom transports still work and
`build()` without one returns a typed missing-transport error. Unsupported
platforms receive documented feature/build constraints, not stub success.

```text
crates/ton_ledger/
  Cargo.toml
  README.md
  AGENTS.md
  CHANGELOG.md
  src/
    lib.rs
    error.rs
    ton_ledger_wallet.rs       # Public wallet and operation methods.
    ton_ledger_wallet/
      builder.rs
      config.rs               # DerivationPath, SigningPolicy, AddressOptions.
      app.rs                  # AppInfo and AppSettings.
      proof.rs                # ProofRequest and AddressProof.
      data.rs                 # LedgerDataRequest and SignedData.
    transports.rs             # Public Transport trait.
    transports/
      hid.rs
      ble.rs
      framing.rs              # Private transport framing.
    protocol.rs               # Private module; no external protocol API.
    protocol/
      client.rs               # Session state, APDU requests and chunking.
      apdu.rs
      encoding.rs
      derivation_path.rs
      proof.rs
      data.rs
      payload.rs
      payload/                # Exact-cell checks and firmware hints.
    _test_*.rs                # Located beside the responsible modules.
  tests/
    fixtures/                 # Small, source-attributed deterministic vectors.
examples/ton_ledger_self_transfer.rs
```

Record the source licenses for adapted code/fixtures and preserve required
notices. Include README, crate guidance, licenses and needed sources/fixtures
in package rules; do not inherit an unrelated README accidentally. Update the
workspace README, relevant `AGENTS.md`, Ledger Rustdoc/changelog, examples
manifest, release-plz package entry and CI feature coverage together.

## 13. Ordered implementation milestones

- [x] **1. Freeze compatibility and dependency inputs.** Pin source/firmware
  fixtures, resolve SDK path bounds, choose HID/BLE versions and documented
  target support. Preflight native build dependencies and hardware availability.
- [x] **2. Add crate skeleton and transport boundary.** Manifests, features,
  errors, `transports.rs`, module structure and initial crate guidance. Compile a
  custom-transport consumer with default features disabled.
- [x] **3. Implement private Ledger wallet assembly.** Public-key initial data,
  V3/V4 signature attachment and external envelopes reuse existing ton schemas.
  Leave ton unchanged and verify parity against software wallet vectors.
- [x] **4. Implement protocol codec and client.** Strict status/response parsing,
  limits, app inspection, chunk state, timeout and cancellation invalidation.
  Validate with scripted APDU transcripts before adding hardware backends.
- [x] **5. Implement wallet builder and identity.** Defaults, enum path expansion,
  custom transport selection, capability checks, key retrieval, address/state
  init and confirmation. Check builder-order independence.
- [x] **6. Implement transaction and payload signing.** Exact-message validation,
  all 14 hint families, policy, independent hash reconstruction and signature
  verification. Complete the four TonWallet-like message methods.
- [x] **7. Implement address proofs and legacy data.** Independent digest vectors,
  strict lengths and encodings, app-data preimages and verification.
- [x] **8. Implement HID and BLE backends.** Platform dependencies, discovery,
  explicit selection, framing, cancellation and resource lifecycle. Validate
  framing without hardware first, then run bounded device checks where available.
- [x] **9. Add the Bluetooth self-transfer example.** Complete the executable
  behavior above as a small readable sample; compile it without running it.
- [x] **10. Complete public docs, packaging and CI.** Runnable examples, extension
  contract, platform setup, limits, compatibility, changelog, release metadata
  and an external consumer test of the packaged crate.
- [x] **11. Independently review the final implementation.** One focused review
  for public API, wire parity, signing verification, session concurrency and
  lifecycle; resolve findings and rerun affected checks.
- [x] **12. Perform acceptance checks and commit.** Separate local checks,
  emulator evidence and actual hardware/network evidence. Do not push, publish
  or run a live transaction solely because implementation tests passed.

Implementation stays in the current checkout/active branch unless explicitly
changed by the user. Preserve unrelated work. Commit task-owned validated work
under repository Git policy; do not create a separate worktree automatically.

## 14. Validation and completion criteria

Deterministic checks must cover:

- V3R2/V4R2 addresses, custom wallet IDs, workchains and derivation hardening.
- Every hint family; query ID zero; coin/integer maxima and overflow; printable
  byte limits; absent/empty/ref/inline bodies and payloads; trailing data.
- Exact APDU transcripts at 254/255/256-byte and 509/510/511-byte boundaries,
  plus proof budgets for conventional/custom paths and wallet specifiers.
- Independent expected TON cells/hashes, proof digest and legacy data preimage.
- Wrong key, wrong returned hash, invalid signature, malformed length markers,
  truncated/trailing responses, unknown statuses, user rejection and settings.
- Cancellation between chunks, timeout awaiting approval, stale replies,
  disconnect/reconnect, whole-operation exclusivity and worker/stream shutdown.
- HID/BLE framing and malformed/out-of-order fragments independently of TON
  APDU chunking, without merely asserting third-party-library behavior.
- No-default/custom transport, HID-only, BLE-only and combined feature builds.
- Compile the manual example without opening hardware or sending transactions.

Unit-test files use `_test_`; integration-test files may use `test_`. Prefer
`Result` tests with `?`; do not introduce production panic/unwrap control flow.

Start with narrow checks, then complete repository-required validation. Planned
commands (none executed as implementation validation while writing this plan):

```sh
cargo test -p ton_ledger --no-default-features
cargo test -p ton_ledger
cargo test -p ton_ledger --no-default-features --features ble
cargo test -p ton_ledger --all-features
cargo test -p ton --lib
cargo test -p ton_core
cargo test -p ton_macros
cargo check -p examples --example ton_ledger_self_transfer --features ledger-ble
cargo test -p ton_ledger --doc --all-features
cargo test --workspace --all-features
cargo test -p ton --doc --all-features
cargo test -p ton --examples --all-features
cargo hack check --rust-version --workspace --all-targets --all-features --ignore-private
cargo +nightly fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features
cargo package --list -p ton_ledger
cargo package --list -p ton
cargo package --list -p ton_core
cargo package --list -p ton_macros
git diff --check
```

Keep live network/hardware tests excluded from normal runs; inspect and honor
existing integration-test opt-ins before workspace-wide testing. If a required
suite performs live I/O, obtain the necessary execution scope or report that
part separately. Native library/hardware absence is a concrete validation gap,
not evidence that a check passed.

Run `cargo semver-checks` for changed existing public crates against their
latest matching verified release tags; a new crate has no previous SemVer
baseline. Run package build/dry-run and an external extracted-package consumer
once coordinated path dependencies can resolve; report unpublished-dependency
blockers rather than bypassing verification. Compile advertised native targets
in CI, installing HID/BLE system libraries only for enabled backends.

Use emulator/firmware fixtures to validate command compatibility, then physical
USB/BLE checks to establish actual transport support. A testnet self-transfer
requires an available funded device/account and a deliberate manual run. Report
device model, OS, firmware/app versions, transport, network and observed
transaction evidence. Mainnet spending is not an automatic acceptance step.

Completion requires the usable wallet API, both planned transports, all baseline
operations, the working example, docs and applicable checks. If hardware is
unavailable, state that implementation validation is incomplete for hardware
claims; do not describe Bluetooth or a self-transfer as tested.

## 15. Implementation evidence (2026-09-23)

Implemented in the existing `ton_ledger` checkout. The crate uses hidapi 2.6.7
and btleplug 0.13.2, Rust 1.94, default HID and independent BLE/custom features.
All TON app 2.x.x versions are accepted under an assumed SemVer compatibility
policy; other major versions are rejected. The source-validated baseline remains
2.9.1; device identification is not firmware attestation. All 14 hint families have source-attributed independent Python
fixtures, supplemented by explicit zero-varuint, DNS empty-capability and
48-bit timestamp cases. SDK maximum path length is 10 at the commit in NOTICE.

Generic Jetton hints do not infer token registry IDs, tickers or decimals. The
reference Python zero-varuint encoder has a nonminimal-zero bug; see fixture
provenance. Automated implementation validation did not use hardware or live funds. A later
maintainer report on 2026-09-23 confirms successful hardware use; the device,
transport and operations were not specified. USB discovery cannot
forcibly cancel a blocked OS call; asynchronous callers have a deadline and
workers release resources when that call returns. This OS limitation is documented.

Independent review covered protocol reconstruction, signatures, session lifetime,
transports and the complete self-transfer example. Its DNS empty-capability
finding was fixed and covered by an independent fixture; the follow-up found no
remaining P1/P2 issue in the reviewed scope.

The baseline SemVer checkout exporter omitted Git symlinks. Restoring the exact
tracked symlinks in its temporary checkout allowed the ton-v0.4.2 comparison to
pass (196 checks; 57 inapplicable skips). No baseline source was modified.

Original implementation validation (before the review corrections below):

- `cargo test -p ton_ledger --no-default-features`, default HID, BLE-only and
  all-features: each passed 15 tests plus one compiling Rustdoc example.
- The original example and its tests passed before review; review replaces them
  with a compact executable and compilation-only validation.
- `cargo test -p ton --lib`: 120 passed; workspace all-feature unit tests passed
  (141 ton, 121 ton_core, 14 then-current Ledger tests, 6 ton_macros); the final
  added Ledger preflight test passed in all four feature suites.
- `cargo test --workspace --all-features --no-run` compiled every target.
  Live integration tests were not executed, following repository policy.
- `cargo test -p ton --doc --all-features` and `--examples --all-features` passed
  (the existing ton crate has no executable targets in those categories).
- `cargo hack check --rust-version --workspace --all-targets --all-features
  --ignore-private` passed on Rust 1.94; the final Ledger changes were rechecked
  with the same all-target/all-feature MSRV gate.
- Strict workspace Clippy, strict workspace Rustdoc, nightly format and
  `git diff --check` passed; final Ledger Rustdoc was checked again.
- SemVer against the exact ton-v0.4.2 source passed after restoring only the
  baseline export's missing tracked symlinks.
- Package inventories passed for all four libraries. Joint `cargo package`
  built/verified ton_macros, ton_core, ton and ton_ledger. An outside-workspace
  consumer of extracted archives compiled in custom-only, HID and BLE modes,
  including Send-future checks. Pre-commit package runs used `--allow-dirty`;
  clean package verification follows the local task commit.

Review corrections: ton remains unchanged from the preimplementation baseline.
Private Ledger wallet assembly uses the existing ton 0.4 APIs, so publishing
ton_ledger does not require a new ton release. Domain operations use
`error::TonLedgerResult` and `error::TonLedgerError`; dependency requirements are
ton 0.4, hidapi 2 and btleplug 0.13.

Residual acceptance scope: physical USB/BLE pairing, device approval and
cancellation, platform runtime behavior outside macOS, firmware/emulator
execution, and live self-transfer inclusion/receipt were not exercised.

Review correction validation:

- All four Ledger feature configurations passed 15 unit tests and one Rustdoc
  example each. Existing vectors compare V3R2/V4R2 addresses, signatures and
  envelopes with and without state init against TonWallet.
- The 64-line Bluetooth sample compiled; strict Clippy passed for the crate and
  example. Rust 1.94 and strict Ledger Rustdoc checks passed.
- Standalone packaging verified against published ton 0.4.2. No ton source,
  documentation or changelog changes remain relative to the original baseline.
- Independent review found no actionable issue in private wallet assembly,
  builder validation or the error/result rename.

### BLE uncertain-cleanup recovery

A BLE worker marks its lease non-releasable before starting backend I/O. Release
requires a successful disconnect within the cleanup budget. On disconnect error,
timeout or worker cancellation, retain the backend ID in the process registry;
subsequent connections return `DeviceBusy` until process restart. Do not infer
completion from a state snapshot while an old OS disconnect can remain pending.
