use crate::emulators::emul_bc_config::EmulBCConfig;
use crate::emulators::emul_utils::{convert_emulator_response, make_b64_c_str, set_param_failed};
use crate::emulators::tx::tx_emul_args::{TXEmulArgs, TXEmulOrdArgs, TXEmulTickTockArgs};
use crate::emulators::tx::tx_emul_response::{TXEmulationResponse, TXEmulationSuccess};
use crate::errors::TonError;
use std::ffi::CString;
use std::sync::Arc;
use ton_lib_core::cell::TonHash;
use ton_lib_core::constants::TON_ZERO_CONFIG_BOC_B64;
use tonlib_sys::*;

pub struct TXEmulator {
    emulator: *mut std::ffi::c_void,
    cur_bc_config_hash: u64,
    cur_random_seed: TonHash,
    cur_utime: u32,
    cur_lt: u64,
    cur_libs_hash: u64,
    cur_ignore_chksig: bool,
    cur_prev_blocks_info_hash: u64,
}

impl TXEmulator {
    pub fn new(log_level: u32, debug_enabled: bool) -> Result<Self, TonError> {
        let zero_config = Arc::new(CString::new(TON_ZERO_CONFIG_BOC_B64)?);
        let ptr = unsafe { transaction_emulator_create(zero_config.as_ptr(), log_level) };
        if ptr.is_null() {
            return Err(TonError::EmulatorCreationFailed);
        }
        let mut emulator = Self {
            emulator: ptr,
            cur_bc_config_hash: calc_hash(zero_config.as_bytes()),
            cur_random_seed: Default::default(),
            cur_utime: 0,
            cur_lt: 0,
            cur_libs_hash: calc_hash([]),
            cur_ignore_chksig: false,
            cur_prev_blocks_info_hash: 0,
        };
        emulator.set_debug_enabled(debug_enabled)?;
        Ok(emulator)
    }

    /// shard_account: https://github.com/ton-blockchain/ton/blob/cee4c674ea999fecc072968677a34a7545ac9c4d/crypto/block/block.tlb#L275 (NOT Account!!)
    /// You can't emulate tick-tock tx using this method
    pub fn emulate_ord(&mut self, args: &TXEmulOrdArgs) -> Result<TXEmulationSuccess, TonError> {
        self.prepare_emulator(&args.emul_args)?;
        let state_c_str = make_b64_c_str(&args.emul_args.shard_account_boc)?;
        let in_msg_c_str = make_b64_c_str(&args.in_msg_boc)?;
        let response_ptr = unsafe {
            transaction_emulator_emulate_transaction(self.emulator, state_c_str.as_ptr(), in_msg_c_str.as_ptr())
        };
        let response_str = convert_emulator_response(response_ptr)?;
        TXEmulationResponse::from_json(response_str)?.into_success()
    }

    pub fn emulate_ticktock(&mut self, args: &TXEmulTickTockArgs) -> Result<TXEmulationSuccess, TonError> {
        self.prepare_emulator(&args.emul_args)?;
        let state_c_str = make_b64_c_str(&args.emul_args.shard_account_boc)?;
        let response_ptr = unsafe {
            transaction_emulator_emulate_tick_tock_transaction(self.emulator, state_c_str.as_ptr(), args.is_tock)
        };
        let response_str = convert_emulator_response(response_ptr)?;
        TXEmulationResponse::from_json(response_str)?.into_success()
    }

    fn prepare_emulator(&mut self, args: &TXEmulArgs) -> Result<(), TonError> {
        self.actualize_config(&args.bc_config)?;
        self.actualize_rand_seed(&args.rand_seed)?;
        self.actualize_utime(args.utime)?;
        self.actualize_lt(args.lt)?;
        if let Some(libs) = &args.libs_boc {
            self.actualize_libs(libs.as_ref())?;
        }
        self.actualize_ignore_chksig(args.ignore_chksig)?;
        if let Some(prev_blocks) = &args.prev_blocks_boc {
            self.actualize_prev_blocks_info(prev_blocks.as_ref())?;
        }
        Ok(())
    }

    fn actualize_config(&mut self, config: &EmulBCConfig) -> Result<(), TonError> {
        let config_hash = calc_hash(config.as_bytes());
        if self.cur_bc_config_hash == config_hash {
            return Ok(());
        }
        match unsafe { transaction_emulator_set_config(self.emulator, config.as_ptr()) } {
            true => self.cur_bc_config_hash = config_hash,
            false => return set_param_failed("config"),
        }
        Ok(())
    }

    fn actualize_rand_seed(&mut self, rand_seed: &TonHash) -> Result<(), TonError> {
        if self.cur_random_seed == *rand_seed {
            return Ok(());
        }
        match unsafe {
            transaction_emulator_set_rand_seed(self.emulator, CString::new(hex::encode(rand_seed))?.as_ptr())
        } {
            true => self.cur_random_seed = TonHash::from_slice_sized(rand_seed.as_slice_sized()),
            false => return set_param_failed("rand_seed"),
        }
        Ok(())
    }

    fn actualize_utime(&mut self, utime: u32) -> Result<(), TonError> {
        if self.cur_utime == utime {
            return Ok(());
        }
        match unsafe { transaction_emulator_set_unixtime(self.emulator, utime) } {
            true => self.cur_utime = utime,
            false => return set_param_failed("utime"),
        }
        Ok(())
    }

    fn actualize_lt(&mut self, lt: u64) -> Result<(), TonError> {
        if self.cur_lt == lt {
            return Ok(());
        }
        match unsafe { transaction_emulator_set_lt(self.emulator, lt) } {
            true => self.cur_lt = lt,
            false => return set_param_failed("lt"),
        }
        Ok(())
    }

    fn actualize_libs(&mut self, libs_boc: &[u8]) -> Result<(), TonError> {
        let libs_hash = calc_hash(libs_boc);
        if self.cur_libs_hash == libs_hash {
            return Ok(());
        }
        let libs = make_b64_c_str(libs_boc)?;
        match unsafe { transaction_emulator_set_libs(self.emulator, libs.as_ptr()) } {
            true => self.cur_libs_hash = libs_hash,
            false => return set_param_failed("libs"),
        }
        Ok(())
    }

    fn set_debug_enabled(&mut self, debug_enabled: bool) -> Result<(), TonError> {
        match unsafe { transaction_emulator_set_debug_enabled(self.emulator, debug_enabled) } {
            true => Ok(()),
            false => set_param_failed("debug_enabled"),
        }
    }

    fn actualize_prev_blocks_info(&mut self, prev_blocks_info: &[u8]) -> Result<(), TonError> {
        let prev_blocks_hash = calc_hash(prev_blocks_info);
        if self.cur_prev_blocks_info_hash == prev_blocks_hash {
            return Ok(());
        }
        let block_info = make_b64_c_str(prev_blocks_info)?;
        match unsafe { transaction_emulator_set_prev_blocks_info(self.emulator, block_info.as_ptr()) } {
            true => self.cur_prev_blocks_info_hash = prev_blocks_hash,
            false => return set_param_failed("prev_blocks_info"),
        }
        Ok(())
    }

    fn actualize_ignore_chksig(&mut self, ignore: bool) -> Result<(), TonError> {
        if self.cur_ignore_chksig == ignore {
            return Ok(());
        }
        match unsafe { transaction_emulator_set_ignore_chksig(self.emulator, ignore) } {
            true => self.cur_ignore_chksig = ignore,
            false => return set_param_failed("ignore_chksig"),
        }
        Ok(())
    }
}

impl Drop for TXEmulator {
    fn drop(&mut self) {
        unsafe {
            transaction_emulator_destroy(self.emulator);
        }
    }
}
unsafe impl Send for TXEmulator {}
unsafe impl Sync for TXEmulator {}

fn calc_hash<T: AsRef<[u8]> + std::hash::Hash>(data: T) -> u64 {
    use std::hash::{DefaultHasher, Hasher};
    let mut s = DefaultHasher::new();
    data.hash(&mut s);
    s.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_tlb::{Msg, ShardAccount, Tx};
    use crate::emulators::tx::tx_emul_args::TXEmulArgs;
    use crate::sys_utils::sys_tonlib_set_verbosity_level;
    use std::str::FromStr;
    use std::sync::LazyLock;
    use tokio_test::{assert_err, assert_ok};
    use ton_lib_core::traits::tlb::TLB;

    static BC_CONFIG: LazyLock<EmulBCConfig> = LazyLock::new(|| {
        EmulBCConfig::from_boc_hex(include_str!("../../../../resources/tests/bc_config_key_block_42123611.hex"))
            .unwrap()
    });

    #[test]
    fn test_tx_emulator_creation() {
        let emulator = TXEmulator::new(2, false);
        assert!(emulator.is_ok());
    }

    #[test]
    fn test_tx_emulator_emulate_ext_in_wallet() -> anyhow::Result<()> {
        sys_tonlib_set_verbosity_level(0);
        let mut emulator = TXEmulator::new(0, false)?;
        let shard_account = ShardAccount::from_boc_hex("b5ee9c720102170100036600015094fb2314023373e7b36b05b69e31508eba9ba24a60e994060fee1ca55302f8c2000030a4972bcd4301026fc0092eb9106ca20295132ce6170ece2338ba10342134a3ca0d9e499f21c9b4897e422c858e433ce5b6500000c2925caf351106c29d2a534002030114ff00f4a413f4bcf2c80b0400510000001129a9a317cbf377c9b73604c70bf73488ddceba14f763baef2ac70f68d1d6032a120149f4400201200506020148070804f8f28308d71820d31fd31fd31f02f823bbf264ed44d0d31fd31fd3fff404d15143baf2a15151baf2a205f901541064f910f2a3f80024a4c8cb1f5240cb1f5230cbff5210f400c9ed54f80f01d30721c0009f6c519320d74a96d307d402fb00e830e021c001e30021c002e30001c0039130e30d03a4c8cb1f12cb1fcbff090a0b0c02e6d001d0d3032171b0925f04e022d749c120925f04e002d31f218210706c7567bd22821064737472bdb0925f05e003fa403020fa4401c8ca07cbffc9d0ed44d0810140d721f404305c810108f40a6fa131b3925f07e005d33fc8258210706c7567ba923830e30d03821064737472ba925f06e30d0d0e0201200f10006ed207fa00d4d422f90005c8ca0715cbffc9d077748018c8cb05cb0222cf165005fa0214cb6b12ccccc973fb00c84014810108f451f2a7020070810108d718fa00d33fc8542047810108f451f2a782106e6f746570748018c8cb05cb025006cf165004fa0214cb6a12cb1fcb3fc973fb0002006c810108d718fa00d33f305224810108f459f2a782106473747270748018c8cb05cb025005cf165003fa0213cb6acb1f12cb3fc973fb00000af400c9ed54007801fa00f40430f8276f2230500aa121bef2e0508210706c7567831eb17080185004cb0526cf1658fa0219f400cb6917cb1f5260cb3f20c98040fb0006008a5004810108f45930ed44d0810140d720c801cf16f400c9ed540172b08e23821064737472831eb17080185005cb055003cf1623fa0213cb6acb1fcb3fc98040fb00925f03e202012011120059bd242b6f6a2684080a06b90fa0218470d4080847a4937d29910ce6903e9ff9837812801b7810148987159f318402015813140011b8c97ed44d0d70b1f8003db29dfb513420405035c87d010c00b23281f2fff274006040423d029be84c6002012015160019adce76a26840206b90eb85ffc00019af1df6a26840106b90eb858fc0")?;
        let ext_in_msg = Msg::from_boc_hex("b5ee9c72010204010001560001e1880125d7220d944052a2659cc2e1d9c4671742068426947941b3c933e43936912fc800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014d4d18bb3ce5c84000000088001c01016862004975c883aea91de93142ae4dc222d803c74e5f130f37ef0d42fb353897fd0f982068e77800000000000000000000000000010201b20f8a7ea500000000000000005012a05f20080129343398aec31cdbbf7d32d977c27a96d5cd23c38fd4bd47be019abafb9b356b0024bae441b2880a544cb3985c3b388ce2e840d084d28f283679267c8726d225f90814dc9381030099259385618012934339d11465553b2f3e428ae79b0b1e2fd250b80784d4996dd44741736528ca0259f3a0f90024bae441b2880a544cb3985c3b388ce2e840d084d28f283679267c8726d225f910")?;

        let mut ord_args = TXEmulOrdArgs {
            in_msg_boc: ext_in_msg.to_boc()?,
            emul_args: TXEmulArgs {
                shard_account_boc: shard_account.to_boc()?,
                bc_config: BC_CONFIG.clone(),
                rand_seed: TonHash::from_str("14857b338a5bf80a4c87e726846672173bb780f694c96c15084a3cbcc719ebf0")?,
                utime: 1738323935,
                lt: 53483578000001,
                ignore_chksig: false,
                prev_blocks_boc: None,
                libs_boc: None,
            },
        };
        assert_err!(emulator.emulate_ord(&ord_args));
        ord_args.emul_args.ignore_chksig = true;
        let response = assert_ok!(emulator.emulate_ord(&ord_args));
        assert!(response.success);

        let expected_tx = Tx::from_boc_hex("b5ee9c7241020c010002f50003b5792eb9106ca20295132ce6170ece2338ba10342134a3ca0d9e499f21c9b4897e4000030a49dab028194fb2314023373e7b36b05b69e31508eba9ba24a60e994060fee1ca55302f8c2000030a4972bcd43679cb7df00034657bf0280102030201e00405008272fb026ad92478055ab0086833e193b9e2ad35aa0073769228fcdc27ed38ef72a4c533ffcf55fd97275de407b0068404ed61966be66ec1e82d6c49d100f01e6064020f0c51c618a18604400a0b01e1880125d7220d944052a2659cc2e1d9c4671742068426947941b3c933e43936912fc800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000014d4d18bb3ce5c84000000088001c060101df07016862004975c883aea91de93142ae4dc222d803c74e5f130f37ef0d42fb353897fd0f982068e77800000000000000000000000000010801b1680125d7220d944052a2659cc2e1d9c4671742068426947941b3c933e43936912fc90024bae441d7548ef498a15726e1116c01e3a72f89879bf786a17d9a9c4bfe87cc103473bc000614884c000061493b560504cf396fbec00801b20f8a7ea500000000000000005012a05f20080129343398aec31cdbbf7d32d977c27a96d5cd23c38fd4bd47be019abafb9b356b0024bae441b2880a544cb3985c3b388ce2e840d084d28f283679267c8726d225f90814dc9381090099259385618012934339d11465553b2f3e428ae79b0b1e2fd250b80784d4996dd44741736528ca0259f3a0f90024bae441b2880a544cb3985c3b388ce2e840d084d28f283679267c8726d225f910009d419d8313880000000000000000110000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000020006fc987b3184c14882800000000000200000000000224cb2890dee94c80761e06b8c446b1a9835aff2fc055cee75373ceeceffa6b4240d03f644db9e7b3")?;
        let expected_shard_account = ShardAccount::from_boc_hex("b5ee9c7241021701000366000150775a15d6954e05b73e0c25729e776e6be6328ed14ebaf7262014603827198d24000030a49dab028101026fc0092eb9106ca20295132ce6170ece2338ba10342134a3ca0d9e499f21c9b4897e422c858e433ce5bef80000c29276ac0a0d036dd880934002030114ff00f4a413f4bcf2c80b0400510000001229a9a317cbf377c9b73604c70bf73488ddceba14f763baef2ac70f68d1d6032a120149f4400201200506020148070804f8f28308d71820d31fd31fd31f02f823bbf264ed44d0d31fd31fd3fff404d15143baf2a15151baf2a205f901541064f910f2a3f80024a4c8cb1f5240cb1f5230cbff5210f400c9ed54f80f01d30721c0009f6c519320d74a96d307d402fb00e830e021c001e30021c002e30001c0039130e30d03a4c8cb1f12cb1fcbff1314151602e6d001d0d3032171b0925f04e022d749c120925f04e002d31f218210706c7567bd22821064737472bdb0925f05e003fa403020fa4401c8ca07cbffc9d0ed44d0810140d721f404305c810108f40a6fa131b3925f07e005d33fc8258210706c7567ba923830e30d03821064737472ba925f06e30d090a0201200b0c007801fa00f40430f8276f2230500aa121bef2e0508210706c7567831eb17080185004cb0526cf1658fa0219f400cb6917cb1f5260cb3f20c98040fb0006008a5004810108f45930ed44d0810140d720c801cf16f400c9ed540172b08e23821064737472831eb17080185005cb055003cf1623fa0213cb6acb1fcb3fc98040fb00925f03e20201200d0e0059bd242b6f6a2684080a06b90fa0218470d4080847a4937d29910ce6903e9ff9837812801b7810148987159f31840201580f100011b8c97ed44d0d70b1f8003db29dfb513420405035c87d010c00b23281f2fff274006040423d029be84c6002012011120019adce76a26840206b90eb85ffc00019af1df6a26840106b90eb858fc0006ed207fa00d4d422f90005c8ca0715cbffc9d077748018c8cb05cb0222cf165005fa0214cb6b12ccccc973fb00c84014810108f451f2a7020070810108d718fa00d33fc8542047810108f451f2a782106e6f746570748018c8cb05cb025006cf165004fa0214cb6a12cb1fcb3fc973fb0002006c810108d718fa00d33f305224810108f459f2a782106473747270748018c8cb05cb025005cf165003fa0213cb6acb1f12cb3fc973fb00000af400c9ed5494cb980d")?;
        assert_eq!(response.shard_account_parsed()?, expected_shard_account);
        assert_eq!(response.tx_parsed()?, expected_tx);
        Ok(())
    }
}
