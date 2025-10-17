extern crate ton_lib_core;

use ton_lib_core::cell::TonCell;

fn main() -> anyhow::Result<()> {
    for _ in 0..10000000 {
        let mut builder1 = TonCell::builder();
        builder1.write_bit(true)?;
        builder1.write_bits([1, 2, 3], 24)?;
        builder1.write_num(&4, 4)?;

        let mut builder2 = TonCell::builder();
        builder2.write_bits([10, 20, 30], 24)?;

        let mut builder3 = TonCell::builder();
        builder3.write_bits([100, 200, 255], 24)?;

        let mut builder = TonCell::builder();
        builder.write_ref(builder1.build()?)?;
        builder.write_ref(builder2.build()?)?;
        builder.write_ref(builder3.build()?)?;

        #[allow(unused)]
        let cell = builder.build()?;
        // println!("{cell}");
    }
    Ok(())
}
