use super::*;

#[test]
fn test_coins_math() -> TonCoreResult<()> {
    let a = Coins::new(1_000_000);
    let b = Coins::new(500_000);

    let c = a.checked_add(b)?;
    assert_eq!(c, Coins::new(1_500_000));

    let d = a.checked_sub(500_000)?;
    assert_eq!(d, Coins::new(500_000));

    let e = a.checked_mul(3)?;
    assert_eq!(e, Coins::new(3_000_000));

    let f = a.checked_div(Coins::new(2))?;
    assert_eq!(f, Coins::new(500_000));

    Ok(())
}
