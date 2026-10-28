# `ton_macros` crate guide

`ton_macros` is the public procedural-macro crate used by `ton_core` and `ton`.
It provides `TLB`, `FromTVMStack`, and `ton_methods`.

## Downstream macro reference

Read this before hand-writing serialization, TVM stack extraction, or get-method
boilerplate. Paths below are relative to this package. `ton_core` re-exports
these macros; importing `ton_core::TLB` imports the derive, while
`ton_core::traits::tlb::TLB` imports the serialization trait. With only a `ton`
dependency, the existing `ton::ton_core` path exposes both. Generated TLB paths
resolve renamed dependencies, but adapter expressions still use caller scope.

### `#[derive(TLB)]`

| Placement | Attribute | Meaning |
| --- | --- | --- |
| Struct or enum | `#[tlb(prefix = 0xa1, bits_len = 8)]` | Encode/check an eight-bit prefix; both values can be expressions. Set both for a nonempty prefix. Without them the prefix has zero bits. |
| Struct field | `#[tlb(bits_len = 24)]` | Fixed-width value, an alias for `ConstLen::<FieldType>::new(24)`. Import `ton_core::types::tlb_core::adapters::ConstLen`. The shorthand takes a numeric literal; use an adapter expression for a computed width. |
| Struct field | `#[tlb(adapter = "InternalAddress")]` | Use the adapter's `read`/`write` methods. Import the adapter or put its full path in the string. Expressions such as `ConstLen::<u32>::new(WIDTH)` are supported. |
| Struct | `#[tlb(ensure_empty = true)]` | After reading fields, reject leftover bits **and references in the current cell**. Default is false. Works for named, tuple, and unit structs. |

Do not combine `bits_len` and `adapter` on one field: the current expansion
replaces the supplied adapter with `ConstLen`. `ConstLen` supports primitive,
fastnum and big integers, `Vec<u8>`, and their optional forms; on `Option<T>`,
the width applies to the present value after a separate Maybe bit. It does not
describe a whole record's length or arbitrary nested type.

Choose strictness at the cell boundary. A strict struct embedded inline before
more fields rejects those fields as leftovers. An outer `ensure_empty` check
does not inspect tails inside referenced cells; their contained types must own
that validation. `TLB::from_cell` and `from_boc*` do not add strictness themselves.
**The enum derive currently does not apply `ensure_empty`**, even if specified.
For a strict enum boundary, read the enum with `TLB::read` and then call
`parser.ensure_empty()`, or wrap it in a strict whole-cell struct.

Enums require exactly one unnamed payload per variant and dispatch on payload
TLB prefixes; an enum's own prefix is additional. Use distinct, unambiguous
payload prefixes. Zero-length prefixes can shadow later variants in fallback
declaration order. The derive also supplies `From<Payload>`, `as_<variant>()`,
`as_<variant>_mut()` and `into_<variant>()` helpers (snake_case variant names).
Check `src/tlb_derive_enum.rs` and the resolved
`ton_core` package's `src/tests/traits/tlb/test_tlb_enum.rs` before composing enums.

Sources of truth: `src/tlb_derive.rs` (accepted attributes),
`src/tlb_derive_struct.rs` (field adapters/strictness), and
`src/tlb_derive_enum.rs` (dispatch). The resolved `ton_core` package's `AGENTS.md`
has a complete address/fixed-width recipe; its `src/types/tlb_core/adapters/`
contains reusable adapters. Dictionary adapters live in `ton::tlb_adapters`.

### `#[derive(FromTVMStack)]`

Requires a `ton` dependency and a named-field struct. Fields are extracted in
reverse declaration order because `TVMStack` pops from the end; declare fields
in the get-method's logical result order, not pop order.

- `#[from_tvm_stack(ensure_empty = true)]` rejects remaining stack values after
  extraction. Use for a complete result with a known shape.
- `#[from_tvm_stack(allow_extra = true)]` first discards top values until the
  stack length equals the number of struct fields. Use only for a deliberately
  compatible result shape; it assumes one stack item per field. It can be
  combined with `ensure_empty`, but discarded values are then not rejected.
- Both flags default to false. Do not treat `allow_extra` as validation or use
  it to hide a mismatched ABI.

```rust
use ton::block_tlb::{FromTVMStack, TVMStack};
use ton::ton_core::FromTVMStack;

#[derive(Debug, PartialEq, FromTVMStack)]
#[from_tvm_stack(ensure_empty = true)]
struct Counters {
    sent: u32,
    received: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stack = TVMStack::default();
    stack.push_tiny_int(7);
    stack.push_tiny_int(9);
    assert_eq!(Counters::from_stack(&mut stack)?, Counters { sent: 7, received: 9 });
    Ok(())
}
```

See `src/from_tvm_stack_derive.rs` and the `ton` package's
`src/block_tlb/tvm_types/from_tvm_stack.rs` for supported field conversions.

### `#[ton_methods]` and `ton_contract!`

`ton_methods` generates bodyless async get methods in a trait or impl, packs
arguments with `ton::block_tlb::ToTVMStack`, and decodes the return value with
`FromTVMStack`. Default method names are unchanged. Use
`#[ton_methods(name_format = "camelCase")]` for a shared naming convention and
`#[ton_method(name = "getUIVariables")]` for an exact per-method override (which
wins over the format). Use `#[ton_method(id = 109505)]` for an explicit numeric
selector without hashing a name. The ID is an i32 integer literal (decimal or hex,
optionally negative); it bypasses `name_format`. Specify exactly one of `name` or
`id`. The override belongs only on a generated bodyless method.

`ton::ton_contract!` is a separate declarative macro owned by `ton`; it supplies
wrapper state and the `TonContract` implementation. For example:

```rust
use ton::contracts::TonContract;
use ton::errors::TonResult;
use ton::ton_contract;
use ton::ton_core::ton_methods;

#[async_trait::async_trait]
#[ton_methods(name_format = "camelCase")]
pub trait CounterMethods: TonContract {
    async fn get_counter(&self) -> TonResult<u32>;
}

ton_contract!(CounterContract: CounterMethods);
```

This recipe also needs a direct `async-trait` dependency. For typed contract
storage use `ton_contract!(CounterContract<Storage>: CounterMethods)` with a
`Storage: TLB` type; omit it to use `TonCell`. Import `TonContract` to call its
`new`, `load_state`, and `load_parsed_data` methods. See `README.md`,
`src/ton_methods.rs`, and `ton`'s `src/contracts/ton_contract.rs` for more recipes.

## Maintaining macros

Use `rust-library-review` for public macro syntax or generated API changes.
Generated paths must resolve when crates are renamed. Preserve existing
attributes and diagnostics unless a breaking change is documented. Keep parsing
strict by default; tolerance such as `allow_extra` must remain opt-in.

Add focused expansion/behavior tests for macro-owned rules. Validate with
`cargo test -p ton_macros`, downstream `ton_core`/`ton` tests, strict Rustdoc and
Clippy, and `cargo package --list -p ton_macros`.
