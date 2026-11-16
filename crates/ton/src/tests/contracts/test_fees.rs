use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;

use async_trait::async_trait;
use fastnum::I512;
use ton_core::cell::{TonCell, TonHash};
use ton_core::errors::TonCoreError;
use ton_core::traits::emulation_provider::{
    EmulationProvider, EmulatorContractState, EmulatorGetMethodRequest, EmulatorGetMethodSuccess,
};
use ton_core::traits::state_provider::{ContractState, StateProvider};
use ton_core::traits::tlb::TLB;
use ton_core::types::{Coins, TonAddress, TxLTHash};

use super::calculate_forward_fee;
use crate::block_tlb::TVMStack;
use crate::contracts::ContractClient;
use crate::errors::TonError;

struct NoStateReads;
#[async_trait]
impl StateProvider for NoStateReads {
    async fn last_mc_seqno(&self) -> Result<u32, TonCoreError> {
        panic!("fee quote must not read state")
    }
    async fn load_state(&self, _: TonAddress, _: Option<TxLTHash>) -> Result<ContractState, TonCoreError> {
        panic!("fee quote must not read state")
    }
    async fn load_latest_tx_per_address(&self, _: u32) -> Result<Vec<(TonAddress, TxLTHash)>, TonCoreError> {
        panic!("fee quote must not read state")
    }
}

enum Reply {
    Stack(TVMStack),
    Bytes(Vec<u8>),
    Vm(i32),
    Provider,
    Timeout,
}
struct ResponseProvider {
    expected: TVMStack,
    reply: Reply,
    calls: Arc<AtomicUsize>,
}
const TIMEOUT: Duration = Duration::from_secs(7);
#[async_trait]
impl EmulationProvider for ResponseProvider {
    fn requires_resolved_state(&self) -> bool {
        true
    }
    async fn emulate_get_method(
        &self,
        request: EmulatorGetMethodRequest,
        timeout: Option<Duration>,
    ) -> Result<EmulatorGetMethodSuccess, TonCoreError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(timeout, Some(TIMEOUT));
        assert_eq!(request.method_id, 0);
        let EmulatorContractState::Custom(state) = request.contract_state else {
            panic!("fee code must use supplied state")
        };
        assert_eq!(state.mc_seqno, None);
        let code = TonCell::from_boc(state.code_boc.as_ref().unwrap().clone())?;
        let mut parser = code.parser();
        assert_eq!(parser.read_num::<u32>(24)?, 0x30f838);
        parser.ensure_empty()?;
        assert_eq!(TVMStack::from_boc(request.stack_boc)?, self.expected);
        match &self.reply {
            Reply::Stack(stack) => Ok(EmulatorGetMethodSuccess::new(0, stack.to_boc()?)),
            Reply::Bytes(bytes) => Ok(EmulatorGetMethodSuccess::new(0, bytes.clone())),
            Reply::Vm(code) => Ok(EmulatorGetMethodSuccess::new(*code, vec![])),
            Reply::Provider => Err(TonCoreError::BoxedError(Box::new(TonError::EmulatorMissingLibrary(TonHash::ZERO)))),
            Reply::Timeout => Err(TonCoreError::BoxedError(Box::new(TonError::EmulatorTimeout(TIMEOUT)))),
        }
    }
}
fn input(cells: i64, bits: i64, masterchain: bool) -> TVMStack {
    let mut stack = TVMStack::default();
    for value in [cells, bits, if masterchain { -1 } else { 0 }] {
        stack.push_tiny_int(value);
    }
    stack
}
fn result(value: I512) -> TVMStack {
    let mut stack = TVMStack::default();
    stack.push_int(value);
    stack
}
fn client(expected: TVMStack, reply: Reply) -> (ContractClient, Arc<AtomicUsize>) {
    let calls = Arc::new(AtomicUsize::new(0));
    let client = ContractClient::builder(
        NoStateReads,
        ResponseProvider {
            expected,
            reply,
            calls: calls.clone(),
        },
    )
    .unwrap()
    .with_tvm_emulation_timeout(TIMEOUT)
    .build()
    .unwrap();
    (client, calls)
}

#[tokio::test]
async fn packs_chain_and_full_size_range_without_state_reads() {
    for (cells, bits, chain) in [(0, 0, false), (2, 787, true), (i64::MAX, i64::MAX, false)] {
        let (client, calls) =
            client(input(cells, bits, chain), Reply::Stack(result(I512::from_u128(u128::MAX).unwrap())));
        assert_eq!(
            calculate_forward_fee(&client, cells as u64, bits as u64, chain).await.unwrap(),
            Coins::new(u128::MAX)
        );
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
#[tokio::test]
async fn oversized_inputs_fail_before_emulation() {
    for (cells, bits) in [(i64::MAX as u64 + 1, 0), (0, u64::MAX)] {
        let (client, calls) = client(TVMStack::default(), Reply::Provider);
        assert!(matches!(
            calculate_forward_fee(&client, cells, bits, false).await,
            Err(TonError::UnexpectedValue { .. })
        ));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }
}
#[tokio::test]
async fn rejects_invalid_and_extra_results() {
    let mut extra = result(I512::ONE);
    extra.push_tiny_int(2);
    let mut wrong = TVMStack::default();
    wrong.push_cell(TonCell::empty().clone());
    for reply in [
        Reply::Stack(TVMStack::default()),
        Reply::Stack(result(-I512::ONE)),
        Reply::Stack(result(I512::ONE << 128)),
        Reply::Stack(extra),
        Reply::Stack(wrong),
        Reply::Bytes(vec![0xff]),
    ] {
        let (client, _) = client(input(0, 0, false), reply);
        assert!(calculate_forward_fee(&client, 0, 0, false).await.is_err());
    }
}
#[tokio::test]
async fn preserves_vm_provider_and_timeout_errors() {
    let (client, _) = client(input(0, 0, false), Reply::Vm(6));
    assert!(matches!(
        calculate_forward_fee(&client, 0, 0, false).await,
        Err(TonError::EmulatorEmulationError {
            vm_exit_code: Some(6),
            ..
        })
    ));
    let (client, _) = self::client(input(0, 0, false), Reply::Provider);
    assert!(
        matches!(calculate_forward_fee(&client, 0, 0, false).await, Err(TonError::EmulatorMissingLibrary(hash)) if hash == TonHash::ZERO)
    );
    let (client, _) = self::client(input(0, 0, false), Reply::Timeout);
    assert!(
        matches!(calculate_forward_fee(&client, 0, 0, false).await, Err(TonError::EmulatorTimeout(value)) if value == TIMEOUT)
    );
}

#[cfg(feature = "tonlibjson")]
mod native {
    use super::*;
    use crate::emulators::emul_bc_config::EmulBCConfig;
    use crate::emulators::tvm_emulator::{TVMEmulator, TVMEmulatorC7};
    use crate::tlb_adapters::{DictKeyAdapterUint, DictValAdapterTLB, TLBHashMap};
    use std::collections::HashMap;
    use ton_core::types::tlb_core::TLBRef;

    struct NativeProvider(EmulBCConfig);
    #[async_trait]
    impl EmulationProvider for NativeProvider {
        async fn emulate_get_method(
            &self,
            request: EmulatorGetMethodRequest,
            _: Option<Duration>,
        ) -> Result<EmulatorGetMethodSuccess, TonCoreError> {
            let EmulatorContractState::Custom(state) = request.contract_state else {
                panic!("expected supplied fee code")
            };
            let run = || -> crate::errors::TonResult<_> {
                let c7 = TVMEmulatorC7::new(state.address, self.0.clone())?;
                let mut emulator =
                    TVMEmulator::new(state.code_boc.as_ref().unwrap(), state.data_boc.as_ref().unwrap(), &c7)?;
                let result = emulator.emul_get_method(request.method_id, &request.stack_boc)?.into_success()?;
                Ok(EmulatorGetMethodSuccess::new(result.vm_exit_code, result.stack_boc()?))
            };
            run().map_err(|e| TonCoreError::BoxedError(Box::new(e)))
        }
    }
    // Independent block.tlb msg_forward_prices#ea fixture; no production codec is added.
    fn prices(lump: u64, bit: u64, cell: u64, first_frac: u16) -> TonCell {
        let mut b = TonCell::builder();
        b.write_num(&0xea_u8, 8).unwrap();
        for v in [lump, bit, cell] {
            b.write_num(&v, 64).unwrap();
        }
        b.write_num(&0_u32, 32).unwrap();
        b.write_num(&first_frac, 16).unwrap();
        b.write_num(&0_u16, 16).unwrap();
        b.build().unwrap()
    }
    fn config(first_frac: u16) -> EmulBCConfig {
        let raw =
            TonCell::from_boc_hex(include_str!("../../../resources/tests/bc_config_key_block_42123611.hex")).unwrap();
        let adapter = TLBHashMap::<DictKeyAdapterUint<u32>, DictValAdapterTLB<TLBRef<TonCell>>>::new(32);
        let mut dict: HashMap<u32, TLBRef<TonCell>> = adapter.read(&mut raw.parser()).unwrap();
        dict.insert(25, TLBRef::new(prices(17, 3, 5, first_frac)));
        dict.insert(24, TLBRef::new(prices(29, 7, 11, first_frac)));
        let mut b = TonCell::builder();
        adapter.write(&mut b, &dict).unwrap();
        EmulBCConfig::from_boc(&b.build().unwrap().to_boc().unwrap()).unwrap()
    }
    #[tokio::test]
    async fn native_missing_prices_remain_an_error() {
        crate::sys_utils::sys_tonlib_set_verbosity_level(0);
        let raw = TonCell::from_boc(config(0).to_boc().unwrap()).unwrap();
        let adapter = TLBHashMap::<DictKeyAdapterUint<u32>, DictValAdapterTLB<TLBRef<TonCell>>>::new(32);
        let mut dict = adapter.read(&mut raw.parser()).unwrap();
        dict.remove(&25);
        let mut builder = TonCell::builder();
        adapter.write(&mut builder, &dict).unwrap();
        let config = EmulBCConfig::from_boc(&builder.build().unwrap().to_boc().unwrap()).unwrap();
        let client = ContractClient::builder(NoStateReads, NativeProvider(config)).unwrap().build().unwrap();
        assert!(calculate_forward_fee(&client, 0, 0, false).await.is_err());
    }
    #[tokio::test]
    async fn native_prices_round_up_and_do_not_deduct_action_fee() {
        crate::sys_utils::sys_tonlib_set_verbosity_level(0);
        for fraction in [0, 21845, 32768] {
            let client =
                ContractClient::builder(NoStateReads, NativeProvider(config(fraction))).unwrap().build().unwrap();
            for master in [false, true] {
                let (lump, bit_price, cell_price) = if master { (29_u128, 7_u128, 11_u128) } else { (17, 3, 5) };
                for (cells, bits) in [(0_u64, 0_u64), (1, 1), (2, 787), (i64::MAX as u64, i64::MAX as u64)] {
                    let expected =
                        lump + (u128::from(cells) * cell_price + u128::from(bits) * bit_price).div_ceil(65536);
                    assert_eq!(
                        calculate_forward_fee(&client, cells, bits, master).await.unwrap(),
                        Coins::new(expected)
                    );
                }
            }
        }
    }
}
