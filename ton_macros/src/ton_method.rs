use proc_macro::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, TraitItemFn};
use crate::utils::get_crate_name_or_panic;

pub fn ton_method_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut method = parse_macro_input!(item as TraitItemFn);
    let method_name_str = method.sig.ident.to_string();

    let crate_path = get_crate_name_or_panic("ton");

    // Collect argument idents (skip receiver like &self) and track if the arg is a reference
    let mut args: Vec<(syn::Ident, bool)> = Vec::new();
    for input in method.sig.inputs.iter() {
        if let syn::FnArg::Typed(pat_ty) = input {
            if let syn::Pat::Ident(pat_ident) = &*pat_ty.pat {
                let is_ref = matches!(&*pat_ty.ty, syn::Type::Reference(_));
                args.push((pat_ident.ident.clone(), is_ref));
            }
        }
    }

    let body_inner = if args.is_empty() {
        quote! {
            self.emulate_get_method(#method_name_str, &#crate_path::block_tlb::TVMStack::EMPTY, None).await
        }
    } else {
        let push_args = args.iter().map(|(ident, is_ref)| {
            if *is_ref {
                // Argument is already a reference, &.
                quote! {
                    #crate_path::block_tlb::ToTVMStack::push_to_stack(#ident, &mut stack)?;
                }
            } else {
                // Value arg. Pass by reference as expected by ToTVMStack.
                quote! {
                    #crate_path::block_tlb::ToTVMStack::push_to_stack(&#ident, &mut stack)?;
                }
            }
        });

        quote! {
            let mut stack = #crate_path::block_tlb::TVMStack::default();
            #( #push_args )*
            self.emulate_get_method(#method_name_str, &stack, None).await
        }
    };

    // Wrap in async block to satisfy async_trait's boxed Future expectations for default impls
    // let body = quote!({ async move { #body_inner } });

    // Replace the method block with generated default implementation
    method.default = Some(syn::parse_quote!({#body_inner}));

    // Return the modified method item
    TokenStream::from(method.into_token_stream())
}
