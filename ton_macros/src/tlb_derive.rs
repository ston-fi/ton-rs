use crate::tlb_derive_enum::tlb_derive_enum;
use crate::tlb_derive_struct::tlb_derive_struct;
use proc_macro2::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote};
use syn::Data;

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(tlb))]
pub(crate) struct TLBHeaderAttrs {
    pub(crate) prefix: Option<usize>,      // use 0 as default
    pub(crate) bits_len: Option<usize>,    // use 0 as default
    pub(crate) ensure_empty: Option<bool>, // use false as default
}

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(tlb))]
pub(crate) struct TLBFieldAttrs {
    pub(crate) bits_len: Option<u32>, // alias for ConstLen adapter
    pub(crate) adapter: Option<String>,
}

pub(crate) fn tlb_derive_impl(input: proc_macro::TokenStream) -> TokenStream {
    let mut input = syn::parse::<syn::DeriveInput>(input).unwrap();
    // Extract a description, modifying `input.attrs` to remove the matched attributes.
    let header_attrs: TLBHeaderAttrs = match deluxe::extract_attributes(&mut input) {
        Ok(desc) => desc,
        Err(e) => return e.into_compile_error(),
    };

    let crate_path = if let Ok(ton_lib_core_crate) = crate_name("ton_lib_core") {
        match ton_lib_core_crate {
            FoundCrate::Itself => quote::quote! { crate },
            FoundCrate::Name(name) => {
                let ident = format_ident!("{name}");
                quote! { #ident }
            }
        }
    } else if let Ok(ton_lib_crate) = crate_name("ton_lib") {
        match ton_lib_crate {
            FoundCrate::Itself => quote::quote! { crate::ton_lib_core },
            FoundCrate::Name(name) => {
                let ident = format_ident!("{name}");
                quote! { #ident::ton_lib_core }
            }
        }
    } else {
        panic!("Can't find ton_core or ton crate");
    };

    let ident = &input.ident;

    let (read_def_tokens, write_def_tokens, extra_impl_tokens) = match &mut input.data {
        Data::Struct(data) => tlb_derive_struct(&header_attrs, data),
        Data::Enum(data) => tlb_derive_enum(&crate_path, ident, data),
        _ => panic!("TLB derive macros only supports structs and enums"),
    };

    let prefix_val = header_attrs.prefix.unwrap_or(0);
    let prefix_bits_len = header_attrs.bits_len.unwrap_or(0);

    quote::quote! {
        impl #crate_path::traits::tlb::TLB for #ident {
            const PREFIX: #crate_path::traits::tlb::TLBPrefix = #crate_path::traits::tlb::TLBPrefix::new(#prefix_val, #prefix_bits_len);

            fn read_definition(parser: &mut #crate_path::cell::CellParser) -> Result<Self, #crate_path::errors::TonCoreError> {
                use #crate_path::traits::tlb::TLB;

                #read_def_tokens
            }

            fn write_definition(&self, builder: &mut #crate_path::cell::CellBuilder) -> Result<(), #crate_path::errors::TonCoreError> {
                #write_def_tokens
            }
        }

        #extra_impl_tokens
    }
}
