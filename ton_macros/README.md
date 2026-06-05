# ton_macros

Automatically derive TLB and TonContract traits for your types

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
