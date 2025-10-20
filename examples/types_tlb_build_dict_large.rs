use num_bigint::BigUint;
use std::collections::HashMap;
use ton::tlb_adapters::DictKeyAdapterUint;
use ton::tlb_adapters::DictValAdapterNum;
use ton::tlb_adapters::TLBHashMap;
use ton_core::traits::tlb::TLB;
use ton_core::TLB;

extern crate num_bigint;
extern crate ton;
extern crate tonlib_core;

// const ITEMS_COUNT: usize = 40000000;
const ITEMS_COUNT: usize = 400000;

#[derive(TLB)]
struct MyDict {
    #[tlb(adapter = "TLBHashMap::<DictKeyAdapterUint<_>, DictValAdapterNum<_, 256>>::new(256)")]
    pub data: HashMap<usize, BigUint>,
}

fn main() -> anyhow::Result<()> {
    let mut data = HashMap::new();
    for i in 0..ITEMS_COUNT {
        data.insert(i, BigUint::from(i));
    }
    let _cell = MyDict { data }.to_cell()?;
    Ok(())
}
