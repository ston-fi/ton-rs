mod benchmark_utils;

use criterion::{criterion_group, criterion_main, Criterion};
use ton_lib_core_008::cell::TonCell as TonCellTonLibCore008;
use tonlib_core::cell::BagOfCells;

use crate::benchmark_utils::SHARD_BLOCK_BOC;
use std::hint::black_box;
use std::ops::Deref;
use ton_lib_core::cell::TonCell as TonCellCurrent;
use ton_lib_core::traits::tlb::TLB as TLBCurrent;
use ton_lib_core_008::traits::tlb::TLB as TLB008;

const ITERATIONS_COUNT: usize = 20;

// to run: cargo bench --bench cell_read_write
fn benchmark_functions(c: &mut Criterion) {
    run_bench!(c, boc_read_tonlib_core_old);
    run_bench!(c, boc_read_with_hash_ton_lib_core_008);
    run_bench!(c, boc_read_with_hash_ton_rs_current);
    run_bench!(c, boc_read_ton_lib_core_008);
    run_bench!(c, boc_read_ton_rs_current);

    run_bench!(c, boc_write_tonlib_core_old);
    run_bench!(c, boc_write_ton_lib_core_008);
    run_bench!(c, boc_write_ton_rs_current);
}

// tonlib-core (old)
fn boc_read_tonlib_core_old() {
    for _ in 0..ITERATIONS_COUNT {
        black_box(BagOfCells::parse(&SHARD_BLOCK_BOC).unwrap());
    }
}

fn boc_write_tonlib_core_old() {
    let cell = BagOfCells::parse(&SHARD_BLOCK_BOC).unwrap().single_root().unwrap();
    for _ in 0..ITERATIONS_COUNT {
        black_box(BagOfCells::new(&[cell.clone()]).serialize(false).unwrap());
    }
}

// ton_lib_core_008
fn boc_read_ton_lib_core_008() {
    for _ in 0..ITERATIONS_COUNT {
        black_box(TonCellTonLibCore008::from_boc(&SHARD_BLOCK_BOC).unwrap());
    }
}

fn boc_read_with_hash_ton_lib_core_008() {
    for _ in 0..ITERATIONS_COUNT {
        black_box(TonCellTonLibCore008::from_boc(&SHARD_BLOCK_BOC).unwrap().hash().unwrap());
    }
}

fn boc_write_ton_lib_core_008() {
    let cell = TonCellTonLibCore008::from_boc(&SHARD_BLOCK_BOC).unwrap().into_ref();
    for _ in 0..ITERATIONS_COUNT {
        black_box(cell.to_boc().unwrap());
    }
}

// ton_rs (current)
fn boc_read_ton_rs_current() {
    for _ in 0..ITERATIONS_COUNT {
        black_box(TonCellCurrent::from_boc(SHARD_BLOCK_BOC.deref().clone()).unwrap());
    }
}

fn boc_read_with_hash_ton_rs_current() {
    for _ in 0..ITERATIONS_COUNT {
        black_box(TonCellCurrent::from_boc(SHARD_BLOCK_BOC.deref().clone()).unwrap().hash().unwrap());
    }
}

fn boc_write_ton_rs_current() {
    let cell = TonCellCurrent::from_boc(SHARD_BLOCK_BOC.deref().clone()).unwrap();
    for _ in 0..ITERATIONS_COUNT {
        black_box(cell.to_boc().unwrap());
    }
}

criterion_group!(benches, benchmark_functions);
criterion_main!(benches);
