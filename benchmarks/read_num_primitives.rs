// use criterion::{criterion_group, criterion_main, Criterion};
// use ton_lib_core_008::cell::TonCell as TonCell008;
// use tonlib_core::cell::CellBuilder as TonlibCellBuilder;
//
// use std::hint::black_box;
// use ton_lib_core::cell::TonCell;
//
// const ITERATIONS_COUNT: usize = 100;
//
// const TEST_VALUE:i32 =-4i32;
// const TEST_WRITE_BIT:usize =32;
// fn read_primitive_ton_rs_current() {
//     let mut builder = TonCell::builder();
//     for i in 0..ITERATIONS_COUNT {
//         if i % 100 == 0 {
//             builder = TonCell::builder();
//         }
//         builder.write_num(&TEST_VALUE, TEST_WRITE_BIT).unwrap();
//         black_box(&builder);
//     }
// }
//
// fn benchmark_functions(c: &mut Criterion) {
//     // c.bench_function("write_primitive_tonlib", |b| b.iter(write_primitive_tonlib));
//     // c.bench_function("write_primitive_ton_lib_core_008", |b| b.iter(write_primitive_ton_lib_core_008));
//     c.bench_function("read_primitive_ton_rs_current", |b| b.iter(read_primitive_ton_rs_current));
// }
//
// criterion_group!(benches, benchmark_functions);
// criterion_main!(benches);
