use super::framing::{Reassembler, frames};
#[test]
fn test_framing_and_malformed_fragments() -> anyhow::Result<()> {
    for (hid, size) in [(true, 64), (false, 20), (false, 244)] {
        for n in [2, 57, 58, 59, 64, 98, 100, 255, 260] {
            let bytes = vec![0xa5; n];
            let packets = frames(&bytes, size, hid)?;
            let mut parser = Reassembler::new(hid);
            let mut result = None;
            for p in &packets {
                result = parser.push(p)?;
            }
            assert_eq!(result, Some(bytes));
        }
        let packet = frames(&[0x90, 0], size, hid)?.remove(0);
        let mut bad_sequence = packet.clone();
        bad_sequence[if hid { 4 } else { 2 }] = 1;
        assert!(Reassembler::new(hid).push(&bad_sequence).is_err());
        let mut bad_tag = packet;
        bad_tag[if hid { 2 } else { 0 }] = 4;
        assert!(Reassembler::new(hid).push(&bad_tag).is_err());
    }
    assert_eq!(frames(&[0xe0, 3, 0, 0, 0], 20, false)?, vec![vec![5, 0, 0, 0, 5, 0xe0, 3, 0, 0, 0]]);
    assert!(Reassembler::new(false).push(&[5, 0, 0, 0, 2, 0x90, 0, 0]).is_err());
    assert!(Reassembler::new(false).push(&[5, 0, 0, 255, 255]).is_err());
    Ok(())
}
