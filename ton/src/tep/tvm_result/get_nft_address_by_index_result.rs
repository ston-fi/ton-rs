use ton_core::types::TonAddress;
use ton_macros::TVMType;

#[derive(Debug, Clone, PartialEq, Eq, TVMType)]
pub struct GetNFTAddressByIndexResult {
    pub nft_address: TonAddress,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::block_tlb::TVMType;
    use std::str::FromStr;

    #[test]
    fn test_get_jetton_data_result() -> anyhow::Result<()> {
        // Plush pepes 298 EQBUXuQI612W1e71Gk5atugejGqteQeDa8hA9tTwREcXWQiv, Collection EQBG-g6ahkAUGWpefWbx-D_9sQ8oWbvy6puuq78U2c4NUDFS
        let result = GetNFTAddressByIndexResult::from_stack_boc_hex(
            "b5ee9c7201010301003200020f000001040010b020010200000043800a8bdc811d6bb2dabddea349cb56dd03d18d55af20f06d79081eda9e0888e2eb30",
        )?;
        assert_eq!(result.nft_address, TonAddress::from_str("EQBUXuQI612W1e71Gk5atugejGqteQeDa8hA9tTwREcXWQiv")?);
        Ok(())
    }
}
