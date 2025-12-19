mod tlb_derive;
mod tlb_derive_enum;
mod tlb_derive_struct;
mod tvm_type_derive;
mod ton_method;
mod utils;

use crate::{
    tlb_derive::{TLBHeaderAttrs, tlb_derive_impl},
    tvm_type_derive::tvm_type_derive_impl,
};
use proc_macro::TokenStream;
use crate::ton_method::ton_method_impl;

/// Automatic `TLB` implementation
#[proc_macro_derive(TLB, attributes(tlb))]
pub fn tlb_derive(input: TokenStream) -> TokenStream { tlb_derive_impl(input).into() }

/// Automatic `TVMType` implementation
#[proc_macro_derive(TVMType, attributes(tvm_type))]
pub fn tvm_result_derive(input: TokenStream) -> TokenStream { tvm_type_derive_impl(input).into() }


#[proc_macro_attribute]
pub fn ton_method(attr: TokenStream, item: TokenStream) -> TokenStream { ton_method_impl(attr, item).into() }
