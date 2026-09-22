# Independent fixture provenance

`payloads.tsv`, `transactions.tsv` and `data.tsv` were generated with the Python
codec in LedgerHQ/app-ton commit 849962d6378567f5aaf708b116292ccf7b17a97c,
`tests/application_client/ton_transaction.py` and `ton_sign_data.py`, using
`tonsdk==1.0.15`. They do not execute firmware and are not hardware evidence.
The C parser and hash reconstruction were inspected separately. The source is
Apache-2.0 (Ledger SAS / Whales Corp); package NOTICE preserves attribution.

Regenerate in a temporary Python environment with that source checkout:
```
pip install tonsdk==1.0.15
PYTHONPATH=/path/to/pinned/app-ton/tests python generate.py
```
Payload columns: hint ID, BOC, hint bytes, cell hash. Transaction columns: V4
flag, unsigned wallet-body BOC, transaction APDU payload, unsigned-body hash.
Data columns: APDU payload, directly signed preimage. The proof vector in
`_test_protocol.rs` uses independent Python hashlib and struct encoding.

The upstream Python `write_varuint(0)` encodes `01 00`, which does not match
its canonical TON-cell encoding. These fixtures use nonzero forwarded amounts;
Rust's independent zero vector explicitly requires `00`, following the firmware
`BitString_storeCoinsBuf` length contract. This upstream bug is not copied.
