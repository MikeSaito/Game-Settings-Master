use super::{decode_bytes, encode_bytes, IniEncoding};

#[test]
fn roundtrip_utf16_le() {
    let text = "[ScalabilityGroups]\r\nsg.ShadowQuality=1\r\n";
    let encoded = encode_bytes(text, IniEncoding::Utf16Le);
    let (decoded, enc) = decode_bytes(&encoded).unwrap();
    assert_eq!(enc, IniEncoding::Utf16Le);
    assert_eq!(decoded, text);
}
