use proc_macro::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, TraitItemFn};
use crate::utils::get_crate_name_or_panic;

pub fn ton_method_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut method = parse_macro_input!(item as TraitItemFn);
    let method_name_str = method.sig.ident.to_string();

    let crate_path = get_crate_name_or_panic("ton");

    // Collect argument idents (skip receiver like &self)
    let mut arg_idents: Vec<syn::Ident> = Vec::new();
    for input in method.sig.inputs.iter() {
        if let syn::FnArg::Typed(pat_ty) = input {
            if let syn::Pat::Ident(pat_ident) = &*pat_ty.pat {
                arg_idents.push(pat_ident.ident.clone());
            }
        }
    }

    let body = if arg_idents.is_empty() {
        quote! {
            self.emulate_get_method(#method_name_str, #crate_path::block_tlb::TVMStack::EMPTY, None).await
        }
    } else {
        let push_args = arg_idents.iter().map(|ident| {
            quote! {
                #crate_path::block_tlb::ToTVMStack::push_to_stack(&#ident, &mut stack)?;
            }
        });


        quote! {
            let mut stack = #crate_path::block_tlb::TVMStack::default();
            #( #push_args )*
            self.emulate_get_method(#method_name_str, &stack, None).await
        }
    };

    // Replace the method block with generated default implementation
    method.default = Some(syn::parse_quote!({ #body }));

    // Return the modified method item
    TokenStream::from(method.into_token_stream())
}
