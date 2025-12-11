mod tlb_derive;
mod tlb_derive_enum;
mod tlb_derive_struct;
mod tvm_result_derive;

use crate::{
    tlb_derive::{TLBHeaderAttrs, tlb_derive_impl},
    tvm_result_derive::tvm_result_derive_impl,
};
use proc_macro::TokenStream;

/// Automatic `TLB` implementation
#[proc_macro_derive(TLB, attributes(tlb))]
pub fn tlb_derive(input: TokenStream) -> TokenStream { tlb_derive_impl(input).into() }

#[proc_macro_derive(TVMResult, attributes(tvm_result))]
pub fn tvm_result_derive(input: TokenStream) -> TokenStream { tvm_result_derive_impl(input).into() }
