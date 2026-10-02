//! Ordered, lossless INI document. Scalar views never own the source text.
use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Debug)]
pub struct Document {
    pub lines: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub line: usize,
    pub section: String,
    pub key: String,
    pub value: String,
}

impl Document {
    pub fn parse(text: &str) -> Self {
        Self {
            lines: text.split_inclusive('\n').map(str::to_owned).collect(),
        }
    }

    pub fn text(&self) -> String {
        self.lines.concat()
    }

    pub fn entries(&self) -> Vec<Entry> {
        let mut section = String::new();
        let mut entries = Vec::new();
        for (line, raw) in self.lines.iter().enumerate() {
            let text = raw.trim();
            if text.starts_with('[') && text.ends_with(']') {
                section = text[1..text.len() - 1].to_string();
            } else if !section.is_empty() && !text.starts_with([';', '#']) {
                if let Some((key, value)) = text.split_once('=') {
                    if !key.trim().is_empty() {
                        entries.push(Entry {
                            line,
                            section: section.clone(),
                            key: key.trim().into(),
                            value: value.trim().into(),
                        });
                    }
                }
            }
        }
        entries
    }

    pub fn scalar_values(&self) -> BTreeMap<(String, String), String> {
        let mut values = BTreeMap::new();
        for entry in self.entries() {
            values.insert(
                (
                    entry.section.to_ascii_lowercase(),
                    entry.key.to_ascii_lowercase(),
                ),
                entry.value,
            );
        }
        values
    }

    pub fn occurrence_values(&self) -> BTreeMap<(String, String), String> {
        let entries = self.entries();
        let mut totals = BTreeMap::new();
        for entry in &entries {
            *totals
                .entry((
                    entry.section.to_ascii_lowercase(),
                    entry.key.to_ascii_lowercase(),
                ))
                .or_insert(0usize) += 1;
        }
        let mut counts = BTreeMap::new();
        let mut values = BTreeMap::new();
        for entry in entries {
            let identity = (
                entry.section.to_ascii_lowercase(),
                entry.key.to_ascii_lowercase(),
            );
            let count = counts.entry(identity.clone()).or_insert(0usize);
            *count += 1;
            let key = if totals[&identity] > 1 || entry.key.starts_with(['+', '-', '!', '.']) {
                format!("{} [#{}]", identity.1, count)
            } else {
                identity.1
            };
            values.insert((identity.0, key), entry.value);
        }
        values
    }

    fn newline(&self) -> &'static str {
        if self.lines.iter().any(|line| line.ends_with("\r\n")) {
            "\r\n"
        } else {
            "\n"
        }
    }

    pub fn set(&mut self, section: &str, key: &str, value: Option<&str>) {
        let matches: Vec<_> = self
            .entries()
            .into_iter()
            .filter(|entry| {
                entry.section.eq_ignore_ascii_case(section) && entry.key.eq_ignore_ascii_case(key)
            })
            .collect();
        if !matches.is_empty() {
            // Update every scalar occurrence so later duplicate sections cannot undo a write.
            for entry in matches.into_iter().rev() {
                if let Some(value) = value {
                    let raw = &self.lines[entry.line];
                    let ending = if raw.ends_with("\r\n") {
                        "\r\n"
                    } else if raw.ends_with('\n') {
                        "\n"
                    } else {
                        ""
                    };
                    let prefix = raw
                        .split_once('=')
                        .map(|(prefix, _)| prefix)
                        .unwrap_or(&entry.key);
                    self.lines[entry.line] = format!("{prefix}={value}{ending}");
                } else {
                    self.lines.remove(entry.line);
                }
            }
            return;
        }
        let Some(value) = value else {
            return;
        };
        let newline = self.newline();
        let header = self
            .lines
            .iter()
            .rposition(|line| line.trim().eq_ignore_ascii_case(&format!("[{section}]")));
        if let Some(header) = header {
            let end = self
                .lines
                .iter()
                .enumerate()
                .skip(header + 1)
                .find(|(_, line)| line.trim().starts_with('[') && line.trim().ends_with(']'))
                .map(|(index, _)| index)
                .unwrap_or(self.lines.len());
            if end > 0 && !self.lines[end - 1].ends_with('\n') {
                self.lines[end - 1].push_str(newline);
            }
            self.lines.insert(end, format!("{key}={value}{newline}"));
        } else {
            if let Some(last) = self.lines.last_mut() {
                if !last.ends_with('\n') {
                    last.push_str(newline);
                }
            }
            self.lines
                .push(format!("[{section}]{newline}{key}={value}{newline}"));
        }
    }

    pub fn patch(
        &mut self,
        updates: &indexmap::IndexMap<String, indexmap::IndexMap<String, String>>,
        removals: &HashMap<String, Vec<String>>,
    ) {
        for (section, keys) in removals {
            for key in keys {
                self.set(section, key, None);
            }
        }
        for (section, entries) in updates {
            for (key, value) in entries {
                self.set(section, key, Some(value));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_duplicates_comments_and_endings() {
        let text = "; preamble\r\n[A]\r\n+Keys=One\r\n+Keys=Two\r\n; keep\r\nx=1\r\n[B]\r\ny=2";
        let mut doc = Document::parse(text);
        assert_eq!(doc.text(), text);
        doc.set("A", "x", None);
        doc.set("B", "y", None);
        assert_eq!(
            doc.text(),
            "; preamble\r\n[A]\r\n+Keys=One\r\n+Keys=Two\r\n; keep\r\n[B]\r\n"
        );
    }
    #[test]
    fn duplicate_scalar_sections_are_updated_together() {
        let mut doc = Document::parse("[A]\nx=1\n[a]\nX=2\n");
        doc.set("A", "x", Some(""));
        assert_eq!(doc.text(), "[A]\nx=\n[a]\nX=\n");
    }
}
