use crate::utils::crate_name_or_panic;
use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use std::collections::HashSet;
use syn::parse::{Parse, ParseStream};
use syn::*;

pub fn ton_methods_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr as TonMethodsAttrs);
    let new_item = match parse_macro_input!(item as Item) {
        Item::Trait(mut item_impl) => {
            if let Err(err) = rewrite_trait(&mut item_impl, &attrs) {
                return err.into_compile_error().into();
            }
            item_impl.into_token_stream()
        },
        Item::Impl(mut item_impl) => {
            if let Err(err) = rewrite_struct_impl(&mut item_impl, &attrs) {
                return err.into_compile_error().into();
            }
            item_impl.into_token_stream()
        },
        other => panic!("#[ton_methods]: unsupported item: {other:?}"),
    };
    TokenStream::from(new_item)
}

struct TonMethodsAttrs {
    name_format: Option<Case<'static>>,
}

struct TonMethodAttrs {
    name: String,
}

impl Parse for TonMethodsAttrs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut name_format = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            if key != "name_format" {
                return Err(Error::new(key.span(), "unsupported #[ton_methods] argument"));
            }
            if name_format.is_some() {
                return Err(Error::new(key.span(), "duplicate name_format argument"));
            }

            input.parse::<Token![=]>()?;
            let value: LitStr = input.parse()?;
            name_format = Some(parse_name_format(&value.value()).map_err(|err| Error::new(value.span(), err))?);

            if input.is_empty() {
                break;
            }
            input.parse::<Token![,]>()?;
        }

        Ok(Self { name_format })
    }
}

impl Parse for TonMethodAttrs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut name = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            if key != "name" {
                return Err(Error::new(key.span(), "unsupported #[ton_method] argument"));
            }
            if name.is_some() {
                return Err(Error::new(key.span(), "duplicate name argument"));
            }

            input.parse::<Token![=]>()?;
            let value: LitStr = input.parse()?;
            if value.value().is_empty() {
                return Err(Error::new(value.span(), "method name cannot be empty"));
            }
            name = Some(value.value());

            if input.is_empty() {
                break;
            }
            input.parse::<Token![,]>()?;
        }

        name.map(|name| Self { name }).ok_or_else(|| Error::new(input.span(), "missing name argument"))
    }
}

fn parse_name_format(format: &str) -> std::result::Result<Case<'static>, String> {
    let format = format.trim();
    for &case in name_format_cases() {
        if case_matches_name(format, case) {
            return Ok(case);
        }
    }

    if let Some(case) = parse_semantic_case_alias(format) {
        return Ok(case);
    }

    let message = format!("unrecognized name_format {format:?}; expected a well-known convert_case format");
    Err(message)
}

fn name_format_cases() -> &'static [Case<'static>] {
    &[
        Case::Snake,
        Case::Constant,
        Case::UpperSnake,
        Case::Ada,
        Case::Kebab,
        Case::Cobol,
        Case::UpperKebab,
        Case::Train,
        Case::Flat,
        Case::UpperFlat,
        Case::Pascal,
        Case::UpperCamel,
        Case::Camel,
        Case::Lower,
        Case::Upper,
        Case::Title,
        Case::Sentence,
    ]
}

fn case_matches_name(input: &str, case: Case<'static>) -> bool {
    let variant_name = format!("{case:?}");
    let conventional_name = case_conventional_name(case, &variant_name);

    input == conventional_name
        || input == variant_name
        || input == variant_name.to_case(Case::Snake)
        || input == variant_name.to_case(Case::Kebab)
        || input == variant_name.to_case(Case::Camel)
        || input == variant_name.to_case(Case::Pascal)
}

fn case_conventional_name(case: Case<'static>, variant_name: &str) -> String {
    let phrase = variant_name.from_case(Case::Pascal).to_case(Case::Lower);
    format!("{phrase} case").to_case(case)
}

fn parse_semantic_case_alias(format: &str) -> Option<Case<'static>> {
    match format {
        "CamelCase" => Some(Case::Pascal),
        _ if case_alias_matches(format, Case::Constant, "screaming snake") => Some(Case::Constant),
        _ if case_alias_matches(format, Case::Cobol, "screaming kebab") => Some(Case::Cobol),
        _ if case_alias_matches(format, Case::Camel, "lower camel") => Some(Case::Camel),
        _ if case_alias_matches(format, Case::Pascal, "upper camel") => Some(Case::Pascal),
        _ => None,
    }
}

fn case_alias_matches(input: &str, case: Case<'static>, phrase: &str) -> bool {
    let phrase_with_case = format!("{phrase} case");
    input == phrase.to_case(Case::Snake)
        || input == phrase.to_case(Case::Kebab)
        || input == phrase.to_case(Case::Camel)
        || input == phrase.to_case(Case::Pascal)
        || input == phrase_with_case.to_case(Case::Snake)
        || input == phrase_with_case.to_case(Case::Kebab)
        || input == phrase_with_case.to_case(Case::Camel)
        || input == phrase_with_case.to_case(Case::Pascal)
        || input == phrase_with_case.to_case(case)
}

fn format_method_name(method_name: &str, name_format: Option<Case<'static>>) -> String {
    match name_format {
        Some(case) => method_name.to_case(case),
        None => method_name.to_owned(),
    }
}

fn take_method_name(attrs: &mut Vec<Attribute>) -> Result<Option<String>> {
    let mut method_name = None;
    let mut index = 0;
    while index < attrs.len() {
        if !attrs[index].path().is_ident("ton_method") {
            index += 1;
            continue;
        }
        if method_name.is_some() {
            return Err(Error::new_spanned(&attrs[index], "duplicate #[ton_method] attribute"));
        }

        method_name = Some(attrs[index].parse_args::<TonMethodAttrs>()?.name);
        attrs.remove(index);
    }
    Ok(method_name)
}

fn rewrite_trait(trait_items: &mut ItemTrait, attrs: &TonMethodsAttrs) -> Result<()> {
    for item in &mut trait_items.items {
        let TraitItem::Fn(method) = item else { continue };
        let method_name = take_method_name(&mut method.attrs)?;
        if method.default.is_some() {
            if method_name.is_some() {
                return Err(Error::new(method.sig.ident.span(), "#[ton_method] requires a generated method body"));
            }
            continue;
        }
        let body = build_body(&method.sig, method_name.as_deref(), attrs.name_format);
        method.default = Some(parse_quote!({ Box::pin(async move { #body.await }) }));
    }
    Ok(())
}

// The syntax like
// impl Struct {
//     async fn blabla(&self);
// }
// is not a valid rust syntax. So we parse it as verbatim and reconstruct the function.
fn rewrite_struct_impl(impl_items: &mut ItemImpl, attrs: &TonMethodsAttrs) -> Result<()> {
    for item in &mut impl_items.items {
        let ImplItem::Verbatim(verb_stream) = item else {
            continue;
        };

        let mut semi = match parse2::<SemiMethod>(verb_stream.clone()) {
            Ok(x) => x,
            Err(_) => panic!("Unexpected tokens in impl block: {}", verb_stream),
        };

        let method_name = take_method_name(&mut semi.attrs)?;
        let block = build_body(&semi.sig, method_name.as_deref(), attrs.name_format);

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
    Ok(())
}

#[rustfmt::skip]
fn build_body(signature: &Signature, method_name: Option<&str>, name_format: Option<Case<'static>>) -> proc_macro2::TokenStream {
    let method_name_str = method_name
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format_method_name(&signature.ident.to_string(), name_format));
    let crate_path = crate_name_or_panic("ton");

    let args = collect_args_info(signature);

    if args.is_empty() {
        return quote! { self.emulate_get_method(#method_name_str, &#crate_path::block_tlb::TVMStack::EMPTY) };
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
        self.emulate_get_method(#method_name_str, &stack)
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
        if let FnArg::Typed(pat_ty) = input
            && let Pat::Ident(pat_ident) = &*pat_ty.pat
        {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_method_name_keeps_default_unchanged() {
        assert_eq!(format_method_name("get_HTTP_data", None), "get_HTTP_data");
    }

    #[test]
    fn test_format_method_name_converts_common_formats() {
        assert_eq!(
            format_method_name("get_wallet_address", Some(parse_name_format("snake_case").unwrap())),
            "get_wallet_address",
        );
        assert_eq!(
            format_method_name("get_wallet_address", Some(parse_name_format("camelCase").unwrap())),
            "getWalletAddress",
        );
        assert_eq!(
            format_method_name("get_wallet_address", Some(parse_name_format("PascalCase").unwrap())),
            "GetWalletAddress",
        );
        assert_eq!(
            format_method_name("get_wallet_address", Some(parse_name_format("CamelCase").unwrap())),
            "GetWalletAddress",
        );
        assert_eq!(
            format_method_name("get_wallet_address", Some(parse_name_format("kebab-case").unwrap())),
            "get-wallet-address",
        );
    }

    #[test]
    fn test_parse_name_format_accepts_well_known_aliases() {
        for format in [
            "snake",
            "snake_case",
            "constant",
            "CONSTANT_CASE",
            "screaming_snake_case",
            "upper_snake",
            "ada",
            "Ada_Case",
            "kebab",
            "kebab-case",
            "cobol",
            "COBOL-CASE",
            "upper_kebab",
            "screaming-kebab-case",
            "train",
            "Train-Case",
            "flat",
            "flatcase",
            "upper_flat",
            "UPPERFLATCASE",
            "pascal",
            "PascalCase",
            "upper_camel",
            "upper_camel_case",
            "CamelCase",
            "camel",
            "camelCase",
            "lower_camel",
            "lower_camel_case",
            "lower",
            "lower case",
            "upper",
            "UPPER CASE",
            "title",
            "Title Case",
            "sentence",
            "Sentence case",
        ] {
            parse_name_format(format).unwrap_or_else(|err| panic!("{format:?} failed: {err}"));
        }
    }

    #[test]
    fn test_parse_name_format_rejects_weird_values() {
        assert!(parse_name_format("wat??").is_err());
    }

    #[test]
    fn test_parse_ton_methods_attrs_rejects_duplicate_name_format() {
        assert!(parse2::<TonMethodsAttrs>(quote!(name_format = "snake_case", name_format = "camelCase")).is_err());
    }

    #[test]
    fn test_method_name_override_rejects_empty_name() -> Result<()> {
        let mut method: TraitItemFn = parse_quote! {
            #[ton_method(name = "")]
            async fn get_ui_variables(&self);
        };

        assert!(take_method_name(&mut method.attrs).is_err());
        Ok(())
    }
}
