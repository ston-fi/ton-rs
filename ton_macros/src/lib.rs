mod tlb_derive;
mod tlb_derive_enum;
mod tlb_derive_struct;
mod ton_contract;

use crate::tlb_derive::{tlb_derive_impl, TLBHeaderAttrs};
use crate::ton_contract::ton_contract_impl;
use proc_macro::TokenStream;

/// Automatic `TLB` implementation
#[proc_macro_derive(TLB, attributes(tlb))]
pub fn tlb_derive(input: TokenStream) -> TokenStream { tlb_derive_impl(input).into() }

/// Automatic `TonContract` implementation
#[proc_macro_attribute]
pub fn ton_contract(_attr: TokenStream, item: TokenStream) -> TokenStream { ton_contract_impl(_attr, item) }
