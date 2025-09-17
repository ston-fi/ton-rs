use crate::tlb_adapters::ConstLen;
use crate::tlb_adapters::DictKeyAdapterTonHash;
use crate::tlb_adapters::DictValAdapterTLB;
use crate::tlb_adapters::TLBHashMapE;
use std::collections::HashMap;
use ton_lib_core::cell::{TonCellRef, TonHash};
use ton_lib_core::errors::TonCoreError;
use ton_lib_core::traits::tlb::TLB;
use ton_lib_core::types::TonAddress;
use ton_lib_core::TLB;

// https://github.com/ton-blockchain/ton/blob/59a8cf0ae5c3062d14ec4c89a04fee80b5fd05c1/crypto/block/block.tlb#L281
#[derive(Debug, Clone, PartialEq, TLB)]
pub struct StateInit {
    #[tlb(bits_len = 5)]
    pub split_depth: Option<u8>,
    pub tick_tock: Option<TickTock>,
    pub code: Option<TonCellRef>,
    pub data: Option<TonCellRef>,
    #[tlb(adapter = "TLBHashMapE::<DictKeyAdapterTonHash, DictValAdapterTLB, _, _>::new(256)")]
    pub library: HashMap<TonHash, SimpleLib>,
}

#[derive(Debug, Clone, PartialEq, Eq, TLB)]
pub struct SimpleLib {
    pub public: bool,
    pub root: TonCellRef,
}

#[derive(Debug, Clone, PartialEq, TLB)]
pub struct TickTock {
    pub tick: bool,
    pub tock: bool,
}

impl StateInit {
    pub fn new(code: TonCellRef, data: TonCellRef) -> Self {
        StateInit {
            split_depth: None,
            tick_tock: None,
            code: Some(code),
            data: Some(data),
            library: Default::default(),
        }
    }

    pub fn derive_address(&self, workchain: i32) -> Result<TonAddress, TonCoreError> {
        Ok(TonAddress::new(workchain, self.cell_hash()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use ton_lib_core::cell::TonCell;

    #[test]
    fn test_state_init_regular_contract() -> anyhow::Result<()> {
        // state_init for UQCJ7Quj9gM_SE3uwOk3gEJC2JFQcgg0s7CSpLr7B_2yiHPG caching
        let source_cell = TonCell::from_boc_hex("b5ee9c720102160100030400020134020100510000082f29a9a31738dd3a33f904d35e2f4f6f9af2d2f9c563c05faa6bb0b12648d5632083ea3f89400114ff00f4a413f4bcf2c80b03020120090404f8f28308d71820d31fd31fd31f02f823bbf264ed44d0d31fd31fd3fff404d15143baf2a15151baf2a205f901541064f910f2a3f80024a4c8cb1f5240cb1f5230cbff5210f400c9ed54f80f01d30721c0009f6c519320d74a96d307d402fb00e830e021c001e30021c002e30001c0039130e30d03a4c8cb1f12cb1fcbff08070605000af400c9ed54006c810108d718fa00d33f305224810108f459f2a782106473747270748018c8cb05cb025005cf165003fa0213cb6acb1f12cb3fc973fb000070810108d718fa00d33fc8542047810108f451f2a782106e6f746570748018c8cb05cb025006cf165004fa0214cb6a12cb1fcb3fc973fb0002006ed207fa00d4d422f90005c8ca0715cbffc9d077748018c8cb05cb0222cf165005fa0214cb6b12ccccc973fb00c84014810108f451f2a702020148130a0201200c0b0059bd242b6f6a2684080a06b90fa0218470d4080847a4937d29910ce6903e9ff9837812801b7810148987159f31840201200e0d0011b8c97ed44d0d70b1f8020158120f02012011100019af1df6a26840106b90eb858fc00019adce76a26840206b90eb85ffc0003db29dfb513420405035c87d010c00b23281f2fff274006040423d029be84c6002e6d001d0d3032171b0925f04e022d749c120925f04e002d31f218210706c7567bd22821064737472bdb0925f05e003fa403020fa4401c8ca07cbffc9d0ed44d0810140d721f404305c810108f40a6fa131b3925f07e005d33fc8258210706c7567ba923830e30d03821064737472ba925f06e30d1514008a5004810108f45930ed44d0810140d720c801cf16f400c9ed540172b08e23821064737472831eb17080185005cb055003cf1623fa0213cb6acb1fcb3fc98040fb00925f03e2007801fa00f40430f8276f2230500aa121bef2e0508210706c7567831eb17080185004cb0526cf1658fa0219f400cb6917cb1f5260cb3f20c98040fb0006")?;
        let parsed_state_init = StateInit::from_cell(&source_cell)?;

        // assert_eq!(parsed_state_init.split_depth, None);
        assert_eq!(parsed_state_init.tick_tock, None);
        assert!(parsed_state_init.code.is_some());
        assert!(parsed_state_init.data.is_some());
        assert_eq!(parsed_state_init.library, Default::default());

        let serial_cell = parsed_state_init.to_cell()?;
        assert_eq!(source_cell, serial_cell);

        let parsed_back = StateInit::from_cell(&serial_cell)?;
        assert_eq!(parsed_state_init, parsed_back);
        Ok(())
    }

    #[test]
    fn test_state_init_derive_address() -> anyhow::Result<()> {
        let code_cell = TonCell::from_boc_hex(
            "b5ee9c7201010101002300084202a9338ecd624ca15d37e4a8d9bf677ddc9b84f0e98f05f2fb84c7afe332a281b4",
        )?;
        let data_cell = TonCell::from_boc_hex("b5ee9c720101040100b900010d000000000000050102c9801459f7c0a12bb4ac4b78a788c425ee4d52f8b6041dda17b77b09fc5a03e894d6900287cd9fbe2ea663415da0aa6bbdf0cb136abe9c4f45214dd259354b80da8c265a006aebb27f5d0f1daf43e200f52408f3eb9ff5610f5b43284224644e7c6a590d14400203084202c00836440d084e44fb94316132ac5a21417ef4f429ee09b5560b5678b334c3e8084202c95a2ed22ab516f77f9d4898dc4578e72f18a2448e8f6832334b0b4bf501bc79")?.into_ref();
        let state_init = StateInit::new(code_cell.into_ref(), data_cell);
        let address = state_init.derive_address(0)?;
        let exp_addr = TonAddress::from_str("EQAdltEfzXG_xteLFaKFGd-HPVKrEJqv_FdC7z2roOddRNdM")?;
        assert_eq!(address, exp_addr);
        Ok(())
    }
}
