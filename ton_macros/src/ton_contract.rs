use proc_macro::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote};
use syn::{parse_macro_input, FieldsNamed, ItemStruct};

pub(crate) fn ton_contract_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let struct_name = &input.ident;
    let vis = &input.vis;
    let attrs = &input.attrs;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let found_crate = crate_name("ton_lib").expect("ton crate not found");

    let crate_path = match found_crate {
        FoundCrate::Itself => quote::quote! { crate },
        FoundCrate::Name(name) => {
            let ident = format_ident!("{name}");
            quote! { #ident }
        }
    };

    let mut new_fields = input.fields.clone();
    if let syn::Fields::Unnamed(_) = &new_fields {
        panic!("ton_contract derive does not support tuple structs, use named fields instead");
    };

    if let syn::Fields::Unit = new_fields {
        new_fields = syn::Fields::Named(FieldsNamed {
            named: syn::punctuated::Punctuated::new(),
            brace_token: syn::token::Brace::default(),
        })
    };

    if let syn::Fields::Named(fields) = &mut new_fields {
        fields.named.push(syn::parse_quote! {
            contract_ctx: ContractCtx
        });
    } else {
        panic!("unexpected error");
    }

    let output = quote! {
        #(#attrs)*
        #vis struct #struct_name #generics #new_fields

        impl #impl_generics #crate_path::contracts::ton_contract::TonContract for #struct_name #ty_generics #where_clause {
            fn ctx(&self) -> &ContractCtx {
                &self.contract_ctx
            }

            fn from_ctx(ctx: ContractCtx) -> Self {
                Self {
                    contract_ctx: ctx,
                }
            }
        }
    };
    output.into()
}
