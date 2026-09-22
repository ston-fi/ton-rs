# Changelog

## [Unreleased]

- Use private field-mapping macros for Ledger hints without changing TON types
  or the wallet API; preserve the firmware encoding and signing policies.

- Add V3R2/V4R2 Ledger wallet builder, exact-cell transaction signing, 14 payload
  hint families, address proofs, legacy data signing and app inspection.
- Add typed `error::TonLedgerError` and `error::TonLedgerResult<T>`.
- Reuse existing ton 0.4 APIs with private Ledger wallet assembly.
- Add exclusive custom transport, default USB HID and optional native BLE.
- Add deterministic protocol/firmware vectors and manual Bluetooth self-transfer.
- Firmware profile is source-pinned to TON app 2.9.1; hardware acceptance pending.
