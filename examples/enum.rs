use std::str::FromStr;
use std::sync::Arc;
use ton_core::TLB;
use ton_core::traits::tlb::TLB;
use ton_core::types::TonAddress;

#[derive(TLB, Eq, PartialEq, Debug)]
#[tlb(prefix = 1, bits_len = 4)]
struct Struct1 {
    value: u32,
}

#[derive(TLB, Eq, PartialEq, Debug)]
#[tlb(prefix = 2, bits_len = 6)]
struct Struct2 {
    value: TonAddress,
}

#[derive(TLB, Eq, PartialEq, Debug)]
#[tlb(prefix = 3, bits_len = 8)]
struct Struct3 {
    value: TonAddress,
}

#[derive(TLB, Eq, PartialEq, Debug)]
#[tlb(prefix = 4, bits_len = 8)]
struct Struct4 {
    value: TonAddress,
}

#[derive(TLB, Eq, PartialEq, Debug)]
#[tlb(prefix = 4, bits_len = 8)]
enum InnerEnum {
    Var1(u8),
    Var2(u16),
    Var3(u32),
}

/// Automatically match underlying variant by prefix (tlb tag)
#[derive(TLB, Eq, PartialEq, Debug)]
enum MyEnum {
    Var1(Struct1),
    Var2(Struct2),
    Var3(Box<Struct3>),
    Var4(Arc<Struct4>),
    Var5(InnerEnum),
}

fn main() -> anyhow::Result<()> {
    let s1 = Struct1 { value: 42 };
    let e1 = MyEnum::Var1(s1);

    let s2 = Struct2 {
        value: TonAddress::from_str("EQBSUY4UWGJFAps0KwHY4tpOGqzU41DZhyrT8OuyAWWtnezy")?,
    };
    let e2 = MyEnum::Var2(s2);

    let e1_boc = e1.to_boc()?;
    let e2_boc = e2.to_boc()?;

    assert_eq!(MyEnum::from_boc(e1_boc)?, e1);
    assert_eq!(MyEnum::from_boc(e2_boc)?, e2);
    Ok(())
}
