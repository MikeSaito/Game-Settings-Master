use super::{decode_bytes, encode_bytes, IniEncoding};

#[test]
fn roundtrip_utf16_le() {
    let text = "[ScalabilityGroups]\r\nsg.ShadowQuality=1\r\n";
    let encoded = encode_bytes(text, IniEncoding::Utf16Le);
    let (decoded, enc) = decode_bytes(&encoded).unwrap();
    assert_eq!(enc, IniEncoding::Utf16Le);
    assert_eq!(decoded, text);
}

#[test]
fn rejects_truncated_and_invalid_utf16_without_replacing_bytes() {
    assert!(decode_bytes(&[0xff, 0xfe, 0x41]).is_err());
    assert!(decode_bytes(&[0xff, 0xfe, 0x00, 0xd8]).is_err());
}
