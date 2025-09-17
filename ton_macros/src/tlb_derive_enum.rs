use convert_case::{Case, Casing};
use deluxe::____private::Ident;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DataEnum, Fields};

pub(crate) fn tlb_derive_enum(
    crate_path: &TokenStream,
    ident: &Ident,
    data: &mut DataEnum,
) -> (TokenStream, TokenStream, TokenStream) {
    let variant_readers = data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        // Expect single unnamed field (like `Std(...)`)
        let Fields::Unnamed(fields) = &variant.fields else {
            panic!("tlb_derive_enum only supports tuple-like enums");
        };
        if fields.unnamed.len() != 1 {
            panic!("Each enum variant must have exactly one unnamed field");
        }
        let field_type = &fields.unnamed.first().unwrap().ty;
        quote! {
                match #field_type::read(parser) {
                    Ok(res) => return Ok(#ident::#variant_name(res)),
                    Err(#crate_path::errors::TonCoreError::TLBWrongPrefix { .. }) => {},
                    Err(#crate_path::errors::TonCoreError::TLBEnumOutOfOptions { .. }) => {},
                    Err(err) => return Err(err),
                };
        }
    });

    let variant_writers = data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        // Expect single unnamed field (like `Std(...)`)
        let Fields::Unnamed(fields) = &variant.fields else {
            panic!("tlb_derive_enum only supports tuple-like enums");
        };

        if fields.unnamed.len() != 1 {
            panic!("Each enum variant must have exactly one unnamed field");
        }
        quote! {
            Self::#variant_name(ref value) => value.write(builder)?,
        }
    });

    let ident_str = ident.to_string();

    let read_impl = quote! {
        #(#variant_readers)*
        Err(#crate_path::errors::TonCoreError::TLBEnumOutOfOptions((#ident_str).to_string()))
    };

    let write_impl = quote! {
        match self {
            #(#variant_writers)*
        }
        Ok(())
    };

    let variants_access = variants_access_impl(ident, data);
    let variants_into = variants_into_impl(ident, data);
    let extra_impl = quote! {
        #variants_access
        #variants_into
    };

    // impl
    (read_impl, write_impl, extra_impl)
}

// generate From<X> for each enum variant
fn variants_into_impl(ident: &Ident, data: &mut DataEnum) -> TokenStream {
    let from_impls = data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        match &variant.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                let ty = &fields.unnamed.first().unwrap().ty;

                Some(quote! {
                    impl From<#ty> for #ident {
                        fn from(v: #ty) -> Self {
                            #ident::#variant_name(v)
                        }
                    }
                })
            }
            _ => panic!("variants_into_impl supports only tuple-like enums "),
        }
    });
    quote! {
        #(#from_impls)*
    }
}

// generate as_X and is_X methods for each enum variant
fn variants_access_impl(ident: &Ident, data: &mut DataEnum) -> TokenStream {
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
                            #ident::#variant_name(ref inner) => Some(inner),
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
        impl #ident {
            #(#methods)*
        }
    }
}
