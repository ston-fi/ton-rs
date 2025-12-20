use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{TraitItemFn, ImplItemFn, ItemFn};
use crate::utils::crate_name_or_panic;

pub fn ton_method_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Try parse as inherent impl method first; else as trait method (async_trait); else as free/inherent ItemFn
    if let Ok(mut f) = syn::parse::<ImplItemFn>(item.clone()) {
        return ton_method_inherent_fn_impl(f);
    }
    if let Ok(mut method) = syn::parse::<TraitItemFn>(item.clone()) {
        return ton_method_trait_impl(method);
    }
    if let Ok(mut f) = syn::parse::<ItemFn>(item.clone()) {
        return ton_method_item_fn_impl(f);
    }
    // Fallback: return item unchanged
    item
}

fn ton_method_trait_impl(mut method: TraitItemFn) -> TokenStream {
    let method_name_str = method.sig.ident.to_string();
    let crate_path = crate_name_or_panic("ton");

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
    let generic_idents: std::collections::HashSet<syn::Ident> = method
        .sig.generics.params.iter().filter_map(|gp| match gp { syn::GenericParam::Type(tp) => Some(tp.ident.clone()), _ => None }).collect();

    let body = if args.is_empty() {
        quote! { self.emulate_get_method(#method_name_str, &#crate_path::block_tlb::TVMStack::EMPTY, None) }
    } else {
        let push_args = args.iter().map(|(ident, ty, is_ref)| {
            let type_ident_opt = match ty {
                syn::Type::Path(tp) => tp.path.get_ident().cloned(),
                syn::Type::Reference(tr) => match &*tr.elem { syn::Type::Path(tp) => tp.path.get_ident().cloned(), _ => None },
                _ => None,
            };
            let is_generic_arg = type_ident_opt.as_ref().map(|id| generic_idents.contains(id)).unwrap_or(false);
            if is_generic_arg {
                if *is_ref { quote! { stack.push_int((*#ident).into()); } } else { quote! { stack.push_int(#ident.into()); } }
            } else if *is_ref {
                quote! { #crate_path::block_tlb::ToTVMStack::push_to_stack(#ident, &mut stack)?; }
            } else {
                quote! { #crate_path::block_tlb::ToTVMStack::push_to_stack(&#ident, &mut stack)?; }
            }
        });
        quote! { let mut stack = #crate_path::block_tlb::TVMStack::default(); #( #push_args )* self.emulate_get_method(#method_name_str, &stack, None) }
    };

    method.default = Some(syn::parse_quote!({ Box::pin(async move { #body.await }) }));
    TokenStream::from(method.into_token_stream())
}

fn ton_method_inherent_fn_impl(mut f: ImplItemFn) -> TokenStream {
    let crate_path = crate_name_or_panic("ton");
    let method_name_str = f.sig.ident.to_string();

    // Ensure we keep the original async signature; do not rewrite to Future

    // Collect args
    let mut args: Vec<(syn::Ident, syn::Type, bool)> = Vec::new();
    for input in f.sig.inputs.iter() {
        if let syn::FnArg::Typed(pat_ty) = input {
            if let syn::Pat::Ident(pat_ident) = &*pat_ty.pat {
                let is_ref = matches!(&*pat_ty.ty, syn::Type::Reference(_));
                args.push((pat_ident.ident.clone(), (*pat_ty.ty).clone(), is_ref));
            }
        }
    }
    let generic_idents: std::collections::HashSet<syn::Ident> = f
        .sig.generics.params.iter().filter_map(|gp| match gp { syn::GenericParam::Type(tp) => Some(tp.ident.clone()), _ => None }).collect();

    let body = if args.is_empty() {
        quote! { self.emulate_get_method(#method_name_str, &#crate_path::block_tlb::TVMStack::EMPTY, None).await }
    } else {
        let push_args = args.iter().map(|(ident, ty, is_ref)| {
            let type_ident_opt = match ty {
                syn::Type::Path(tp) => tp.path.get_ident().cloned(),
                syn::Type::Reference(tr) => match &*tr.elem { syn::Type::Path(tp) => tp.path.get_ident().cloned(), _ => None },
                _ => None,
            };
            let is_generic_arg = type_ident_opt.as_ref().map(|id| generic_idents.contains(id)).unwrap_or(false);
            if is_generic_arg {
                if *is_ref { quote! { stack.push_int((*#ident).into()); } } else { quote! { stack.push_int(#ident.into()); } }
            } else if *is_ref {
                quote! { #crate_path::block_tlb::ToTVMStack::push_to_stack(#ident, &mut stack)?; }
            } else {
                quote! { #crate_path::block_tlb::ToTVMStack::push_to_stack(&#ident, &mut stack)?; }
            }
        });
        quote! { let mut stack = #crate_path::block_tlb::TVMStack::default(); #( #push_args )* self.emulate_get_method(#method_name_str, &stack, None).await }
    };

    f.block = syn::parse_quote!({ #body });
    TokenStream::from(f.into_token_stream())
}

fn ton_method_item_fn_impl(mut f: ItemFn) -> TokenStream {
    let crate_path = crate_name_or_panic("ton");
    let method_name_str = f.sig.ident.to_string();

    // Collect args
    let mut args: Vec<(syn::Ident, syn::Type, bool)> = Vec::new();
    for input in f.sig.inputs.iter() {
        if let syn::FnArg::Typed(pat_ty) = input {
            if let syn::Pat::Ident(pat_ident) = &*pat_ty.pat {
                let is_ref = matches!(&*pat_ty.ty, syn::Type::Reference(_));
                args.push((pat_ident.ident.clone(), (*pat_ty.ty).clone(), is_ref));
            }
        }
    }
    let generic_idents: std::collections::HashSet<syn::Ident> = f
        .sig.generics.params.iter().filter_map(|gp| match gp { syn::GenericParam::Type(tp) => Some(tp.ident.clone()), _ => None }).collect();

    let body = if args.is_empty() {
        quote! { self.emulate_get_method(#method_name_str, &#crate_path::block_tlb::TVMStack::EMPTY, None).await }
    } else {
        let push_args = args.iter().map(|(ident, ty, is_ref)| {
            let type_ident_opt = match ty {
                syn::Type::Path(tp) => tp.path.get_ident().cloned(),
                syn::Type::Reference(tr) => match &*tr.elem { syn::Type::Path(tp) => tp.path.get_ident().cloned(), _ => None },
                _ => None,
            };
            let is_generic_arg = type_ident_opt.as_ref().map(|id| generic_idents.contains(id)).unwrap_or(false);
            if is_generic_arg {
                if *is_ref { quote! { stack.push_int((*#ident).into()); } } else { quote! { stack.push_int(#ident.into()); } }
            } else if *is_ref {
                quote! { #crate_path::block_tlb::ToTVMStack::push_to_stack(#ident, &mut stack)?; }
            } else {
                quote! { #crate_path::block_tlb::ToTVMStack::push_to_stack(&#ident, &mut stack)?; }
            }
        });
        quote! { let mut stack = #crate_path::block_tlb::TVMStack::default(); #( #push_args )* self.emulate_get_method(#method_name_str, &stack, None).await }
    };

    f.block = syn::parse_quote!({ #body });
    TokenStream::from(f.into_token_stream())
}
