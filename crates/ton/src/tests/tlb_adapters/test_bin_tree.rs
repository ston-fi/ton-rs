use super::*;
use crate::tlb_adapters::DictValAdapterNum;

#[test]
fn test_bin_tree() -> anyhow::Result<()> {
    //                                                * (0000,len=0)
    //                 * (0100,len=1)                                        * (1100, len=1)
    //        *(0010,len=2)          * [0100,len=2]=3                   * (1010,len=2)          * [1100,len=2] = 6
    // * [0000,len=3]=1   * [0010,len=3]=2                 * [1000,len=3]=4     * [1010,len=3]=5
    let data = HashMap::from([
        (
            ShardPfx {
                value: 0x0,
                bits_len: 3,
            },
            1,
        ),
        (
            ShardPfx {
                value: 0x2_000000000000000,
                bits_len: 3,
            },
            2,
        ),
        (
            ShardPfx {
                value: 0x4_000000000000000,
                bits_len: 2,
            },
            3,
        ),
        (
            ShardPfx {
                value: 0x8_000000000000000,
                bits_len: 3,
            },
            4,
        ),
        (
            ShardPfx {
                value: 0xA_000000000000000,
                bits_len: 3,
            },
            5,
        ),
        (
            ShardPfx {
                value: 0xC_000000000000000,
                bits_len: 2,
            },
            6,
        ),
    ]);
    let mut builder = TonCell::builder();
    BinTree::<DictValAdapterNum<u32, 32>>::write(&mut builder, &data)?;
    let cell = builder.build()?;
    let mut parser = cell.parser();
    let parsed_data = BinTree::<DictValAdapterNum<u32, 32>>::read(&mut parser)?;
    assert_eq!(data, parsed_data);
    Ok(())
}
