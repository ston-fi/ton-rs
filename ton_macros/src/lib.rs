mod tlb_derive;
mod tlb_derive_enum;
mod tlb_derive_struct;
mod tvm_type_derive;

use crate::{
    tlb_derive::{TLBHeaderAttrs, tlb_derive_impl},
    tvm_type_derive::tvm_type_derive_impl,
};
use proc_macro::TokenStream;

/// Automatic `TLB` implementation
#[proc_macro_derive(TLB, attributes(tlb))]
pub fn tlb_derive(input: TokenStream) -> TokenStream { tlb_derive_impl(input).into() }

/// Automatic `TVMType` implementation
#[proc_macro_derive(TVMType, attributes(tvm_type))]
pub fn tvm_result_derive(input: TokenStream) -> TokenStream { tvm_type_derive_impl(input).into() }
