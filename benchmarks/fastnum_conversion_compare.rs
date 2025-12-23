mod benchmark_utils;
use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use fastnum::{I512, U256, i512};

static VALUE: I512 = i512!(90111020304222200504);

// cargo bench --bench fastnum_conversion_compare
// string_conversion       time:   [144.16 ns 144.54 ns 144.94 ns]
// slice_conversion        time:   [27.620 ns 27.743 ns 27.858 ns]
fn benchmark_functions(c: &mut Criterion) {
    run_bench!(c, string_conversion);
    run_bench!(c, slice_conversion);
}
fn string_conversion() {
    let val = U256::from_str(&VALUE.to_string()).unwrap();
    black_box(val);
}

fn slice_conversion() {
    let val = U256::from_be_slice(VALUE.to_radix_be(256).as_slice()).unwrap();
    black_box(val);
}

criterion_group!(benches, benchmark_functions);
criterion_main!(benches);
