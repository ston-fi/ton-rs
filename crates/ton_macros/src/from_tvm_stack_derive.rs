use crate::utils::crate_name_or_panic;
use proc_macro2::TokenStream;
use quote::quote;

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(from_tvm_stack))]
struct HeaderAttributes {
    ensure_empty: Option<bool>, // use false as default
    allow_extra: Option<bool>,  // use false as default
}

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(from_tvm_stack))]
struct FieldAttributes {
    adapter: Option<syn::Expr>,
}

pub fn from_tvm_stack_derive_impl(input: proc_macro::TokenStream) -> TokenStream {
    let mut input = syn::parse::<syn::DeriveInput>(input).unwrap();
    let header_attrs: HeaderAttributes = match deluxe::extract_attributes(&mut input) {
        Ok(desc) => desc,
        Err(e) => return e.into_compile_error(),
    };

    let crate_path = crate_name_or_panic("ton");

    let name = input.ident;
    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = input.data
    {
        named
    } else {
        // Now macros only for ordinary structs with named fields, add more if needed
        unimplemented!("Now it is implemented only for ordinary structs with named fields")
    };

    let members_count = fields.len();
    let names = fields.clone().into_iter().map(|f| {
        let name = &f.ident;
        quote! {#name}
    });
    let mut assigns = Vec::with_capacity(fields.len());
    for field in fields.iter().rev() {
        let adapter = match field_adapter(field) {
            Ok(adapter) => adapter,
            Err(error) => return error.into_compile_error(),
        };
        let name = &field.ident;
        let ty = &field.ty;
        let value = if let Some(adapter) = adapter {
            quote! {
                {
                    let cell = <#crate_path::ton_core::cell::TonCell as
                        #crate_path::block_tlb::FromTVMStack>::from_stack(stack)?;
                    (#adapter).read(&mut cell.parser())?
                }
            }
        } else {
            quote! { #crate_path::block_tlb::FromTVMStack::from_stack(stack)? }
        };
        assigns.push(quote! { let #name: #ty = #value; });
    }

    let ensure_empty = if header_attrs.ensure_empty.unwrap_or(false) {
        quote! {stack.ensure_empty()?;}
    } else {
        quote! {}
    };
    let drop_extra = if header_attrs.allow_extra.unwrap_or(false) {
        quote! {
            while stack.len() > #members_count {
                let _ = stack.pop_checked()?;
            }
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        impl #crate_path::block_tlb::FromTVMStack for #name  {
            fn from_stack(stack: &mut #crate_path::block_tlb::TVMStack) -> #crate_path::errors::TonResult<Self> {
                #drop_extra

                #(#assigns)*

                #ensure_empty

                Ok(Self {
                    #(#names,) *
                })
            }
        }
    };

    // Hand the output tokens back to the compiler.
    expanded
}

fn field_adapter(field: &syn::Field) -> syn::Result<Option<syn::Expr>> {
    let mut attrs = field.attrs.clone();
    let attrs: FieldAttributes = deluxe::extract_attributes(&mut attrs)?;
    match attrs.adapter {
        // Accept the quoted expressions used by TLB field adapters as well.
        Some(syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(value),
            ..
        })) => value.parse::<syn::Expr>().map(Some),
        adapter => Ok(adapter),
    }
}

#[cfg(test)]
#[path = "tests/test_from_tvm_stack_derive.rs"]
mod tests;
