mod benchmark_utils;
use criterion::{criterion_group, criterion_main, Criterion};
use ton_lib_core_008::cell::TonCell as TonCellTonLibCore008;
use tonlib_core::cell::CellBuilder;

use std::hint::black_box;
use ton_lib_core::cell::TonCell as TonCellCurrent;

const ITERATIONS_COUNT: usize = 100;

// cargo bench --bench build_full_cell
fn benchmark_functions(c: &mut Criterion) {
    run_bench!(c, build_full_cell_tonlib_core_old);
    run_bench!(c, build_full_cell_ton_lib_008);
    run_bench!(c, build_full_cell_calc_hash_ton_lib_008_calc_hash);
    run_bench!(c, build_full_cell_ton_rs_current);
    run_bench!(c, build_full_cell_calc_hash_ton_rs_current_calc_hash);
}

fn build_full_cell_tonlib_core_old() {
    for _ in 0..ITERATIONS_COUNT {
        let mut builder1 = CellBuilder::new();
        builder1.store_slice(&[1, 2, 3]).unwrap();

        let mut builder2 = CellBuilder::new();
        builder2.store_slice(&[10, 20, 30]).unwrap();

        let mut builder3 = CellBuilder::new();
        builder3.store_slice(&[100, 200, 255]).unwrap();

        let mut builder = CellBuilder::new();
        builder.store_child(builder1.build().unwrap()).unwrap();
        builder.store_child(builder2.build().unwrap()).unwrap();
        builder.store_child(builder3.build().unwrap()).unwrap();

        black_box(builder.build().unwrap());
    }
}

fn build_full_cell_ton_lib_008() {
    for _ in 0..ITERATIONS_COUNT {
        let mut builder1 = TonCellTonLibCore008::builder();
        builder1.write_bits([1, 2, 3], 24).unwrap();

        let mut builder2 = TonCellTonLibCore008::builder();
        builder2.write_bits([10, 20, 30], 24).unwrap();

        let mut builder3 = TonCellTonLibCore008::builder();
        builder3.write_bits([100, 200, 255], 24).unwrap();

        let mut builder = TonCellTonLibCore008::builder();
        builder.write_ref(builder1.build().unwrap().into_ref()).unwrap();
        builder.write_ref(builder2.build().unwrap().into_ref()).unwrap();
        builder.write_ref(builder3.build().unwrap().into_ref()).unwrap();
        black_box(builder.build().unwrap());
    }
}

fn build_full_cell_calc_hash_ton_lib_008_calc_hash() {
    for _ in 0..ITERATIONS_COUNT {
        let mut builder1 = TonCellTonLibCore008::builder();
        builder1.write_bits([1, 2, 3], 24).unwrap();

        let mut builder2 = TonCellTonLibCore008::builder();
        builder2.write_bits([10, 20, 30], 24).unwrap();

        let mut builder3 = TonCellTonLibCore008::builder();
        builder3.write_bits([100, 200, 255], 24).unwrap();

        let mut builder = TonCellTonLibCore008::builder();
        builder.write_ref(builder1.build().unwrap().into_ref()).unwrap();
        builder.write_ref(builder2.build().unwrap().into_ref()).unwrap();
        builder.write_ref(builder3.build().unwrap().into_ref()).unwrap();

        let cell = builder.build().unwrap();
        let hash = cell.hash().unwrap();
        black_box(hash);
    }
}

fn build_full_cell_ton_rs_current() {
    for _ in 0..ITERATIONS_COUNT {
        let mut builder1 = TonCellCurrent::builder();
        builder1.write_bits([1, 2, 3], 24).unwrap();

        let mut builder2 = TonCellCurrent::builder();
        builder2.write_bits([10, 20, 30], 24).unwrap();

        let mut builder3 = TonCellCurrent::builder();
        builder3.write_bits([100, 200, 255], 24).unwrap();

        let mut builder = TonCellCurrent::builder();
        builder.write_ref(builder1.build().unwrap()).unwrap();
        builder.write_ref(builder2.build().unwrap()).unwrap();
        builder.write_ref(builder3.build().unwrap()).unwrap();
        black_box(builder.build().unwrap());
    }
}

fn build_full_cell_calc_hash_ton_rs_current_calc_hash() {
    for _ in 0..ITERATIONS_COUNT {
        let mut builder1 = TonCellCurrent::builder();
        builder1.write_bits([1, 2, 3], 24).unwrap();

        let mut builder2 = TonCellCurrent::builder();
        builder2.write_bits([10, 20, 30], 24).unwrap();

        let mut builder3 = TonCellCurrent::builder();
        builder3.write_bits([100, 200, 255], 24).unwrap();

        let mut builder = TonCellCurrent::builder();
        builder.write_ref(builder1.build().unwrap()).unwrap();
        builder.write_ref(builder2.build().unwrap()).unwrap();
        builder.write_ref(builder3.build().unwrap()).unwrap();

        let cell = builder.build().unwrap();
        let hash = cell.hash().unwrap();
        black_box(hash);
    }
}

criterion_group!(benches, benchmark_functions);
criterion_main!(benches);
