use crate::emulators::tx_emulator::TXEmulTickTockArgs;
use crate::emulators::tx_emulator::{TXEmulOrdArgs, TXEmulationResponse, TXEmulator};
use crate::errors::TonResult;
use crate::thread_pool::{PoolObject, ThreadPool};

pub type TXEmulatorPool = ThreadPool<TXEmulator>;

impl PoolObject for TXEmulator {
    type Task = TXEmulTask;
    type Retval = TXEmulationResponse;
    fn process<T: Into<Self::Task>>(&mut self, task: T) -> TonResult<TXEmulationResponse> {
        match task.into() {
            TXEmulTask::TXOrd(args) => self.emulate_ord(&args),
            TXEmulTask::TXTickTock(args) => self.emulate_ticktock(&args),
        }
    }
    fn descriptor(&self) -> &str { "TXEmulator" }
}

#[derive(Clone, Debug)]
pub enum TXEmulTask {
    TXOrd(TXEmulOrdArgs),
    TXTickTock(TXEmulTickTockArgs),
}

impl From<TXEmulOrdArgs> for TXEmulTask {
    fn from(args: TXEmulOrdArgs) -> Self { TXEmulTask::TXOrd(args) }
}
impl From<TXEmulTickTockArgs> for TXEmulTask {
    fn from(args: TXEmulTickTockArgs) -> Self { TXEmulTask::TXTickTock(args) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_tlb::{Msg, ShardAccount};
    use crate::emulators::emul_bc_config::EmulBCConfig;
    use crate::emulators::tx_emulator::TXEmulArgs;
    use crate::sys_utils::sys_tonlib_set_verbosity_level;
    use std::str::FromStr;
    use std::sync::Arc;
    use tokio_test::assert_ok;
    use ton_core::cell::TonHash;
    use ton_core::traits::tlb::TLB;

    #[tokio::test]
    async fn test_tx_emulator_pool() -> anyhow::Result<()> {
        sys_tonlib_set_verbosity_level(0);
        let objects = vec![TXEmulator::new(0, false)?, TXEmulator::new(0, false)?];
        let pool = TXEmulatorPool::builder(objects)?.build()?;

        let shard_account = ShardAccount::from_boc_hex(
            "b5ee9c720102170100036600015094fb2314023373e7b36b05b69e31508eba9ba24a60e994060fee1ca55302f8c2000030a4972bcd4301026fc0092eb9106ca20295132ce6170ece2338ba10342134a3ca0d9e499f21c9b4897e422c858e433ce5b6500000c2925caf351106c29d2a534002030114ff00f4a413f4bcf2c80b0400510000001129a9a317cbf377c9b73604c70bf73488ddceba14f763baef2ac70f68d1d6032a120149f4400201200506020148070804f8f28308d71820d31fd31fd31f02f823bbf264ed44d0d31fd31fd3fff404d15143baf2a15151baf2a205f901541064f910f2a3f80024a4c8cb1f5240cb1f5230cbff5210f400c9ed54f80f01d30721c0009f6c519320d74a96d307d402fb00e830e021c001e30021c002e30001c0039130e30d03a4c8cb1f12cb1fcbff090a0b0c02e6d001d0d3032171b0925f04e022d749c120925f04e002d31f218210706c7567bd22821064737472bdb0925f05e003fa403020fa4401c8ca07cbffc9d0ed44d0810140d721f404305c810108f40a6fa131b3925f07e005d33fc8258210706c7567ba923830e30d03821064737472ba925f06e30d0d0e0201200f10006ed207fa00d4d422f90005c8ca0715cbffc9d077748018c8cb05cb0222cf165005fa0214cb6b12ccccc973fb00c84014810108f451f2a7020070810108d718fa00d33fc8542047810108f451f2a782106e6f746570748018c8cb05cb025006cf165004fa0214cb6a12cb1fcb3fc973fb0002006c810108d718fa00d33f305224810108f459f2a782106473747270748018c8cb05cb025005cf165003fa0213cb6acb1f12cb3fc973fb00000af400c9ed54007801fa00f40430f8276f2230500aa121bef2e0508210706c7567831eb17080185004cb0526cf1658fa0219f400cb6917cb1f5260cb3f20c98040fb0006008a5004810108f45930ed44d0810140d720c801cf16f400c9ed540172b08e23821064737472831eb17080185005cb055003cf1623fa0213cb6acb1fcb3fc98040fb00925f03e202012011120059bd242b6f6a2684080a06b90fa0218470d4080847a4937d29910ce6903e9ff9837812801b7810148987159f318402015813140011b8c97ed44d0d70b1f8003db29dfb513420405035c87d010c00b23281f2fff274006040423d029be84c6002012015160019adce76a26840206b90eb85ffc00019af1df6a26840106b90eb858fc0",
        )?;

        let ext_in_msg = Msg::from_boc_hex(
            "b5ee9c72010204010001560001e1880125d7220d944052a2659cc2e1d9c4671742068426947941b3c933e43936912fc800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014d4d18bb3ce5c84000000088001c01016862004975c883aea91de93142ae4dc222d803c74e5f130f37ef0d42fb353897fd0f982068e77800000000000000000000000000010201b20f8a7ea500000000000000005012a05f20080129343398aec31cdbbf7d32d977c27a96d5cd23c38fd4bd47be019abafb9b356b0024bae441b2880a544cb3985c3b388ce2e840d084d28f283679267c8726d225f90814dc9381030099259385618012934339d11465553b2f3e428ae79b0b1e2fd250b80784d4996dd44741736528ca0259f3a0f90024bae441b2880a544cb3985c3b388ce2e840d084d28f283679267c8726d225f910",
        )?;

        let emul_args = TXEmulArgs {
            shard_account_boc: Arc::new(shard_account.to_boc()?),
            bc_config: EmulBCConfig::from_boc_hex(include_str!(
                "../../../resources/tests/bc_config_key_block_42123611.hex"
            ))?,
            rand_seed: TonHash::from_str("14857b338a5bf80a4c87e726846672173bb780f694c96c15084a3cbcc719ebf0")?,
            utime: 1738323935,
            lt: 53483578000001,
            ignore_chksig: true,
            prev_blocks_boc: None,
            libs_boc: None,
        };
        let ord_args = TXEmulOrdArgs {
            in_msg_boc: Arc::new(ext_in_msg.to_boc()?),
            emul_args: emul_args.clone(),
        };
        let response = assert_ok!(pool.execute(ord_args.clone(), None).await);
        let success = assert_ok!(response.into_success());
        assert!(success.success);
        Ok(())
    }
}
