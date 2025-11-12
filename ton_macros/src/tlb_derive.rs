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

fn strip_defaults(mut generics: syn::Generics) -> syn::Generics {
    for param in generics.params.iter_mut() {
        if let syn::GenericParam::Type(tp) = param {
            tp.default = None;
        }
    }
    generics
}

fn add_tlb_bounds(mut generics: syn::Generics, crate_path: &TokenStream) -> syn::Generics {
    let mut where_clause = generics.where_clause.clone().unwrap_or_else(|| syn::WhereClause {
        where_token: Default::default(),
        predicates: Default::default(),
    });

    for param in generics.params.iter() {
        if let syn::GenericParam::Type(tp) = param {
            let ident = &tp.ident;
            let pred: syn::WherePredicate = syn::parse_quote!(#ident: #crate_path::traits::tlb::TLB);
            where_clause.predicates.push(pred);
        }
    }

    generics.where_clause = Some(where_clause);
    generics
}

pub(crate) fn tlb_derive_impl(input: proc_macro::TokenStream) -> TokenStream {
    let mut input = syn::parse::<syn::DeriveInput>(input).unwrap();
    // Extract a description, modifying `input.attrs` to remove the matched attributes.
    let header_attrs: TLBHeaderAttrs = match deluxe::extract_attributes(&mut input) {
        Ok(desc) => desc,
        Err(e) => return e.into_compile_error(),
    };

    let crate_path = if let Ok(ton_core_crate) = crate_name("ton_core") {
        match ton_core_crate {
            FoundCrate::Itself => quote::quote! { crate },
            FoundCrate::Name(name) => {
                let ident = format_ident!("{name}");
                quote! { #ident }
            }
        }
    } else if let Ok(ton_crate) = crate_name("ton") {
        match ton_crate {
            FoundCrate::Itself => quote::quote! { crate::ton_core },
            FoundCrate::Name(name) => {
                let ident = format_ident!("{name}");
                quote! { #ident::ton_core }
            }
        }
    } else {
        panic!("Can't find ton_core or ton crate");
    };

    let ident = &input.ident;

    // Use original generics for type usage (may include defaults), but strip defaults for impl
    let ty_generics = input.generics.split_for_impl().1;
    let generics_for_impl = add_tlb_bounds(strip_defaults(input.generics.clone()), &crate_path);
    let (impl_generics, _, where_clause) = generics_for_impl.split_for_impl();

    let (read_def_tokens, write_def_tokens, extra_impl_tokens) = match &mut input.data {
        Data::Struct(data) => tlb_derive_struct(&header_attrs, data),
        Data::Enum(data) => tlb_derive_enum(&crate_path, ident, data, &input.generics),
        _ => panic!("TLB derive macros only supports structs and enums"),
    };

    let prefix_val = header_attrs.prefix.unwrap_or(0);
    let prefix_bits_len = header_attrs.bits_len.unwrap_or(0);

    quote::quote! {
        impl #impl_generics #crate_path::traits::tlb::TLB for #ident #ty_generics #where_clause {
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
