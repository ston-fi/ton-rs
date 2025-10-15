use std::sync::{Arc, LazyLock};
use ton_lib_core::cell::TonCell;
use ton_lib_core::traits::tlb::TLB;

extern crate ton_lib_core;

static SHARD_BLOCK_BOC: LazyLock<Arc<Vec<u8>>> = LazyLock::new(|| {
    let hex = include_str!("../resources/tests/shard_block_6000000000000000_52111590.hex");
    Arc::new(hex::decode(hex).unwrap())
});

fn main() -> anyhow::Result<()> {
    for _ in 0..50000 {
        #[allow(unused)]
        let cell = TonCell::from_boc(SHARD_BLOCK_BOC.to_owned())?;
    }
    Ok(())
}
