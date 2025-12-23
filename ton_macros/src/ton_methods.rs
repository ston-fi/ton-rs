use crate::utils::crate_name_or_panic;
use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use std::collections::HashSet;
use syn::parse::{Parse, ParseStream};
use syn::*;

pub fn ton_methods_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let new_item = match parse_macro_input!(item as Item) {
        Item::Trait(mut item_impl) => {
            rewrite_trait(&mut item_impl);
            item_impl.into_token_stream()
        }
        Item::Impl(mut item_impl) => {
            rewrite_struct_impl(&mut item_impl);
            item_impl.into_token_stream()
        }
        other => panic!("#[ton_methods]: unsupported item: {other:?}"),
    };
    TokenStream::from(new_item)
}

fn rewrite_trait(trait_items: &mut ItemTrait) {
    for item in &mut trait_items.items {
        let TraitItem::Fn(method) = item else { continue };
        if method.default.is_some() {
            continue;
        }
        let body = build_body(&method.sig);
        method.default = Some(parse_quote!({ Box::pin(async move { #body.await }) }));
    }
}

// The syntax like
// impl Struct {
//     async fn blabla(&self);
// }
// is not a valid rust syntax. So we parse it as verbatim and reconstruct the function.
fn rewrite_struct_impl(impl_items: &mut ItemImpl) {
    for item in &mut impl_items.items {
        let ImplItem::Verbatim(verb_stream) = item else {
            continue;
        };

        let semi = match parse2::<SemiMethod>(verb_stream.clone()) {
            Ok(x) => x,
            Err(_) => panic!("Unexpected tokens in impl block: {}", verb_stream),
        };

        let block = build_body(&semi.sig);

        // bind fields to idents for quote repetition
        let attrs = &semi.attrs;
        let vis = &semi.vis;
        let sig = &semi.sig;

        let rebuilt = quote! {
            #(#attrs)*
            #vis #sig {#block.await}
        };

        let new_fn: ImplItemFn = parse2(rebuilt).expect("Failed to parse generated impl method");

        *item = ImplItem::Fn(new_fn);
    }
}

#[rustfmt::skip]
fn build_body(signature: &Signature) -> proc_macro2::TokenStream {
    let method_name_str = signature.ident.to_string();
    let crate_path = crate_name_or_panic("ton");

    let args = collect_args_info(signature);

    if args.is_empty() {
        return quote! { self.emulate_get_method(#method_name_str, &#crate_path::block_tlb::TVMStack::EMPTY, None) };
    }
    let push_args = args.into_iter().map(|info| {
        let ident = info.ident;
        match (info.is_generic, info.is_ref) {
            (true, true) => quote! { #crate_path::block_tlb::ToTVMStack::to_stack(#ident.into(), &mut stack)?; },
            (true, false) => quote! { #crate_path::block_tlb::ToTVMStack::to_stack(&#ident.into(), &mut stack)?; },
            (false, true) => quote! { #crate_path::block_tlb::ToTVMStack::to_stack(#ident, &mut stack)?; },
            (false, false) => quote! { #crate_path::block_tlb::ToTVMStack::to_stack(&#ident, &mut stack)?; },
        }
    });
    quote! {
        let mut stack = #crate_path::block_tlb::TVMStack::default();
        #( #push_args )*
        self.emulate_get_method(#method_name_str, &stack, None)
    }
}

struct ArgInfo {
    ident: Ident,
    is_ref: bool,
    is_generic: bool,
}

fn collect_args_info(signature: &Signature) -> Vec<ArgInfo> {
    let generic_idents: HashSet<_> = signature
        .generics
        .params
        .iter()
        .filter_map(|gp| match gp {
            GenericParam::Type(tp) => Some(tp.ident.clone()),
            _ => None,
        })
        .collect();

    let mut args: Vec<ArgInfo> = Vec::new();
    for input in signature.inputs.iter() {
        if let FnArg::Typed(pat_ty) = input {
            if let Pat::Ident(pat_ident) = &*pat_ty.pat {
                let ty = *pat_ty.ty.clone();
                let type_ident_opt = match &ty {
                    Type::Path(tp) => tp.path.get_ident().cloned(),
                    Type::Reference(tr) => match &*tr.elem {
                        Type::Path(tp) => tp.path.get_ident().cloned(),
                        _ => None,
                    },
                    _ => None,
                };
                let is_ref = matches!(&ty, Type::Reference(_));
                let is_generic = type_ident_opt.as_ref().map(|id| generic_idents.contains(id)).unwrap_or(false);
                let arg_info = ArgInfo {
                    ident: pat_ident.ident.clone(),
                    is_ref,
                    is_generic,
                };
                args.push(arg_info);
            }
        }
    }
    args
}

// Helper to reconstruct method from Verbatim tokens
struct SemiMethod {
    attrs: Vec<Attribute>,
    vis: Visibility,
    sig: Signature,
    _semi: Token![;],
}

impl Parse for SemiMethod {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            attrs: input.call(Attribute::parse_outer)?,
            vis: input.parse()?,
            sig: input.parse()?,
            _semi: input.parse()?,
        })
    }
}
