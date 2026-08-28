# Wallet module guide

This module derives wallet addresses, encodes wallet data and external request
bodies, and signs them with Ed25519 keys.

Each wallet contract has its own code, initial-data layout, request layout,
signature position, limits, and default subwallet ID. Treat all of these as wire
contracts. Do not model a wallet as `WalletVersion` unless one immutable version
fully describes its code, initial data, address derivation, and signing behavior.

`TonWallet` is constructed through `new`, `new_with_creds`, or
`new_with_params`; its non-exhaustive fields are readable state, not a preferred
literal-construction API. New wallet support must update code resources, data and
message TLB types, version dispatch, address/signature vectors, README guidance,
and package contents together.

Tests must prove upstream-compatible address derivation and signed BOC layout.
Use external-chain fixtures only when a deterministic vector cannot prove the
contract.
