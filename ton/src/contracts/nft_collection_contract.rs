use crate::contracts::{
    methods::get_collection_data::GetCollectionData, methods::get_nft_address_by_index::GetNFTAddressByIndex,
    methods::get_nft_content::GetNFTContent, ton_contract::ContractCtx,
};
use ton_lib_core::ton_contract;

#[ton_contract]
pub struct NFTCollectionContract;
impl GetNFTContent for NFTCollectionContract {}
impl GetCollectionData for NFTCollectionContract {}
impl GetNFTAddressByIndex for NFTCollectionContract {}
