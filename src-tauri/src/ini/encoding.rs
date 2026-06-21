use crate::fs_util::{read_file_bytes, write_file_bytes};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IniEncoding {
    Utf8,
    Utf16Le,
}

pub fn read_text(path: &Path) -> Result<(String, IniEncoding), String> {
    let bytes = read_file_bytes(path)?;
    decode_bytes(&bytes)
}

pub fn detect_encoding(path: &Path) -> IniEncoding {
    read_file_bytes(path)
        .ok()
        .map(|bytes| encoding_from_bytes(&bytes))
        .unwrap_or(IniEncoding::Utf8)
}

pub fn write_text(path: &Path, content: &str, encoding: IniEncoding) -> Result<(), String> {
    let bytes = encode_bytes(content, encoding);
    write_file_bytes(path, &bytes)
}

fn encoding_from_bytes(bytes: &[u8]) -> IniEncoding {
    if bytes.starts_with(&[0xFF, 0xFE]) {
        IniEncoding::Utf16Le
    } else {
        IniEncoding::Utf8
    }
}

fn decode_bytes(bytes: &[u8]) -> Result<(String, IniEncoding), String> {
    if bytes.starts_with(&[0xFF, 0xFE]) {
        let units: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();
        Ok((String::from_utf16_lossy(&units), IniEncoding::Utf16Le))
    } else if bytes.starts_with(&[0xFE, 0xFF]) {
        return Err(crate::i18n::t(
            "UTF-16 BE ini пока не поддерживается",
            "UTF-16 BE ini is not supported yet",
        ));
    } else {
        let bytes = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
            &bytes[3..]
        } else {
            bytes
        };
        Ok((
            String::from_utf8(bytes.to_vec()).map_err(|e| {
                crate::i18n::t(
                    &format!("Файл не в UTF-8/UTF-16: {e}"),
                    &format!("File is not UTF-8/UTF-16: {e}"),
                )
            })?,
            IniEncoding::Utf8,
        ))
    }
}

fn encode_bytes(content: &str, encoding: IniEncoding) -> Vec<u8> {
    match encoding {
        IniEncoding::Utf8 => content.as_bytes().to_vec(),
        IniEncoding::Utf16Le => {
            let mut bytes = vec![0xFF, 0xFE];
            for unit in content.encode_utf16() {
                bytes.extend_from_slice(&unit.to_le_bytes());
            }
            bytes
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_utf16_le() {
        let text = "[ScalabilityGroups]\r\nsg.ShadowQuality=1\r\n";
        let encoded = encode_bytes(text, IniEncoding::Utf16Le);
        let (decoded, enc) = decode_bytes(&encoded).unwrap();
        assert_eq!(enc, IniEncoding::Utf16Le);
        assert_eq!(decoded, text);
    }
}
