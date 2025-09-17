use crate::block_tlb::TVMStack;
use crate::tep::tvm_results::tvm_result::TVMResult;
use std::ops::Deref;
use ton_lib_core::errors::TonCoreError;
use ton_lib_core::traits::tlb::TLB;
use ton_lib_core::types::TonAddress;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetNFTAddressByIndexResult {
    pub nft_address: TonAddress,
}

impl TVMResult for GetNFTAddressByIndexResult {
    fn from_stack(stack: &mut TVMStack) -> Result<Self, TonCoreError> {
        let nft_address = TonAddress::from_cell(stack.pop_cell()?.deref())?;

        Ok(Self { nft_address })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_get_jetton_data_result() -> anyhow::Result<()> {
        // Plush pepes 298 EQBUXuQI612W1e71Gk5atugejGqteQeDa8hA9tTwREcXWQiv, Collection EQBG-g6ahkAUGWpefWbx-D_9sQ8oWbvy6puuq78U2c4NUDFS
        let result = GetNFTAddressByIndexResult::from_boc_hex("b5ee9c7201010301003200020f000001040010b020010200000043800a8bdc811d6bb2dabddea349cb56dd03d18d55af20f06d79081eda9e0888e2eb30")?;
        assert_eq!(result.nft_address, TonAddress::from_str("EQBUXuQI612W1e71Gk5atugejGqteQeDa8hA9tTwREcXWQiv")?);
        Ok(())
    }
}
