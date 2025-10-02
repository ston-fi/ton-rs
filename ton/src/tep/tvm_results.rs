// It would be better to put it somewhere else (in contracts folder for example),
// But then we stack with feature-flag to make it available
mod get_collection_data_result;
mod get_jetton_data_result;
mod get_nft_address_by_index_result;
mod get_nft_content_result;
mod get_nft_data_result;
mod get_wallet_address_result;
mod get_wallet_data_result;
mod tvm_result;

pub use get_collection_data_result::*;
pub use get_jetton_data_result::*;
pub use get_nft_address_by_index_result::*;
pub use get_nft_content_result::*;
pub use get_nft_data_result::*;
pub use get_wallet_address_result::*;
pub use get_wallet_data_result::*;
pub use tvm_result::*;
