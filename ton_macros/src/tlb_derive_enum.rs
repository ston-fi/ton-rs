use convert_case::{Case, Casing};
use deluxe::____private::Ident;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DataEnum, Fields};

pub(crate) fn tlb_derive_enum(
    crate_path: &TokenStream,
    ident: &Ident,
    data: &mut DataEnum,
    generics: &syn::Generics,
) -> (TokenStream, TokenStream, TokenStream) {
    // Prepare variant info (tuple-like enums with exactly one unnamed field)
    let variant_infos: Vec<_> = data
        .variants
        .iter()
        .map(|variant| {
            let variant_name = &variant.ident;
            let Fields::Unnamed(fields) = &variant.fields else {
                panic!("tlb_derive_enum only supports tuple-like enums");
            };
            if fields.unnamed.len() != 1 {
                panic!("Each enum variant must have exactly one unnamed field");
            }
            let field_type = &fields.unnamed.first().unwrap().ty;
            (variant_name, field_type)
        })
        .collect();

    // Fallback reader: try each variant sequentially (use full trait qualification)
    let fallback_readers = variant_infos.iter().map(|(variant_name, field_type)| {
        quote! {
            match <#field_type as #crate_path::traits::tlb::TLB>::read(parser) {
                Ok(res) => return Ok(#ident::#variant_name(res)),
                Err(#crate_path::errors::TonCoreError::TLBWrongPrefix { .. }) => {},
                Err(#crate_path::errors::TonCoreError::TLBEnumOutOfOptions { .. }) => {},
                Err(err) => return Err(err),
            };
        }
    });

    // Generate const bits_len values for each variant
    let const_bits_decls = variant_infos.iter().enumerate().map(|(i, (_, ty))| {
        let name = Ident::new(&format!("PREFIX_BITS_LEN_{i}"), ident.span());
        quote! { const #name: usize = <#ty as TLB>::PREFIX.bits_len; }
    });

    let first_bits_ident = Ident::new("PREFIX_BITS_LEN_0", ident.span());

    // Build all-same expression at runtime (bool), using the consts above
    let all_same_checks = {
        let mut exprs: Vec<TokenStream> = Vec::new();
        for i in 1..variant_infos.len() {
            let cname = Ident::new(&format!("PREFIX_BITS_LEN_{}", i), ident.span());
            exprs.push(quote! { #first_bits_ident == #cname });
        }
        if exprs.is_empty() {
            quote! { #first_bits_ident > 0 }
        } else {
            quote! { #first_bits_ident > 0 && #( #exprs )&&* }
        }
    };

    let ident_str = ident.to_string();

    // Optimized match arms: use guard with equality against Type::PREFIX.value
    let match_arms = variant_infos.iter().map(|(_, ty)| {
        quote! {
            actual_prefix if actual_prefix == <#ty as TLB>::PREFIX.value => <#ty as TLB>::read(parser).map(Into::into),
        }
    });

    // Inline if/else inside read(): optimized vs fallback
    let read_impl = quote! {
        #(#const_bits_decls)*
        const ALL_BITS_LEN_SAME: bool = #all_same_checks ;
        if ALL_BITS_LEN_SAME {
            let prefix_bits_len = #first_bits_ident;
            let actual_prefix = match parser.read_num::<usize>(prefix_bits_len) {
                Ok(prefix) => prefix,
                Err(err) => return Err(#crate_path::errors::TonCoreError::TLBEnumOutOfOptions(format!("{}: {err}", #ident_str))),
            };
            parser.seek_bits(-(prefix_bits_len as i32))?;
            match actual_prefix {
                #(#match_arms)*
                _ => Err(#crate_path::errors::TonCoreError::TLBEnumOutOfOptions(format!("{}: got prefix: 0x{actual_prefix:x}", #ident_str))),
            }
        } else {
            #(#fallback_readers)*
            Err(#crate_path::errors::TonCoreError::TLBEnumOutOfOptions((#ident_str).to_string()))
        }
    };

    // write_definition stays the same
    let variant_writers = data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let Fields::Unnamed(fields) = &variant.fields else {
            panic!("TLB derive only supports tuple-like enums");
        };
        if fields.unnamed.len() != 1 {
            panic!("Each enum variant must have exactly one unnamed field");
        }
        quote! { Self::#variant_name(value) => value.write(builder)?, }
    });

    let write_impl = quote! { match self { #(#variant_writers)* } Ok(()) };

    // Keep accessor/From impls
    let variants_access = variants_access_impl(ident, data, generics);
    let variants_into = variants_into_impl(ident, data, generics);
    let extra_impl = quote! {
        #variants_access
        #variants_into
    };

    (read_impl, write_impl, extra_impl)
}

// generate From<X> for each enum variant
fn variants_into_impl(ident: &Ident, data: &mut DataEnum, generics: &syn::Generics) -> TokenStream {
    use syn::{PathArguments, Type, TypePath};

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    fn unwrap_box_or_arc(ty: &Type) -> Option<(&'static str, &Type)> {
        let Type::Path(TypePath { path, .. }) = ty else {
            return None;
        };

        let seg = path.segments.last()?;
        let ident = seg.ident.to_string();

        let PathArguments::AngleBracketed(args) = &seg.arguments else {
            return None;
        };
        let syn::GenericArgument::Type(inner_ty) = args.args.first()? else {
            return None;
        };

        match ident.as_str() {
            "Box" => Some(("Box", inner_ty)),
            "Arc" => Some(("Arc", inner_ty)),
            _ => None,
        }
    }

    let from_impls = data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        match &variant.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                let ty = &fields.unnamed.first().unwrap().ty;

                let base_from = quote! {
                    impl #impl_generics From<#ty> for #ident #ty_generics #where_clause {
                        fn from(v: #ty) -> Self {
                            #ident::#variant_name(v)
                        }
                    }
                };

                if let Some((wrapper, inner_ty)) = unwrap_box_or_arc(ty) {
                    let wrapper_ident = syn::Ident::new(wrapper, variant_name.span());
                    quote! {
                        #base_from
                        impl #impl_generics From<#inner_ty> for #ident #ty_generics #where_clause {
                            fn from(v: #inner_ty) -> Self {
                                #ident::#variant_name(#wrapper_ident::new(v))
                            }
                        }
                    }
                } else {
                    quote! { #base_from }
                }
            }
            _ => panic!("variants_into_impl supports only tuple-like enums "),
        }
    });
    quote! {
        #(#from_impls)*
    }
}

// generate as_X and is_X methods for each enum variant
fn variants_access_impl(ident: &Ident, data: &mut DataEnum, generics: &syn::Generics) -> TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let methods = data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let method_suffix = variant_name.to_string().to_case(Case::Snake);
        let as_fn = Ident::new(&format!("as_{method_suffix}"), variant_name.span());
        let as_fn_mut = Ident::new(&format!("as_{method_suffix}_mut"), variant_name.span());
        let into_fn = Ident::new(&format!("into_{method_suffix}"), variant_name.span());

        match &variant.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                let field_ty = &fields.unnamed.first().unwrap().ty;

                quote! {
                    pub fn #as_fn(&self) -> Option<& #field_ty> {
                        match self {
                            #ident::#variant_name(inner) => Some(inner),
                             _ => None,
                        }
                    }

                    pub fn #as_fn_mut(&mut self) -> Option<&mut #field_ty> {
                        match self {
                            #ident::#variant_name(inner) => Some(inner),
                            _ => None,
                        }
                    }

                    pub fn #into_fn(self) -> Option<#field_ty> {
                        match self {
                            #ident::#variant_name(inner) => Some(inner),
                            _ => None,
                        }
                    }
                }
            }
            _ => panic!("variants_access_impl supports only tuple-like enums "),
        }
    });

    quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            #(#methods)*
        }
    }
}
