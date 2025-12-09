use quote::quote;
use syn::{DeriveInput, parse_macro_input};

pub fn tvm_result_derive_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = input.data
    {
        named
    } else {
        // Now macros only for ordinary structs with named fields, add more if needed
        unimplemented!()
    };

    let names = fields.clone().into_iter().map(|f| {
        let name = &f.ident;
        quote! {#name}
    });
    let assigns = fields.into_iter().rev().map(|f| {
        let name = &f.ident;
        quote! {
            let #name = TVMResult::from_stack(stack)?;
        }
    });

    let expanded = quote! {
        impl TVMResult for #name  {
            fn from_stack(stack: &mut TVMStack) -> TonResult<Self> {
                #(#assigns)*

                Ok(Self {
                    #(#names,) *
                })
            }
        }
    };

    // Hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)
}
