mod tlb_derive;
mod tlb_derive_enum;
mod tlb_derive_struct;

use crate::tlb_derive::{tlb_derive_impl, TLBHeaderAttrs};
use proc_macro::TokenStream;

/// Automatic `TLB` implementation
#[proc_macro_derive(TLB, attributes(tlb))]
pub fn tlb_derive(input: TokenStream) -> TokenStream { tlb_derive_impl(input).into() }
