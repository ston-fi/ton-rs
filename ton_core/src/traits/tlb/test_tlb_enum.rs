use crate::traits::tlb::TLB;
use crate::types::TonAddress;
use std::str::FromStr;
use std::sync::Arc;
use ton_macros::TLB;

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
struct Struct3;

#[derive(TLB, Eq, PartialEq, Debug)]
#[tlb(prefix = 4, bits_len = 8)]
struct Struct4;

/// Automatically match underlying variant by prefix (tlb tag)
#[derive(TLB, Eq, PartialEq, Debug)]
enum MyEnum {
    Var1(Struct1),
    Var2(Struct2),
    Var3(Box<Struct3>),
    Var4(Arc<Struct4>),
    Var5(Box<MyEnum>),
}

#[test]
fn test_tlb_enum_tlb() -> anyhow::Result<()> {
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

#[test]
fn test_tlb_enum_from() -> anyhow::Result<()> {
    // just check it works
    let s1 = Struct1 { value: 42 };
    let e1: MyEnum = s1.into();

    let _e3: MyEnum = Struct3.into();
    let _e3: MyEnum = Box::new(Struct3).into();

    let _e4: MyEnum = Struct4.into();
    let _e4: MyEnum = Arc::new(Struct4).into();

    let _e5: MyEnum = Box::new(e1).into();
    Ok(())
}
