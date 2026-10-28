use super::*;
use std::str::FromStr;

#[test]
fn test_biguint_to_from_u1024() -> anyhow::Result<()> {
    for value in [0, 1, 123, 1024, u128::MAX] {
        let fastnum_expected = U1024::from_u128(value)?;
        let bignum_converted = biguint_from_u1024(fastnum_expected)?;
        let bignum_expected = BigUint::from(value);
        let fastnum_converted = biguint_to_u1024(&bignum_converted)?;
        assert_eq!(bignum_converted, bignum_expected, "value={value}");
        assert_eq!(fastnum_converted, fastnum_expected, "value={value}");
    }
    let fastnum_max_expected = U1024::MAX;
    let bignum_max_converted = biguint_from_u1024(fastnum_max_expected)?;
    let bignum_max_expected = BigUint::from_str(
        "179769313486231590772930519078902473361797697894230657273430081157732675805500963132708477322407536021120113879871393357658789768814416622492847430639474124377767893424865485276302219601246094119453082952085005768838150682342462881473913110540827237163350510684586298239947245938479716304835356329624224137215",
    )?;
    let fastnum_max_converted = biguint_to_u1024(&bignum_max_converted)?;
    assert_eq!(bignum_max_converted, bignum_max_expected, "value=U1024::MAX");
    assert_eq!(fastnum_max_converted, fastnum_max_expected, "value=U1024::MAX");
    Ok(())
}

#[test]
fn test_biguint_to_from_i1024() -> anyhow::Result<()> {
    for value in [i128::MIN, -1024, -123, -1, 0, 1, 123, 1024, i128::MAX] {
        let fastnum_expected = I1024::from_i128(value)?;
        let bignum_converted = bigint_from_i1024(fastnum_expected)?;
        let bignum_expected = BigInt::from(value);
        let fastnum_converted = bigint_to_i1024(&bignum_converted)?;
        assert_eq!(bignum_converted, bignum_expected, "value={value}");
        assert_eq!(fastnum_converted, fastnum_expected, "value={value}");
    }
    let fastnum_max_expected = I1024::MAX;
    let bignum_max_converted = bigint_from_i1024(fastnum_max_expected)?;
    let bignum_max_expected = BigInt::from_str(
        "89884656743115795386465259539451236680898848947115328636715040578866337902750481566354238661203768010560056939935696678829394884407208311246423715319737062188883946712432742638151109800623047059726541476042502884419075341171231440736956555270413618581675255342293149119973622969239858152417678164812112068607",
    )?;
    let fastnum_max_converted = bigint_to_i1024(&bignum_max_converted)?;
    assert_eq!(bignum_max_converted, bignum_max_expected, "value=I1024::MAX");
    assert_eq!(fastnum_max_converted, fastnum_max_expected, "value=I1024::MAX");

    let fastnum_min_expected = I1024::MIN;
    let bignum_min_converted = bigint_from_i1024(fastnum_min_expected)?;
    let bignum_min_expected = BigInt::from_str(
        "-89884656743115795386465259539451236680898848947115328636715040578866337902750481566354238661203768010560056939935696678829394884407208311246423715319737062188883946712432742638151109800623047059726541476042502884419075341171231440736956555270413618581675255342293149119973622969239858152417678164812112068608",
    )?;
    let fastnum_min_converted = bigint_to_i1024(&bignum_min_converted)?;
    assert_eq!(bignum_min_converted, bignum_min_expected, "value=I1024::MIN");
    assert_eq!(fastnum_min_converted, fastnum_min_expected, "value=I1024::MIN");
    Ok(())
}
