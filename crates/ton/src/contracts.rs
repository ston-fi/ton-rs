//! Contract state loading and provider-neutral TVM get-method execution.

mod contract_client;
pub mod tep;
mod ton_contract;
mod tvm_get_method;

pub use contract_client::*;
pub use ton_contract::*;
pub use tvm_get_method::*;

#[cfg(test)]
mod tests {
    use crate::block_tlb::{FromTVMStack, TVMStack};
    use crate::contracts::TVMGetMethodID;
    use crate::contracts::TonContract;
    use crate::errors::TonResult;
    use crate::ton_contract;
    use ton_macros::{TLB, ton_methods};

    struct MethodRecorder {
        expected_name: &'static str,
    }

    #[async_trait::async_trait]
    trait RecordsMethod: Send + Sync {
        async fn emulate_get_method<M, T: FromTVMStack>(&self, method: M, _stack: &TVMStack) -> TonResult<T>
        where
            M: Into<TVMGetMethodID> + Send,
        {
            assert_eq!(method.into().as_str(), self.expected_name());
            let mut result = TVMStack::default();
            result.push_tiny_int(7);
            T::from_stack(&mut result)
        }

        fn expected_name(&self) -> &'static str;
    }

    #[async_trait::async_trait]
    impl RecordsMethod for MethodRecorder {
        fn expected_name(&self) -> &'static str {
            self.expected_name
        }
    }

    #[async_trait::async_trait]
    #[ton_methods(name_format = "camelCase")]
    trait ExactNameTraitMethods: RecordsMethod {
        #[ton_method(name = "getUIVariables")]
        async fn get_ui_variables(&self) -> TonResult<u32>;
    }

    impl ExactNameTraitMethods for MethodRecorder {}

    #[ton_methods(name_format = "camelCase")]
    impl MethodRecorder {
        #[ton_method(name = "getUIVariables")]
        async fn get_ui_variables_from_impl(&self) -> TonResult<u32>;
    }

    #[test]
    #[allow(unused)] // we just check it compiles
    fn test_ton_methods_name_format_camel_case_compiles() {
        #[derive(TLB)]
        pub struct OrderContractData;

        #[async_trait::async_trait]
        #[ton_methods(name_format = "camelCase")]
        trait OrderContractMethods: TonContract {
            async fn get_order_data(&self) -> TonResult<u32>;
        }

        ton_contract!(OrderContract<OrderContractData>: OrderContractMethods);
    }

    #[tokio::test]
    async fn test_ton_method_exact_name_overrides_format() -> anyhow::Result<()> {
        let contract = MethodRecorder {
            expected_name: "getUIVariables",
        };

        assert_eq!(ExactNameTraitMethods::get_ui_variables(&contract).await?, 7);
        assert_eq!(contract.get_ui_variables_from_impl().await?, 7);
        Ok(())
    }
}
