use super::*;
use crate::cell::TonHash;
use std::str::FromStr;

#[test]
fn test_boc_create() {
    let boc = BoC::new(TonCell::empty().to_owned());
    assert_eq!(boc.roots.len(), 1);
}

#[test]
fn test_boc_from_to() -> anyhow::Result<()> {
    let boc_hex = "b5ee9c720101040100b900010d000000000000050102c9801459f7c0a12bb4ac4b78a788c425ee4d52f8b6041dda17b77b09fc5a03e894d6900287cd9fbe2ea663415da0aa6bbdf0cb136abe9c4f45214dd259354b80da8c265a006aebb27f5d0f1daf43e200f52408f3eb9ff5610f5b43284224644e7c6a590d14400203084202c00836440d084e44fb94316132ac5a21417ef4f429ee09b5560b5678b334c3e8084202c95a2ed22ab516f77f9d4898dc4578e72f18a2448e8f6832334b0b4bf501bc79";
    let boc = BoC::from_hex(boc_hex)?;
    let boc_hex_back = boc.to_hex(false)?;
    assert_eq!(boc_hex, boc_hex_back);
    Ok(())
}

#[test]
fn test_boc_from_to_cell_data() -> anyhow::Result<()> {
    let boc_hex = "b5ee9c720101040100b900010d000000000000050102c9801459f7c0a12bb4ac4b78a788c425ee4d52f8b6041dda17b77b09fc5a03e894d6900287cd9fbe2ea663415da0aa6bbdf0cb136abe9c4f45214dd259354b80da8c265a006aebb27f5d0f1daf43e200f52408f3eb9ff5610f5b43284224644e7c6a590d14400203084202c00836440d084e44fb94316132ac5a21417ef4f429ee09b5560b5678b334c3e8084202c95a2ed22ab516f77f9d4898dc4578e72f18a2448e8f6832334b0b4bf501bc79";
    let cell = BoC::from_hex(boc_hex)?.single_root()?;
    let hash = cell.hash()?;
    assert_eq!(hash, &TonHash::from_str("46284eb2ecbae743ce06dc39ba379f854f135dc8029c488abf8e84e90106302a")?);
    let serial_hex = BoC::new(cell).to_hex(false)?;
    assert_eq!(boc_hex, serial_hex);
    Ok(())
}

#[test]
fn test_boc_from_to_cell_lib() -> anyhow::Result<()> {
    let boc_hex = "b5ee9c7201010101002300084202a9338ecd624ca15d37e4a8d9bf677ddc9b84f0e98f05f2fb84c7afe332a281b4";
    let cell = BoC::from_hex(boc_hex)?.single_root()?;
    let hash = cell.hash()?;
    assert_eq!(hash, &TonHash::from_str("ec614ea4aaea3f7768606f1c1632b3374d3de096a1e7c4ba43c8009c487fee9d")?);
    let serial_hex = BoC::new(cell).to_hex(false)?;
    assert_eq!(boc_hex, serial_hex);
    Ok(())
}
