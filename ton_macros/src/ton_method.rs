use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, TraitItemFn};
use crate::utils::crate_name_or_panic;

pub fn ton_method_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut method = parse_macro_input!(item as TraitItemFn);
    let method_name_str = method.sig.ident.to_string();

    let crate_path = crate_name_or_panic("ton");

    // Collect generic idents that have an Into bound
    use syn::{GenericParam, TypeParamBound};
    let mut generics_into: std::collections::HashSet<syn::Ident> = std::collections::HashSet::new();
    for gp in method.sig.generics.params.iter() {
        if let GenericParam::Type(tp) = gp {
            for b in tp.bounds.iter() {
                if let TypeParamBound::Trait(tb) = b {
                    if tb.path.segments.iter().any(|seg| seg.ident == "Into") {
                        generics_into.insert(tp.ident.clone());
                    }
                }
            }
        }
    }

    // Collect argument idents, their type, and whether the arg is a reference
    let mut args: Vec<(syn::Ident, syn::Type, bool)> = Vec::new();
    for input in method.sig.inputs.iter() {
        if let syn::FnArg::Typed(pat_ty) = input {
            if let syn::Pat::Ident(pat_ident) = &*pat_ty.pat {
                let is_ref = matches!(&*pat_ty.ty, syn::Type::Reference(_));
                args.push((pat_ident.ident.clone(), (*pat_ty.ty).clone(), is_ref));
            }
        }
    }

    let body = if args.is_empty() {
        quote! {
            self.emulate_get_method(#method_name_str, &#crate_path::block_tlb::TVMStack::EMPTY, None)
        }
    } else {
        let push_args = args.iter().map(|(ident, ty, is_ref)| {
            let use_into = match ty {
                syn::Type::Path(tp) => tp.path.get_ident().map(|id| generics_into.contains(id)).unwrap_or(false),
                _ => false,
            };

            if use_into {
                quote! { #crate_path::block_tlb::ToTVMStack::push_to_stack(&(#ident.into()), &mut stack)?; }
            } else if *is_ref {
                quote! { #crate_path::block_tlb::ToTVMStack::push_to_stack(#ident, &mut stack)?; }
            } else {
                quote! { #crate_path::block_tlb::ToTVMStack::push_to_stack(&#ident, &mut stack)?; }
            }
        });

        quote! {
            let mut stack = #crate_path::block_tlb::TVMStack::default();
            #( #push_args )*
            self.emulate_get_method(#method_name_str, &stack, None)
        }
    };

    // Replace the method block with generated default implementation returning a boxed future for async_trait compatibility.
    method.default = Some(syn::parse_quote!({ Box::pin(async move { #body.await }) }));

    TokenStream::from(method.into_token_stream())
}
