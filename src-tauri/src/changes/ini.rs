use super::*;
pub(crate) fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub(crate) fn read_optional(dir: &Path, file: &str) -> Result<Option<Vec<u8>>, String> {
    let path = crate::fs_util::safe_child_path(dir, file)?;
    if !path.exists() {
        return Ok(None);
    }
    crate::fs_util::ensure_safe_child_file(dir, &path)?;
    if std::fs::metadata(&path).map_err(|e| e.to_string())?.len() > 16 * 1024 * 1024 {
        return Err(crate::i18n::t(
            "INI-файл превышает 16 МБ",
            "INI file exceeds 16 MB",
        ));
    }
    crate::fs_util::read_file_bytes(&path).map(Some)
}

pub(super) fn decode(bytes: Option<&Vec<u8>>) -> Result<String, String> {
    bytes
        .map(|bytes| crate::ini::encoding::decode_bytes(bytes).map(|(text, _)| text))
        .transpose()
        .map(|text| text.unwrap_or_default())
}

pub(crate) fn encode_like(
    text: &str,
    original: Option<&Vec<u8>>,
    hint: Option<&Vec<u8>>,
) -> Vec<u8> {
    let source = original.or(hint);
    let encoding = source
        .and_then(|bytes| crate::ini::encoding::decode_bytes(bytes).ok())
        .map(|(_, encoding)| encoding)
        .unwrap_or(crate::ini::encoding::IniEncoding::Utf8);
    let mut bytes = crate::ini::encoding::encode_bytes(text, encoding);
    if encoding == crate::ini::encoding::IniEncoding::Utf8
        && source.is_some_and(|bytes| bytes.starts_with(&[0xef, 0xbb, 0xbf]))
    {
        bytes.splice(0..0, [0xef, 0xbb, 0xbf]);
    }
    bytes
}
