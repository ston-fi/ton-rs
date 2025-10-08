use criterion::{criterion_group, criterion_main, Criterion};
use ton_lib_core_008::cell::TonCell as TonCell008;
use tonlib_core::cell::CellBuilder as TonlibCellBuilder;

use std::hint::black_box;
use ton_lib_core::cell::TonCell;

const ITERATIONS_COUNT: usize = 100;

fn write_primitive_tonlib() {
    let mut builder = TonlibCellBuilder::new();
    for i in 0..ITERATIONS_COUNT {
        if i % 100 == 0 {
            builder = TonlibCellBuilder::new();
        }
        let res = builder.store_u32(10, 4).unwrap();
        black_box(res);
    }
}

fn write_primitive_ton_lib_core_008() {
    let mut builder = TonCell008::builder();
    for i in 0..ITERATIONS_COUNT {
        if i % 100 == 0 {
            builder = TonCell008::builder();
        }
        builder.write_num(&4, 10).unwrap();
        black_box(&builder);
    }
}

fn write_primitive_ton_rs_current() {
    let mut builder = TonCell::builder();
    for i in 0..ITERATIONS_COUNT {
        if i % 100 == 0 {
            builder = TonCell::builder();
        }
        builder.write_num(&4, 10).unwrap();
        black_box(&builder);
    }
}

fn benchmark_functions(c: &mut Criterion) {
    c.bench_function("write_primitive_tonlib", |b| b.iter(write_primitive_tonlib));
    c.bench_function("write_primitive_ton_lib_core_008", |b| b.iter(write_primitive_ton_lib_core_008));
    c.bench_function("write_primitive_ton_rs_current", |b| b.iter(write_primitive_ton_rs_current));
}

criterion_group!(benches, benchmark_functions);
criterion_main!(benches);
