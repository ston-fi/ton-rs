use criterion::{criterion_group, criterion_main, Criterion};
use ton_lib_core_008::cell::TonCell as TonCell008;
use tonlib_core::cell::CellBuilder as TonlibCellBuilder;

use bitstream_io::read::BitRead;
use std::hint::black_box;
use std::io::Cursor;
use std::sync::OnceLock;
use ton_lib_core::cell::{CellBitReader, TonCell};

const ITERATIONS_COUNT: usize = 102;

const TEST_VALUE_U32: u32 = 4u32;
const TEST_VALUE_I32: i32 = -4i32;
const TEST_READ_BIT: usize = 10;

// Singletons to create cells once
static TONLIB_CELL: OnceLock<tonlib_core::cell::Cell> = OnceLock::new();
static TON_LIB_CORE_008_CELL: OnceLock<TonCell008> = OnceLock::new();
static TON_RS_CELL_U32: OnceLock<TonCell> = OnceLock::new();
static TON_RS_CELL_I32: OnceLock<TonCell> = OnceLock::new();

fn get_tonlib_cell() -> &'static tonlib_core::cell::Cell {
    TONLIB_CELL.get_or_init(|| {
        let mut builder = TonlibCellBuilder::new();
        for _ in 0..ITERATIONS_COUNT {
            builder.store_u32(TEST_READ_BIT, TEST_VALUE_U32).unwrap();
        }
        builder.build().unwrap()
    })
}

fn get_ton_lib_core_008_cell() -> &'static TonCell008 {
    TON_LIB_CORE_008_CELL.get_or_init(|| {
        let mut builder = TonCell008::builder();
        for _ in 0..ITERATIONS_COUNT {
            builder.write_num(&TEST_VALUE_U32, TEST_READ_BIT).unwrap();
        }
        builder.build().unwrap()
    })
}

fn get_ton_rs_cell_u32() -> &'static TonCell {
    TON_RS_CELL_U32.get_or_init(|| {
        let mut builder = TonCell::builder();
        for _ in 0..ITERATIONS_COUNT {
            builder.write_num(&TEST_VALUE_U32, TEST_READ_BIT).unwrap();
        }
        builder.build().unwrap()
    })
}

fn get_ton_rs_cell_i32() -> &'static TonCell {
    TON_RS_CELL_I32.get_or_init(|| {
        let mut builder = TonCell::builder();
        for _ in 0..ITERATIONS_COUNT {
            builder.write_num(&TEST_VALUE_I32, TEST_READ_BIT).unwrap();
        }
        builder.build().unwrap()
    })
}

fn read_primitive_tonlib() {
    let cell = get_tonlib_cell();

    for _ in 0..10 {
        // Benchmark reading - create a new parser each iteration to match the pattern
        for _ in 0..ITERATIONS_COUNT {
            let val = cell.parse(|slice| slice.load_u32(TEST_READ_BIT)).unwrap();
            black_box(val);
        }
    }
}

fn read_primitive_ton_lib_core_008() {
    let cell = get_ton_lib_core_008_cell();

    // Create parser and read all values
    for _ in 0..10 {
        let mut parser = cell.parser();

        for _ in 0..ITERATIONS_COUNT {
            let val = parser.read_num::<u32>(TEST_READ_BIT).unwrap();
            black_box(val);
        }
    }
}

fn read_primitive_bit_reader() {
    let cell = get_ton_rs_cell_u32();

    // Create bit reader and read all values
    let tvb = TEST_READ_BIT as u32;
    for _ in 0..10 {
        let cursor = Cursor::new(cell.data.as_slice());
        let mut bit_reader = CellBitReader::new(cursor);
        for _ in 0..ITERATIONS_COUNT {
            let val: u32 = bit_reader.read_var(tvb).unwrap();
            black_box(val);
        }
    }
}

fn read_primitive_ton_rs_current() {
    let cell = get_ton_rs_cell_u32();

    // Create parser and read all values
    for _ in 0..10 {
        let mut parser = cell.parser();
        for _ in 0..ITERATIONS_COUNT {
            let val = parser.read_num::<u32>(TEST_READ_BIT).unwrap();
            black_box(val);
        }
    }
}

fn read_primitive_ton_rs_current_negative() {
    let cell = get_ton_rs_cell_i32();

    // Create parser and read all values
    for _ in 0..10 {
        let mut parser = cell.parser();
        for _ in 0..ITERATIONS_COUNT {
            let val = parser.read_num::<i32>(TEST_READ_BIT).unwrap();
            black_box(val);
        }
    }
}

fn benchmark_functions(c: &mut Criterion) {
    c.bench_function("read_primitive_baseline_bit_reader", |b| b.iter(read_primitive_bit_reader));
    c.bench_function("read_primitive_tonlib", |b| b.iter(read_primitive_tonlib));
    c.bench_function("read_primitive_ton_lib_core_008", |b| b.iter(read_primitive_ton_lib_core_008));
    c.bench_function("read_primitive_ton_rs_current", |b| b.iter(read_primitive_ton_rs_current));
    c.bench_function("read_primitive_ton_rs_current_negative", |b| b.iter(read_primitive_ton_rs_current_negative));
}

criterion_group!(benches, benchmark_functions);
criterion_main!(benches);
