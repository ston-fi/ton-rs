use std::sync::{Arc, LazyLock};

#[macro_export]
macro_rules! run_bench {
    ($c:expr, $func:ident) => {
        $c.bench_function(stringify!($func), |b| b.iter($func));
    };
}

#[allow(unused)] // it's used in the benchmarks modules
pub(super) static SHARD_BLOCK_BOC: LazyLock<Arc<Vec<u8>>> = LazyLock::new(|| {
    let hex = include_str!("../resources/tests/shard_block_6000000000000000_52111590.hex");
    Arc::new(hex::decode(hex).unwrap())
});
