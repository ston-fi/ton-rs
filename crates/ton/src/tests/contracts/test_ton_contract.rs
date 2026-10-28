use ton_macros::TLB;

#[test]
#[allow(unused)] // we just check it compiles
fn test_ton_contract_macro() {
    ton_contract!(MyContract1);

    trait MyTrait1 {}
    ton_contract!(MyContract2: MyTrait1);

    trait MyTrait2 {}
    ton_contract!(MyContract3: MyTrait1, MyTrait2);

    #[derive(TLB)]
    pub struct MyContract4Data;
    ton_contract!(MyContract4<MyContract4Data>);

    #[derive(TLB)]
    pub struct MyContract5Data;
    ton_contract!(MyContract5<MyContract5Data>: MyTrait1, MyTrait2);
}
