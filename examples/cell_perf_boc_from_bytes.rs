use std::ops::Deref;
use std::sync::LazyLock;
use ton_lib_core::boc::BoC;
extern crate ton_lib_core;

static BOC_BYTES: LazyLock<Vec<u8>> = LazyLock::new(|| {
    let boc_hex = "b5ee9c720101040100b900010d000000000000050102c9801459f7c0a12bb4ac4b78a788c425ee4d52f8b6041dda17b77b09fc5a03e894d6900287cd9fbe2ea663415da0aa6bbdf0cb136abe9c4f45214dd259354b80da8c265a006aebb27f5d0f1daf43e200f52408f3eb9ff5610f5b43284224644e7c6a590d14400203084202c00836440d084e44fb94316132ac5a21417ef4f429ee09b5560b5678b334c3e8084202c95a2ed22ab516f77f9d4898dc4578e72f18a2448e8f6832334b0b4bf501bc79";
    hex::decode(boc_hex).unwrap()
});

fn main() -> anyhow::Result<()> {
    for _ in 0..10000000 {
        #[allow(unused)]
        let boc = BoC::from_bytes(BOC_BYTES.deref())?;
    }
    Ok(())
}
