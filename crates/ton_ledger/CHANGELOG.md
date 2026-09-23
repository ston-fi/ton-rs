# Changelog

## [Unreleased]

- Change `create_ext_in_body` and `create_ext_in_msg` to accept one `TonCell`
  instead of `Vec<TonCell>`. Migrate calls from `vec![message]` to `message`;
  Ledger signing continues to reject caller-built bodies with zero or multiple messages.

- Accept all TON app 2.x.x versions under an assumed SemVer compatibility policy;
  retain 2.9.1 as the source-validated protocol baseline and reject other majors.

- Add an optional random opaque payload to the mainnet self-transfer example
  for manual blind-signing checks.

- Document public API contracts, configuration defaults and recovery behavior;
  enable missing-documentation warnings and expand maintainer navigation.

- Quarantine BLE devices after failed, timed-out or cancelled disconnect cleanup.
  Reuse requires a confirmed disconnect; uncertain cleanup requires process restart.

- Reject duplicate in-process USB HID connections by backend path, retaining
  ownership through cancelled setup and device-handle cleanup.

- Switch the manual self-transfer example to mainnet and separate wallet
  connection and client setup from the signing and broadcasting flow.

- Prefer USB in the manual self-transfer example, falling back to Bluetooth
  scanning when no USB Ledger is connected.

- Reorganize the pre-release API into `ton_ledger_wallet`, `transports` and
  `error`. Move `DerivationPath`, `SigningPolicy` and `AddressOptions` to
  `ton_ledger_wallet::config`, app/proof/data types to the corresponding wallet
  submodules, and `Transport` to `transports::Transport`. Remove the old root
  modules without compatibility aliases. Firmware/session code is private
  under `protocol`; signing behavior and wire formats are unchanged.

- Reject duplicate in-process BLE sessions with `TransportError::DeviceBusy`;
  retain ownership until the connection worker finishes cleanup.
- Clarify BLE discovery cleanup time and document a complete transfer example.
- Exercise proof and legacy-data signing through the wallet, including rejected
  hashes/signatures and unusable-session behavior.
- Restore the Bluetooth example's explicit testnet selection and display flags.

- Fix BLE packet-size negotiation rejecting newer Ledger SDK reply headers;
  wait for the negotiation tag within the existing connection timeout.

- Rename the private payload enum to `LedgerSupportedMsg`; make the Bluetooth
  example interactive with device selection, rescanning and actionable errors.

- Parse supported Ledger payloads through a private TLB enum; give DNS records
  and NFT address preservation explicit types instead of manual opcode dispatch.

- Use private field-mapping macros for Ledger hints without changing TON types
  or the wallet API; preserve the firmware encoding and signing policies.

- Add V3R2/V4R2 Ledger wallet builder, exact-cell transaction signing, 14 payload
  hint families, address proofs, legacy data signing and app inspection.
- Add typed `error::TonLedgerError` and `error::TonLedgerResult<T>`.
- Reuse existing ton 0.4 APIs with private Ledger wallet assembly.
- Add exclusive custom transport, default USB HID and optional native BLE.
- Add deterministic protocol/firmware vectors and manual Bluetooth self-transfer.
- Firmware profile is source-pinned to TON app 2.9.1; hardware acceptance pending.
