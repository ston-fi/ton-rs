use proc_macro2::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote};

pub(crate) fn get_crate_name_or_panic(orig_name: &'static str) -> TokenStream {
    let Ok(ton_crate) = crate_name(orig_name) else {
        panic!("Can't find {orig_name} crate");
    };

    match ton_crate {
        FoundCrate::Itself => quote::quote! { crate },
        FoundCrate::Name(name) => {
            let ident = format_ident!("{name}");
                quote! { #ident }
            }
    }
}
