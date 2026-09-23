# Changelog

## [Unreleased]

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
