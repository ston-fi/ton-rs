use proc_macro::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, TraitItemFn};
use crate::utils::get_crate_name_or_panic;

pub fn ton_method_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut method = parse_macro_input!(item as TraitItemFn);
    let method_name_str = method.sig.ident.to_string();

    let crate_path = get_crate_name_or_panic("ton");

    // Ensure the trait method is async so we can use await in the generated body
    method.sig.asyncness = Some(Default::default());

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

    let body_inner = if args.is_empty() {
        quote! {
            self.emulate_get_method(#method_name_str, &#crate_path::block_tlb::TVMStack::EMPTY, None).await
        }
    } else {
        let push_args = args.iter().map(|(ident, ty, is_ref)| {
            // If arg type is a generic with Into bound, use ident.into()
            let use_into = match ty {
                syn::Type::Path(tp) => tp.path.get_ident().map(|id| generics_into.contains(id)).unwrap_or(false),
                _ => false,
            };

            if use_into {
                quote! { #crate_path::block_tlb::ToTVMStack::push_to_stack(&#ident.into(), &mut stack)?; }
            } else if *is_ref {
                quote! { #crate_path::block_tlb::ToTVMStack::push_to_stack(#ident, &mut stack)?; }
            } else {
                quote! { #crate_path::block_tlb::ToTVMStack::push_to_stack(&#ident, &mut stack)?; }
            }
        });

        quote! {
            let mut stack = #crate_path::block_tlb::TVMStack::default();
            #( #push_args )*
            self.emulate_get_method(#method_name_str, &stack, None).await
        }
    };

    // Trampoline future pattern to keep await in a local async block
    let body = quote!({ #body_inner });

    method.default = Some(syn::parse_quote!(#body));

    TokenStream::from(method.into_token_stream())
}
