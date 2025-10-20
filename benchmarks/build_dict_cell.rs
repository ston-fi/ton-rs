mod benchmark_utils;
use criterion::{criterion_group, criterion_main, Criterion};
use std::collections::HashMap;
use std::hint::black_box;
use std::sync::LazyLock;
use tlb_adapters_0039::TLBHashMap;
use ton_lib::tlb_adapters as tlb_adapters_current;
use ton_lib_0039::tlb_adapters as tlb_adapters_0039;
use ton_core::cell::TonCell;
use ton_lib_core_008::cell::TonCell as TonCell008;
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

// cargo bench --bench build_dict_cell
fn benchmark_functions(c: &mut Criterion) {
    run_bench!(c, build_dict_tonlib_core_old);
    run_bench!(c, build_dict_ton_lib_0039);
    run_bench!(c, build_dict_ton_rs_current);
}

fn build_dict_tonlib_core_old() {
    for _ in 0..ITERATIONS_COUNT {
        let mut builder = TonlibCellBuilder::new();
        let data_clone = DICT_DATA.clone();
        builder.store_dict(256, val_writer_unsigned_min_size, data_clone).unwrap();
        black_box(builder.build().unwrap());
    }
}

fn build_dict_ton_lib_0039() {
    for _ in 0..ITERATIONS_COUNT {
        let mut builder = TonCell008::builder();
        let data_clone = DICT_DATA.clone(); // must do it to compare with ton_core
                                            // MyDict{data:data_clone}
        TLBHashMap::<tlb_adapters_0039::DictKeyAdapterInto, tlb_adapters_0039::DictValAdapterNum<2>, _, _>::new(256)
            .write(&mut builder, &data_clone)
            .unwrap();
        black_box(builder.build().unwrap());
    }
}

fn build_dict_ton_rs_current() {
    for _ in 0..ITERATIONS_COUNT {
        let mut builder = TonCell::builder();
        let data_clone = DICT_DATA.clone(); // must do it to compare with ton_core
                                            // MyDict{data:data_clone}
        tlb_adapters_current::TLBHashMap::<
            tlb_adapters_current::DictKeyAdapterUint<_>,
            tlb_adapters_current::DictValAdapterNum<_, 2>,
        >::new(256)
        .write(&mut builder, &data_clone)
        .unwrap();
        black_box(builder.build().unwrap());
    }
}

criterion_group!(benches, benchmark_functions);
criterion_main!(benches);
