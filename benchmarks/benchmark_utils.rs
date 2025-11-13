use std::sync::{Arc, LazyLock};

use std::time::{ SystemTime, UNIX_EPOCH};
use ton_core::errors::TonCoreError;

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

#[inline]
pub fn get_now_ns() -> u128 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() }

pub fn current_cpu_id() -> i32 { unsafe { libc::sched_getcpu() } }

pub fn check_cpu_id(id: i32) -> Result<(), TonCoreError> {
    if current_cpu_id() != id {
        let s = format!("Current CPU id {} does not match the expected CPU id {}", current_cpu_id(), id);
        Err(TonCoreError::Custom { 0: s })
    } else {
        Ok(())
    }
}

const FIBONACHI_TASK_LOAD: u64 = 13; // ~0,87

fn fibonachi(n: u64) -> u64 {
    if n == 0 {
        return 0;
    } else if n == 1 {
        return 1;
    }
    fibonachi(n - 1) + fibonachi(n - 2)
}

pub fn cpu_load_function(load_microseconds: u64) -> u64 {
    let stop_time = get_now_ns() + (load_microseconds * 1000) as u128;
    let mut sum = 0u64;
    while get_now_ns() < stop_time {
        sum += fibonachi(FIBONACHI_TASK_LOAD);
    }
    sum
}