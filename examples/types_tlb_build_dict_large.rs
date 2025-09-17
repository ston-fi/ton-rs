use num_bigint::BigUint;
use std::collections::HashMap;
use ton_lib::tlb_adapters::DictKeyAdapterInto;
use ton_lib::tlb_adapters::DictValAdapterNum;
use ton_lib::tlb_adapters::TLBHashMap;
use ton_lib_core::traits::tlb::TLB;
use ton_lib_core::TLB;

extern crate num_bigint;
extern crate ton_lib;
extern crate tonlib_core;

// const ITEMS_COUNT: usize = 40000000;
const ITEMS_COUNT: usize = 400000;

#[derive(TLB)]
struct MyDict {
    #[tlb(adapter = "TLBHashMap::<DictKeyAdapterInto, DictValAdapterNum<256>, _, _>::new(256)")]
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
