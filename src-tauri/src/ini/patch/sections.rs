pub(crate) struct SectionSpan {
    pub name: String,
    pub start: usize,
    pub end: usize,
}

pub(crate) fn scan_sections(lines: &[String]) -> Vec<SectionSpan> {
    let mut sections = Vec::new();
    let mut current: Option<(String, usize)> = None;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            if let Some((name, start)) = current.take() {
                sections.push(SectionSpan {
                    name,
                    start,
                    end: i,
                });
            }
            let name = trimmed[1..trimmed.len() - 1].to_string();
            current = Some((name, i));
        }
    }
    if let Some((name, start)) = current {
        sections.push(SectionSpan {
            name,
            start,
            end: lines.len(),
        });
    }
    sections
}

pub(crate) fn find_section<'a>(sections: &'a [SectionSpan], name: &str) -> Option<&'a SectionSpan> {
    sections.iter().find(|s| s.name.eq_ignore_ascii_case(name))
}

pub(crate) fn line_key(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    if trimmed.is_empty()
        || trimmed.starts_with(';')
        || trimmed.starts_with('#')
        || trimmed.starts_with('[')
    {
        return None;
    }
    let mut parts = trimmed.splitn(2, '=');
    let key = parts.next()?.trim();
    if key.is_empty() {
        return None;
    }
    let _value = parts.next()?;
    Some(key)
}
