# `ton_core` crate guide

`ton_core` is the public, provider-neutral foundation for cells, hashes,
addresses, TLB serialization, and contract state/emulation traits. It must not
depend on network clients, Tokio runtime behavior, or native tonlib.

Use `rust-library-review` for every non-trivial change.

Preserve cell limits, TLB prefixes, field order, reference layout, address
encoding, and BOC compatibility. Prefer `TonCoreError` and checked builders over
panics.

Open errors, provider requests/results, and returned state should be
`#[non_exhaustive]`. Fixed algebraic helpers and wire records may remain
exhaustive when their complete shape is the protocol contract. Non-exhaustive
structs require a public construction path.

Provider implementations must return exact requested transaction state and
stable cache-invalidation deltas. `EmulationProvider` owns execution; it declares
whether address-based state must be resolved by the caller.

Downstream setup:

```toml
[dependencies]
ton_core = "0.3"
```

Enable `serde` only when serialization support is needed. Start validation with
`cargo test -p ton_core`; finish with the root full checks and package listing.
