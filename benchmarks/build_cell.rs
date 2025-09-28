use criterion::{criterion_group, criterion_main, Criterion};
use tonlib_core::cell::CellBuilder as TonlibCellBuilder;

use std::hint::black_box;
use ton_lib_core::cell::TonCell;

const ITERATIONS_COUNT: usize = 100;

fn build_empty_cell_tonlib() {
    for _ in 0..ITERATIONS_COUNT {
        let mut builder = TonCell::builder();
        builder.write_ref(TonCell::builder().build().unwrap().into_ref()).unwrap();
        let cell = builder.build().unwrap();
        black_box(cell);
    }
}

fn build_empty_cell_tonlib_core() {
    for _ in 0..ITERATIONS_COUNT {
        let mut builder = TonlibCellBuilder::new();
        builder.store_child(TonlibCellBuilder::new().build().unwrap()).unwrap();
        let cell = builder.build().unwrap();
        black_box(cell);
    }
}

fn build_full_cell_tonlib() {
    for _ in 0..ITERATIONS_COUNT {
        let mut builder1 = TonCell::builder();
        builder1.write_bits([1, 2, 3], 24).unwrap();

        let mut builder2 = TonCell::builder();
        builder2.write_bits([10, 20, 30], 24).unwrap();

        let mut builder3 = TonCell::builder();
        builder3.write_bits([100, 200, 255], 24).unwrap();

        let mut builder = TonCell::builder();
        builder.write_ref(builder1.build().unwrap().into_ref()).unwrap();
        builder.write_ref(builder2.build().unwrap().into_ref()).unwrap();
        builder.write_ref(builder3.build().unwrap().into_ref()).unwrap();

        let cell = builder.build().unwrap();
        black_box(cell);
    }
}

fn build_full_cell_tonlib_calc_hash() {
    for _ in 0..ITERATIONS_COUNT {
        let mut builder1 = TonCell::builder();
        builder1.write_bits([1, 2, 3], 24).unwrap();

        let mut builder2 = TonCell::builder();
        builder2.write_bits([10, 20, 30], 24).unwrap();

        let mut builder3 = TonCell::builder();
        builder3.write_bits([100, 200, 255], 24).unwrap();

        let mut builder = TonCell::builder();
        builder.write_ref(builder1.build().unwrap().into_ref()).unwrap();
        builder.write_ref(builder2.build().unwrap().into_ref()).unwrap();
        builder.write_ref(builder3.build().unwrap().into_ref()).unwrap();

        let cell = builder.build().unwrap();
        let hash = cell.hash().unwrap();
        black_box(hash);
    }
}

fn build_full_cell_tonlib_core() {
    for _ in 0..ITERATIONS_COUNT {
        let mut builder1 = TonlibCellBuilder::new();
        builder1.store_slice(&[1, 2, 3]).unwrap();

        let mut builder2 = TonlibCellBuilder::new();
        builder2.store_slice(&[10, 20, 30]).unwrap();

        let mut builder3 = TonlibCellBuilder::new();
        builder3.store_slice(&[100, 200, 255]).unwrap();

        let mut builder = TonlibCellBuilder::new();
        builder.store_child(builder1.build().unwrap()).unwrap();
        builder.store_child(builder2.build().unwrap()).unwrap();
        builder.store_child(builder3.build().unwrap()).unwrap();

        let cell = builder.build().unwrap();
        black_box(cell);
    }
}

fn benchmark_functions(c: &mut Criterion) {
    c.bench_function("build_empty_cell_tonlib", |b| b.iter(build_empty_cell_tonlib));
    c.bench_function("build_empty_cell_tonlib_core", |b| b.iter(build_empty_cell_tonlib_core));

    c.bench_function("build_full_cell_tonlib", |b| b.iter(build_full_cell_tonlib));
    c.bench_function("build_full_cell_tonlib_calc_hash", |b| b.iter(build_full_cell_tonlib_calc_hash));
    c.bench_function("build_full_cell_tonlib_core", |b| b.iter(build_full_cell_tonlib_core));
}

criterion_group!(benches, benchmark_functions);
criterion_main!(benches);
