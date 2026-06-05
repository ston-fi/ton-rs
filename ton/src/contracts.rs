mod contract_client;
mod contracts_impl;
mod ton_contract;

pub use contract_client::*;
pub use contracts_impl::*;
pub use ton_contract::*;

#[cfg(test)]
mod tests {
    use crate::contracts::TonContract;
    use crate::errors::TonResult;
    use crate::ton_contract;
    use ton_macros::{TLB, ton_methods};

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
}
