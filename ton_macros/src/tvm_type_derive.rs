use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use crate::utils::get_crate_name_or_panic;

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(tvm_type))]
pub(crate) struct TVMTypeHeaderAttributes {
    pub(crate) ensure_empty: Option<bool>, // use false as default
}

pub fn tvm_type_derive_impl(input: proc_macro::TokenStream) -> TokenStream {
    let mut input = syn::parse::<syn::DeriveInput>(input).unwrap();
    let header_attrs: TVMTypeHeaderAttributes = match deluxe::extract_attributes(&mut input) {
        Ok(desc) => desc,
        Err(e) => return e.into_compile_error(),
    };

    let crate_path = get_crate_name_or_panic("ton");

    let name = input.ident;
    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = input.data
    {
        named
    } else {
        // Now macros only for ordinary structs with named fields, add more if needed
        unimplemented!("Now it is implemented only for ordinary structs with named fields")
    };

    let names = fields.clone().into_iter().map(|f| {
        let name = &f.ident;
        quote! {#name}
    });
    let assigns = fields.into_iter().rev().map(|f| {
        let name = &f.ident;
        quote! {
            let #name = #crate_path::block_tlb::TVMType::from_stack(stack)?;
        }
    });

    let ensure_empty = if header_attrs.ensure_empty.unwrap_or(false) {
        quote! {stack.ensure_empty()?;}
    } else {
        quote! {}
    };

    let expanded = quote! {
        impl #crate_path::block_tlb::TVMType for #name  {
            fn from_stack(stack: &mut #crate_path::block_tlb::TVMStack) -> #crate_path::errors::TonResult<Self> {
                #(#assigns)*

                #ensure_empty

                Ok(Self {
                    #(#names,) *
                })
            }
        }
    };

    // Hand the output tokens back to the compiler.
    expanded
}
