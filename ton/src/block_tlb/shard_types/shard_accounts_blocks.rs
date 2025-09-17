pub struct ShardAccountBlocks; // TODO
                               // use crate::types::tlb::adapters::DictRef;
                               // use crate::types::tlb::adapters::dict_key_adapters::DictKeyAdapterInto;
                               // use crate::types::tlb::block_tlb::block::TLBRef;
                               // use crate::types::tlb::adapters::Dict;
                               // use crate::types::tlb::adapters::dict_val_adapters::DictValAdapterTLB;
                               // use crate::types::tlb::adapters::dict_key_adapters::DictKeyAdapterTonHash;
                               // use std::collections::HashMap;
                               // use ton_macros::TLBDerive;
                               // use crate::cell::ton_hash::TonHash;
                               // use crate::types::tlb::block_tlb::coins::CurrencyCollection;
                               // use crate::types::tlb::block_tlb::hash_update::HashUpdate;
                               // use crate::types::tlb::block_tlb::tx::Tx;
                               //
                               // #[derive(Debug, Clone, PartialEq, TLBDerive)]
                               // pub struct ShardAccountBlocks {
                               //     #[tlb_derive(adapter = "DictRef::<DictKeyAdapterTonHash, DictValAdapterTLB, _, _>::new(256)")]
                               //     pub data: HashMap<TonHash, AccountBlockData>
                               // }
                               //
                               // #[derive(Debug, Clone, PartialEq, TLBDerive)]
                               // pub struct AccountBlockData {
                               //     pub txs: AccountTxs,
                               //     pub currency: CurrencyCollection,
                               // }
                               //
                               // #[derive(Debug, Clone, PartialEq, TLBDerive)]
                               // #[tlb_derive(prefix = 0x5, bits_len = 4)]
                               // pub struct AccountTxs {
                               //     pub address: TonHash,
                               //     #[tlb_derive(adapter = "DictRef::<DictKeyAdapterInto, DictValAdapterTLB, _, _>::new(64)")]
                               //     txs: HashMap<u64, AccountTx>,
                               //     #[tlb_derive(adapter = "TLBRef")]
                               //     pub state_update: HashUpdate
                               // }
                               //
                               // #[derive(Debug, Clone, PartialEq, TLBDerive)]
                               // pub struct AccountTx {
                               //     #[tlb_derive(adapter = "TLBRef")]
                               //     pub tx: Tx,
                               //     pub currency: CurrencyCollection,
                               // }
                               //
                               // #[cfg(test)]
                               // mod tests {
                               //     use std::str::FromStr;
                               //     use crate::cell::ton_cell::TonCell;
                               //     use crate::cell::ton_hash::TonHash;
                               //     use crate::types::tlb::block_tlb::block::shard_accounts_blocks::{AccountBlockData, AccountTx, AccountTxs, ShardAccountBlocks};
                               //     use crate::types::tlb::TLB;
                               //
                               //     #[test]
                               //     fn test_shard_accounts_blocks() -> anyhow::Result<()> {
                               //         let boc_hex = "b5ee9c720102170100047700010182010203404002030397bfb333333333333333333333333333333333333333333333333333333333333333029999999999999999999999999999999999999999999999999999999999999999cf80000cca7a57c6e0040405060297bf955555555555555555555555555555555555555555555555555555555555555502aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaad000000cca7a57c6e0c107080103504009010340400a00827224cb63f6ca8a66c7ab33a3b4991469c85f92425674a7b4be42a90f3957c8597544e7eab684e87b6634d5f11a016f513b7abbe1d362dafc625d935458d9e9cb4f03af7555555555555555555555555555555555555555555555555555555555555555500003329e95f1b8310636926488004793e561dc21fadb9fb285214ca3316f92d3f89f80a3949b1ab00003329e94fd94368044f0900014080b080c00827201d0ee799fd8b3d40de96ec59f0e0cf748c62a5b659b4dae4a0a1ae8250648661b37aed542989ee05fc2bb5ab9eb7f01fce788479d2657918119f04fa9932a2403af7333333333333333333333333333333333333333333333333333333333333333300003329e95f1b81285e0378a9c9e9bb207df070a626cdd9069e0d11e7be400f041a357135779f1a00003329e94fd94268044f0900014080b0d0e03af7333333333333333333333333333333333333333333333333333333333333333300003329e95f1b8232c4b309cfb846bfd9190369e2fcfa5ccdd95e2f45b7b38b8aec3c2b9959c33500003329e95f1b8168044f0900014080f10110001200205303024121300827224cb63f6ca8a66c7ab33a3b4991469c85f92425674a7b4be42a90f3957c85975bda80fa2e3ca3e879ef424b3717e3aa43fa20e681bee924d0cb99760bf62836d020520302414130101a015008272bda80fa2e3ca3e879ef424b3717e3aa43fa20e681bee924d0cb99760bf62836d44e7eab684e87b6634d5f11a016f513b7abbe1d362dafc625d935458d9e9cb4f020f040928f1d4b01811161300a046f05010b0760000000000000000004400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005bc00000000000000000000000012d452da449e50b8cf7dd27861f146122afe1b546bb8b70fc8216f0c614139f8e0400a043019010b076000000000000000000880000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000ab69fe00000000000000000000000000000000000000000000000000000000000000013fccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccd28f1d4b000000006653d2be3700d0089e124000a042af7010b0760000000000000000006400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
                               //         let cell = TonCell::from_boc_hex(boc_hex)?;
                               //         let expected_cell_hash = TonHash::from_str("8cb00e78c699d6e580f469568980ea0f21a943af1beb71d82f66cdc4ebffff54")?;
                               //         assert_eq!(cell.hash(), &expected_cell_hash);
                               //         let shard_accounts_blocks = ShardAccountBlocks::from_boc_hex(boc_hex)?;
                               //         assert_eq!(shard_accounts_blocks.cell_hash()?, expected_cell_hash);
                               //         Ok(())
                               //     }
                               // }
