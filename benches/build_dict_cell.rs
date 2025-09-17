use criterion::{criterion_group, criterion_main, Criterion};
use std::collections::HashMap;
use std::hint::black_box;
use std::sync::LazyLock;
use ton_lib::tlb_adapters::{DictKeyAdapterInto, DictValAdapterNum, TLBHashMap};
use ton_lib_core::cell::TonCell;
use tonlib_core::cell::dict::predefined_writers::val_writer_unsigned_min_size;
use tonlib_core::cell::CellBuilder as TonlibCellBuilder;

const ITERATIONS_COUNT: usize = 1;
const DICT_ITEMS_COUNT: usize = 100;

static DICT_DATA: LazyLock<HashMap<usize, usize>> = LazyLock::new(|| {
    let mut dict = HashMap::new();
    for i in 0..DICT_ITEMS_COUNT {
        dict.insert(i, 3);
    }
    dict
});

fn build_dict_tonlib() {
    for _ in 0..ITERATIONS_COUNT {
        let mut builder = TonCell::builder();
        let data_clone = DICT_DATA.clone(); // must do it to compare with ton_core
                                            // MyDict{data:data_clone}
        TLBHashMap::<DictKeyAdapterInto, DictValAdapterNum<2>, _, _>::new(256)
            .write(&mut builder, &data_clone)
            .unwrap();
        let cell = builder.build().unwrap();
        black_box(cell);
    }
}

fn build_dict_tonlib_core() {
    for _ in 0..ITERATIONS_COUNT {
        let mut builder = TonlibCellBuilder::new();
        let data_clone = DICT_DATA.clone();
        builder.store_dict(256, val_writer_unsigned_min_size, data_clone).unwrap();
        let cell = builder.build().unwrap();
        black_box(cell);
    }
}

fn benchmark_functions(c: &mut Criterion) {
    c.bench_function("build_dict_tonlib", |b| b.iter(build_dict_tonlib));
    c.bench_function("build_dict_tonlib_core", |b| b.iter(build_dict_tonlib_core));
}

criterion_group!(benches, benchmark_functions);
criterion_main!(benches);
