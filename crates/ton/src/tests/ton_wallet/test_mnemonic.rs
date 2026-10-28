use super::*;

#[test]
fn mnemonic_parse_works() -> anyhow::Result<()> {
    let words = "dose ice enrich trigger test dove century still betray gas diet dune use other base gym mad law immense village world example praise game";
    let mnemonic = Mnemonic::from_str(words, None);
    assert!(mnemonic.is_ok());

    let words = " dose ice enrich trigger test dove \
    century still betray gas diet       dune use other base gym mad law \
    immense village world example praise game ";
    let mnemonic = Mnemonic::from_str(words, None);
    assert!(mnemonic.is_ok());
    Ok(())
}

#[test]
fn mnemonic_validate_works() -> anyhow::Result<()> {
    let mnemonic = Mnemonic::new(
        vec![
            "dose", "ice", "enrich", "trigger", "test", "dove", "century", "still", "betray", "gas", "diet", "dune",
        ],
        None,
    );
    assert!(mnemonic.is_err());
    let mnemonic = Mnemonic::new(vec!["a"], None);
    assert!(mnemonic.is_err());
    Ok(())
}

#[test]
fn mnemonic_to_private_key_works() -> anyhow::Result<()> {
    let mnemonic = Mnemonic::new(
        vec![
            "dose", "ice", "enrich", "trigger", "test", "dove", "century", "still", "betray", "gas", "diet", "dune",
            "use", "other", "base", "gym", "mad", "law", "immense", "village", "world", "example", "praise", "game",
        ],
        None,
    )?;
    let expected = "119dcf2840a3d56521d260b2f125eedc0d4f3795b9e627269a4b5a6dca8257bdc04ad1885c127fe863abb00752fa844e6439bb04f264d70de7cea580b32637ab";

    let kp = mnemonic.to_key_pair()?;
    let expected = Zeroizing::new(hex::decode(expected)?);
    assert_eq!(kp.secret_key.as_slice(), expected.as_slice());

    Ok(())
}
