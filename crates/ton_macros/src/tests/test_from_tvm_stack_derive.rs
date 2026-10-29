use super::field_adapter;
use quote::ToTokens;

#[test]
fn test_field_adapter_accepts_paths_and_configured_expressions() -> syn::Result<()> {
    for field in [
        syn::parse_quote!(#[from_tvm_stack(adapter = InternalAddress)] owner: Address),
        syn::parse_quote!(#[from_tvm_stack(adapter = "InternalAddress")] owner: Address),
    ] {
        assert_eq!(
            field_adapter(&field)?.map(|expr| expr.to_token_stream().to_string()),
            Some("InternalAddress".into())
        );
    }
    let field = syn::parse_quote!(#[from_tvm_stack(adapter = "ConstLen::<u8>::new(4)")] value: u8);
    assert!(matches!(field_adapter(&field)?, Some(syn::Expr::Call(_))));
    Ok(())
}

#[test]
fn test_field_adapter_reports_invalid_or_unknown_options() {
    for field in [
        syn::parse_quote!(#[from_tvm_stack(adapter = "not valid Rust!")] value: u8),
        syn::parse_quote!(#[from_tvm_stack(adaptor = InternalAddress)] value: Address),
        syn::parse_quote!(#[from_tvm_stack(adapter = InternalAddress, adapter = NullableAddress)] value: Address),
    ] {
        assert!(field_adapter(&field).is_err());
    }
}
