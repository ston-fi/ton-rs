use tonlib_core::cell::CellBuilder;

extern crate tonlib_core;

fn main() -> anyhow::Result<()> {
    for _ in 0..10000000 {
        let mut builder1 = CellBuilder::new();
        builder1.store_bit(true)?;
        builder1.store_slice(&[1, 2, 3])?;
        builder1.store_u32(4, 4)?;

        let mut builder2 = CellBuilder::new();
        builder2.store_slice(&[10, 20, 30])?;

        let mut builder3 = CellBuilder::new();
        builder3.store_slice(&[100, 200, 255])?;

        let mut builder = CellBuilder::new();
        builder.store_child(builder1.build()?)?;
        builder.store_child(builder2.build()?)?;
        builder.store_child(builder3.build()?)?;

        #[allow(unused)]
        let cell = builder.build()?;
        // println!("{cell}");
    }
    Ok(())
}
