mod tlb_derive;
mod tlb_derive_enum;
mod tlb_derive_struct;
mod ton_methods;
mod from_tvm_stack_derive;
mod utils;

use crate::ton_methods::ton_methods_impl;
use crate::{
    tlb_derive::{TLBHeaderAttrs, tlb_derive_impl},
    from_tvm_stack_derive::from_tvm_stack_derive_impl,
};
use proc_macro::TokenStream;

/// Automatic `TLB` implementation
#[proc_macro_derive(TLB, attributes(tlb))]
pub fn tlb_derive(input: TokenStream) -> TokenStream { tlb_derive_impl(input).into() }

/// Automatic `FromTVMStack` implementation for POD types
#[proc_macro_derive(FromTVMStack, attributes(from_tvm_stack))]
pub fn from_tvm_stack_derive(input: TokenStream) -> TokenStream { from_tvm_stack_derive_impl(input).into() }

#[proc_macro_attribute]
pub fn ton_methods(attr: TokenStream, item: TokenStream) -> TokenStream { ton_methods_impl(attr, item) }
