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

## Downstream feature map

Paths below are relative to this crate. The `ton_macros` package's `AGENTS.md`
has the full macro option reference. `ton_core` re-exports its derives; import
the `traits::tlb::TLB` trait separately to call `read`, `write`, `from_cell`,
`to_cell`, and the `from_boc*`/`to_boc*` helpers. Use those entry points instead
of calling `read_definition`/`write_definition`, which omit prefix handling.

### Choose the address wire format explicitly

All address adapters are in `ton_core::types::tlb_core::adapters` and can be used
either with `#[tlb(adapter = "...")]` or directly via `.read()`/`.write()`.

| Schema requirement | Rust field / codec | Behavior |
| --- | --- | --- |
| Required standard internal address | `TonAddress` with `InternalAddress` | Always `addr_std`, including internal `0:0`; rejects null, external, variable-length and anycast input; writing checks int8 workchain range. |
| Tolk `address?` (nullable standard internal address) | `Option<TonAddress>` with `NullableAddress` | `None` is `addr_none`; `Some(0:0)` stays internal; no extra Maybe bit. Present values follow `InternalAddress`. |
| Existing default address convention | Plain `TonAddress` | Zero serializes as `addr_none`; do not use when the schema requires internal zero. |
| TL-B `Maybe X` | Plain `Option<T>` | Adds a presence bit before `T`; `Option<TonAddress>` is therefore not Tolk `address?`. |
| Preserve raw address variants | `types::tlb_core::{MsgAddress, MsgAddressInt, MsgAddressExt}` | Use the wire records when the schema needs forms not supported by the standard-address adapters. |

See `src/types/tlb_core/adapters/{internal_address,nullable_address}.rs` and
`src/tests/test_{internal_address,nullable_address}.rs` for implementation and
wire fixtures. Adapters consume only their field; the enclosing record owns
complete-consumption checks.

### Derive before implementing a codec manually

This example combines an opcode, a five-bit value, both address adapters, and
strict parsing of a whole cell. The opcode is illustrative, not a standard TON
message opcode.

```rust
use ton_core::TLB;
use ton_core::traits::tlb::TLB;
use ton_core::types::TonAddress;
use ton_core::types::tlb_core::adapters::{ConstLen, InternalAddress, NullableAddress};

#[derive(Debug, PartialEq, TLB)]
#[tlb(prefix = 0xa1, bits_len = 8, ensure_empty = true)]
struct RoutingRecord {
    #[tlb(bits_len = 5)]
    flags: u8,
    #[tlb(adapter = "InternalAddress")]
    recipient: TonAddress,
    #[tlb(adapter = "NullableAddress")]
    response_address: Option<TonAddress>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let record = RoutingRecord {
        flags: 3,
        recipient: TonAddress::ZERO,
        response_address: None,
    };
    let cell = record.to_cell()?;
    assert_eq!(cell.data_len_bits(), 8 + 5 + 267 + 2);
    assert_eq!(RoutingRecord::from_cell(&cell)?, record);
    Ok(())
}
```

Field `bits_len` expands to `ConstLen::<FieldType>::new(n)`, so keep `ConstLen`
in scope. Header `bits_len` specifies the **prefix** width, not the record size.
`ensure_empty` defaults to false and currently applies to derived structs only.
It rejects remaining bits and references in the current parser; use it on a
whole-cell record, not an inline record followed by other fields. It does not
recursively validate referenced cells. For a strict boundary around another
type, call `T::read(&mut parser)?` followed by `parser.ensure_empty()?`.

### Other codecs and helpers to reuse

The following wire types live in `ton_core::types::tlb_core` (source under
`src/types/tlb_core/`):

| Wire need | Existing type | Important distinction |
| --- | --- | --- |
| `^T` | `TLBRef<T>` | Stores a reference when embedded as a field. Its own `to_cell()` returns the inner cell, not an extra wrapper cell. Child parsing follows `T`'s strictness. |
| `Either L R` | `TLBEither<L, R>` | Writes a selector bit and the chosen inline value. |
| `Either T ^T` | `TLBEitherRef<T>` | `new` chooses layout from available space; `new_ref` forces a reference; `new_with_layout` controls layout explicitly. Layout affects hashes. |
| Length-prefixed integer/bytes | `VarLenBits<T, N>`, `VarLenBytes<T, N>` | `N` is the width of the length prefix. The constructor's payload length is always in bits, including for `VarLenBytes`; byte lengths must be aligned. |
| TON coin wire amount | `TLBCoins` | VarUInteger-16 encoding; use integer base units (nanotons for TON), not floating point TON values. `types::Coins` adds checked arithmetic. |
| Unary length | `adapters::UnaryLen` | Reuse for unary encoding, not a fixed-width integer. |
| Opaque remaining payload | `cell::TonCell` | As an inline TLB field it consumes all remaining bits and refs; normally put it last. Use `TLBRef<TonCell>` for a separate child. |

Use `TonCell::builder()`, `CellParser`, `TonCell::slice(CellBorders)`, and
`BoC::from_bytes` for lower-level work; do not recreate cell/BOC framing.
Use `traits::state_provider::StateProvider` and
`traits::emulation_provider::EmulationProvider` as provider extension points.

With feature `serde`, `ton_core::serde` supplies `#[serde(with = "...")]`
modules for hash hex/base64, address hex/base64, optional values, and transaction
IDs. See `src/serde.rs`; JSON helpers do not change TL-B address encoding.
