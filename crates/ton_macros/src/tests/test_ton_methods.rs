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

    assert!(take_method_selector(&mut method.attrs).is_err());
    Ok(())
}
#[test]
fn test_method_selector_rejects_ambiguous_or_out_of_range_ids() {
    for tokens in [
        quote!(name = "seqno", id = 85143),
        quote!(id = 85143, name = "seqno"),
        quote!(id = 1, id = 2),
        quote!(id = 2147483648),
        quote!(id = -2147483649),
        quote!(id = "85143"),
    ] {
        assert!(parse2::<TonMethodAttrs>(tokens).is_err());
    }
}

#[test]
fn test_method_selector_accepts_signed_i32_limits_and_hex_literals() -> Result<()> {
    for (tokens, expected) in [
        (quote!(id = -2147483648), i32::MIN),
        (quote!(id = 2147483647), i32::MAX),
        (quote!(id = 0x1_abcd,), 0x1_abcd),
    ] {
        let TonMethodAttrs::Id(actual) = parse2(tokens)? else { panic!("expected numeric id") };
        assert_eq!(actual, expected);
    }
    Ok(())
}
