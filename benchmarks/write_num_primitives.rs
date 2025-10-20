use bitstream_io::{BigEndian, BitWrite, BitWriter};
use criterion::{criterion_group, criterion_main, Criterion};
use fastnum::I512;
use num_bigint::BigInt;
use std::hint::black_box;

use ton_core::cell::TonCell;
use ton_lib_core_008::cell::TonCell as TonCell008;
use tonlib_core::cell::CellBuilder as TonlibCellBuilder;
const ITERATIONS_COUNT: usize = 100;

const TEST_VALUE: u32 = 3u32;
const TEST_WRITE_BIT: usize = 10;
const THRESHOLD_TO_RECREATE_BUILDER: usize = 100;

fn write_primitive_tonlib() {
    let mut builder = TonlibCellBuilder::new();
    for i in 0..ITERATIONS_COUNT {
        if i % THRESHOLD_TO_RECREATE_BUILDER == 0 {
            builder = TonlibCellBuilder::new();
        }
        let res = builder.store_u32(TEST_WRITE_BIT, TEST_VALUE).unwrap();
        black_box(res);
    }
}

fn write_primitive_ton_lib_core_008() {
    let mut builder = TonCell008::builder();
    for i in 0..ITERATIONS_COUNT {
        if i % THRESHOLD_TO_RECREATE_BUILDER == 0 {
            builder = TonCell008::builder();
        }
        builder.write_num(&TEST_VALUE, TEST_WRITE_BIT).unwrap();
        black_box(&builder);
    }
}

fn write_primitive_bit_writer() {
    let mut buffer = Vec::new();
    buffer.reserve(128);
    let tvb = TEST_WRITE_BIT as u32;
    let mut bit_writer = BitWriter::endian(buffer, BigEndian);
    for _ in 0..ITERATIONS_COUNT {
        bit_writer.write_var(tvb, TEST_VALUE).unwrap();
        black_box(&bit_writer);
    }
}

fn write_primitive_ton_rs_current() {
    let mut builder = TonCell::builder();
    for i in 0..ITERATIONS_COUNT {
        if i % THRESHOLD_TO_RECREATE_BUILDER == 0 {
            builder = TonCell::builder();
        }
        builder.write_num(&TEST_VALUE, TEST_WRITE_BIT).unwrap();
        black_box(&builder);
    }
}
fn write_primitive_ton_rs_current_negative() {
    let mut builder = TonCell::builder();
    let tv = TEST_VALUE as i32 * (-1i32);
    for i in 0..ITERATIONS_COUNT {
        if i % THRESHOLD_TO_RECREATE_BUILDER == 0 {
            builder = TonCell::builder();
        }
        builder.write_num(&tv, TEST_WRITE_BIT).unwrap();
        black_box(&builder);
    }
}

fn write_bigint_ton_rs_current_negative() {
    let mut builder = TonCell::builder();
    let tv = BigInt::from(TEST_VALUE as i32 * (-1i32));
    for i in 0..ITERATIONS_COUNT {
        if i % THRESHOLD_TO_RECREATE_BUILDER == 0 {
            builder = TonCell::builder();
        }
        builder.write_num(&tv, TEST_WRITE_BIT).unwrap();
        black_box(&builder);
    }
}
fn write_i512_ton_rs_current_negative() {
    let mut builder = TonCell::builder();
    let tv = -I512::from(TEST_VALUE);
    for i in 0..ITERATIONS_COUNT {
        if i % THRESHOLD_TO_RECREATE_BUILDER == 0 {
            builder = TonCell::builder();
        }
        builder.write_num(&tv, TEST_WRITE_BIT).unwrap();
        black_box(&builder);
    }
}

fn benchmark_functions(c: &mut Criterion) {
    c.bench_function("write_primitive_baseline_bit_writer", |b| b.iter(write_primitive_bit_writer));
    c.bench_function("write_primitive_tonlib", |b| b.iter(write_primitive_tonlib));
    c.bench_function("write_primitive_ton_lib_core_008", |b| b.iter(write_primitive_ton_lib_core_008));
    c.bench_function("write_primitive_ton_rs_current", |b| b.iter(write_primitive_ton_rs_current));
    c.bench_function("write_primitive_ton_rs_current_negative", |b| b.iter(write_primitive_ton_rs_current_negative));
    c.bench_function("write_bigint_ton_rs_current_negative", |b| b.iter(write_bigint_ton_rs_current_negative));
    c.bench_function("write_i512_ton_rs_current_negative", |b| b.iter(write_i512_ton_rs_current_negative));
}

criterion_group!(benches, benchmark_functions);
criterion_main!(benches);
