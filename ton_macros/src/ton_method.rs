use proc_macro::TokenStream;
use std::collections::HashSet;
use quote::{quote, ToTokens};
use syn::{TraitItemFn, Signature, ImplItem};
use crate::utils::crate_name_or_panic;

pub fn ton_method_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // try struct impl first
    // We can't parse directly to ImplItemFn because it doesn't allow empty body
    if let Ok(ImplItem::Fn(mut method)) = syn::parse::<ImplItem>(item.clone()) {
        let body = build_body(&method.sig);
        method.block = syn::parse_quote!({ #body });
        return TokenStream::from(method.into_token_stream())
    }

    // then try trait impl
    if let Ok(mut method) = syn::parse::<TraitItemFn>(item.clone()) {
        let body = build_body(&method.sig);
        method.default = Some(syn::parse_quote!({ Box::pin(async move { #body.await }) }));
        return TokenStream::from(method.into_token_stream())
    }
    // Fallback: return item unchanged
    item
}

fn build_body(signature: &Signature) -> proc_macro2::TokenStream {
    let method_name_str = signature.ident.to_string();
    let crate_path = crate_name_or_panic("ton");

    let args = collect_args_info(&signature);

    if args.is_empty() {
        return quote! { self.emulate_get_method(#method_name_str, &#crate_path::block_tlb::TVMStack::EMPTY, None) }
    }
    let push_args = args.into_iter().map(|info| {
        let ident = info.ident;
        match (info.is_generic, info.is_ref) {
            (true, true) => quote! { #crate_path::block_tlb::PushToStack::push_to_stack(#ident.into(), &mut stack)?; },
            (true, false) => quote! { #crate_path::block_tlb::PushToStack::push_to_stack(&#ident.into(), &mut stack)?; },
            (false, true) => quote! { #crate_path::block_tlb::PushToStack::push_to_stack(#ident, &mut stack)?; },
            (false, false) => quote! { #crate_path::block_tlb::PushToStack::push_to_stack(&#ident, &mut stack)?; },
        }
    });
    quote! {
        let mut stack = #crate_path::block_tlb::TVMStack::default();
        #( #push_args )*
        self.emulate_get_method(#method_name_str, &stack, None)
    }
}

struct ArgInfo {
    ident: syn::Ident,
    is_ref: bool,
    is_generic: bool,
}

fn collect_args_info(signature: &Signature) -> Vec<ArgInfo> {
    let generic_idents: HashSet<_> = signature.generics.params.iter()
        .filter_map(|gp| match gp {
            syn::GenericParam::Type(tp) => Some(tp.ident.clone()),
            _ => None
        })
        .collect();

    let mut args: Vec<ArgInfo> = Vec::new();
    for input in signature.inputs.iter() {
        if let syn::FnArg::Typed(pat_ty) = input {
            if let syn::Pat::Ident(pat_ident) = &*pat_ty.pat {
                let ty = *pat_ty.ty.clone();
                let type_ident_opt = match &ty {
                    syn::Type::Path(tp) => tp.path.get_ident().cloned(),
                    syn::Type::Reference(tr) => match &*tr.elem {
                        syn::Type::Path(tp) => tp.path.get_ident().cloned(),
                        _ => None
                    },
                    _ => None,
                };
                let is_ref = matches!(&ty, syn::Type::Reference(_));
                let is_generic = type_ident_opt.as_ref().map(|id| generic_idents.contains(id)).unwrap_or(false);
                let arg_info = ArgInfo {
                    ident: pat_ident.ident.clone(),
                    is_ref,
                    is_generic,
                };
                args.push(arg_info);
            }
        }
    };
    args
}
