use crate::block_tlb::{FromTVMStack, TVMStack};
use crate::contracts::TVMGetMethodID;
use crate::errors::{TonError, TonResult};
use std::sync::Mutex;
use ton_core::traits::tlb::TLB;
use ton_macros::ton_methods;

#[derive(Default)]
struct Contract {
    calls: Mutex<Vec<(TVMGetMethodID, TVMStack)>>,
}
impl Contract {
    async fn emulate_get_method<M: Into<TVMGetMethodID>, T: FromTVMStack>(
        &self,
        method: M,
        stack: &TVMStack,
    ) -> TonResult<T> {
        self.calls
            .lock()
            .map_err(|_| TonError::Custom("test recording lock".into()))?
            .push((method.into(), TVMStack::from_boc(stack.to_boc()?)?));
        let mut reply = TVMStack::default();
        reply.push_tiny_int(7);
        T::from_stack(&mut reply)
    }
}

#[ton_methods(name_format = "camelCase")]
impl Contract {
    #[ton_method(id = 109505)]
    async fn estimate_gas(&self, kind: u8, amount: u128) -> TonResult<u32>;
    async fn get_version(&self) -> TonResult<u32>;
    #[ton_method(name = "getUIVariables")]
    async fn get_ui_variables(&self) -> TonResult<u32>;
}

#[async_trait::async_trait]
trait Emulator: Send + Sync {
    async fn emulate_get_method<M: Into<TVMGetMethodID> + Send, T: FromTVMStack>(
        &self,
        method: M,
        stack: &TVMStack,
    ) -> TonResult<T>;
}
#[async_trait::async_trait]
impl Emulator for Contract {
    async fn emulate_get_method<M: Into<TVMGetMethodID> + Send, T: FromTVMStack>(
        &self,
        method: M,
        stack: &TVMStack,
    ) -> TonResult<T> {
        Contract::emulate_get_method(self, method, stack).await
    }
}
#[async_trait::async_trait]
#[ton_methods(name_format = "camelCase")]
trait Methods: Emulator {
    #[ton_method(id = -1,)]
    async fn numeric_entry(&self) -> TonResult<u32>;
}
impl Methods for Contract {}

#[test]
fn test_numeric_selectors_bypass_name_format_and_preserve_argument_order() -> anyhow::Result<()> {
    let contract = Contract::default();
    assert_eq!(tokio_test::block_on(contract.estimate_gas(u8::MAX, u128::MAX))?, 7);
    assert_eq!(tokio_test::block_on(contract.numeric_entry())?, 7);
    assert_eq!(tokio_test::block_on(contract.get_version())?, 7);
    assert_eq!(tokio_test::block_on(contract.get_ui_variables())?, 7);
    let mut calls = contract.calls.into_inner()?;
    assert_eq!(calls[0].0, TVMGetMethodID::Number(109505));
    assert_eq!(calls[0].1.pop_num()?, (fastnum::I512::ONE << 128u32) - fastnum::I512::ONE);
    assert_eq!(calls[0].1.pop_num()?, fastnum::I512::from(u8::MAX));
    calls[0].1.ensure_empty()?;
    assert_eq!(calls[1].0, TVMGetMethodID::Number(-1));
    assert!(calls[1].1.is_empty());
    assert_eq!(calls[2].0, TVMGetMethodID::from("getVersion"));
    assert_eq!(calls[3].0, TVMGetMethodID::from("getUIVariables"));
    Ok(())
}
