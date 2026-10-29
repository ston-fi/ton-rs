# ton_macros

Procedural macros for `TLB`, `FromTVMStack`, and async get-method implementations
with `#[ton_methods]`. The `ton_contract!` wrapper macro belongs to the `ton` crate.

## TLB Derive

```rust
use ton_macros::TLB;

#[derive(Debug, Clone, PartialEq, TLB)]
#[tlb(prefix = 0xc4, bits_len = 8)]
pub struct GlobalVersion {
    pub version: u32,
    pub capabilities: u64,
}

// specify custom adapter (ser/de functions for TLB)
#[derive(Debug, Clone, PartialEq, TLB)]
pub struct StateInit {
    #[tlb(bits_len = 5)]
    pub split_depth: Option<u8>,
    pub tick_tock: Option<TickTock>,
    pub code: Option<TLBRef<TonCell>>,
    pub data: Option<TLBRef<TonCell>>,
    #[tlb(adapter = "TLBHashMapE::<DictKeyAdapterTonHash, DictValAdapterTLB<_>>::new(256)")]
    pub library: HashMap<TonHash, SimpleLib>,
}
```

## TonContract and ton_methods

```rust
use ton::contracts::TonContract;
use ton::errors::TonResult;
use ton::ton_contract;
use ton_macros::ton_methods;

#[async_trait::async_trait]
#[ton_methods]
pub trait JettonMasterMethods: TonContract {
    async fn get_jetton_data(&self) -> TonResult<u32>;
}

ton_contract!(JettonMaster: JettonMasterMethods);
```

`#[ton_methods]` generates default get-method implementations for traits or
inherent impl blocks. By default, it passes the Rust function name to
`emulate_get_method` unchanged.

Use `name_format` to convert Rust method names before emulation:

```rust
use ton::contracts::TonContract;
use ton::errors::TonResult;
use ton::ton_contract;
use ton_macros::ton_methods;

#[async_trait::async_trait]
#[ton_methods(name_format = "camelCase")]
pub trait OrderContractMethods: TonContract {
    // Emulates getOrderData.
    async fn get_order_data(&self) -> TonResult<u32>;
}

ton_contract!(OrderContract: OrderContractMethods);
```

Supported format names are based on
[`convert_case::Case`](https://docs.rs/convert_case/0.11.0/convert_case/enum.Case.html).
Common values include `snake_case`, `camelCase`, `PascalCase`, `CamelCase`,
`kebab-case`, and `CONSTANT_CASE`.

Use `#[ton_method(name = "...")]` when one method does not follow the enclosing
format. The exact name takes precedence over `name_format`:

```rust
#[async_trait::async_trait]
#[ton_methods(name_format = "camelCase")]
pub trait OrderContractMethods: TonContract {
    #[ton_method(name = "getUIVariables")]
    async fn get_ui_variables(&self) -> TonResult<u32>;
}
```


For an export with a known numeric selector, use `id` instead of `name`:

```rust,ignore
#[ton_methods]
impl Contract {
    #[ton_method(id = 109505)]
    async fn estimate_gas(&self, kind: u8) -> TonResult<u32>;
}
```

The signed i32 literal is passed directly to emulation, bypassing name formatting
and hashing. Decimal and hexadecimal literals are accepted. `name` and `id` are
mutually exclusive and apply only to generated bodyless methods.

## FromTVMStack field adapters

Fields normally use their native `FromTVMStack` implementation. Select a TLB
adapter when a stack Cell/Slice contains a specific wire representation:

```rust,ignore
use ton::ton_core::types::{TonAddress, tlb_core::adapters::{InternalAddress, NullableAddress}};

#[derive(ton::ton_core::FromTVMStack)]
#[from_tvm_stack(ensure_empty = true)]
struct Owners {
    #[from_tvm_stack(adapter = InternalAddress)]
    owner: TonAddress,
    #[from_tvm_stack(adapter = NullableAddress)]
    pending_owner: Option<TonAddress>,
}
```

The adapter expression can also be quoted, for example `adapter = "InternalAddress"`,
or configured, for example `adapter = ConstLen::<u32>::new(24)`. It uses caller
scope and must provide `read(&mut CellParser)` returning the field's type.
Each adapted field consumes one Cell/Slice; TVM NULL and other stack kinds fail.
`NullableAddress` accepts encoded addr_none while preserving Some(internal zero).
Cell tail handling follows the adapter; exact stack arity remains controlled by
`ensure_empty`. Unannotated fields and method arguments retain their existing
behavior. For internal-address arguments, native `MsgAddressInt: ToTVMStack`
lets `ton_methods` generate ordinary signatures without parameter attributes.
