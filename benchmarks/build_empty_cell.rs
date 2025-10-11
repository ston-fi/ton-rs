mod benchmark_utils;
use criterion::{criterion_group, criterion_main, Criterion};
use ton_lib_core_008::cell::TonCell as TonCellTonLibCore008;
use tonlib_core::cell::CellBuilder;

use std::hint::black_box;
use ton_lib_core::cell::TonCell as TonCellCurrent;

const ITERATIONS_COUNT: usize = 100;

// cargo bench --bench build_empty_cell
fn benchmark_functions(c: &mut Criterion) {
    run_bench!(c, build_cell_with_ref_tonlib_core_old);
    run_bench!(c, build_cell_with_ref_ton_lib_008);
    run_bench!(c, build_cell_with_ref_ton_rs_current);
}

fn build_cell_with_ref_tonlib_core_old() {
    for _ in 0..ITERATIONS_COUNT {
        let mut builder = CellBuilder::new();
        builder.store_child(CellBuilder::new().build().unwrap()).unwrap();
        black_box(builder.build().unwrap());
    }
}

fn build_cell_with_ref_ton_lib_008() {
    for _ in 0..ITERATIONS_COUNT {
        let mut builder = TonCellTonLibCore008::builder();
        builder.write_ref(TonCellTonLibCore008::builder().build().unwrap().into_ref()).unwrap();
        black_box(builder.build().unwrap());
    }
}

fn build_cell_with_ref_ton_rs_current() {
    for _ in 0..ITERATIONS_COUNT {
        let mut builder = TonCellCurrent::builder();
        builder.write_ref(TonCellCurrent::builder().build().unwrap()).unwrap();
        black_box(builder.build().unwrap());
    }
}

criterion_group!(benches, benchmark_functions);
criterion_main!(benches);
