use super::*;
pub(super) fn render(
    plan: &Plan,
    selected: &[ChangeOperation],
) -> Result<BTreeMap<String, Option<Vec<u8>>>, AppInvokeError> {
    let mut writes = BTreeMap::new();
    for op in selected {
        if matches!(op.kind, OperationKind::File | OperationKind::FileText) {
            writes.insert(op.file.clone(), plan.replacements[&op.file].clone());
        } else {
            let before = plan.originals.get(&op.file).and_then(Option::as_ref);
            let current = writes
                .entry(op.file.clone())
                .or_insert_with(|| before.cloned());
            let text = decode(current.as_ref())?;
            let mut document = Document::parse(&text);
            if let Some(line) = op.line.map(|line| line as usize) {
                let raw = document
                    .lines
                    .get_mut(line)
                    .ok_or_else(|| error("Привязка изменилась", "Binding changed"))?;
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
                    .unwrap_or(&op.key);
                *raw = format!(
                    "{prefix}={}{ending}",
                    op.after.as_deref().unwrap_or_default()
                );
            } else {
                document.set(&op.section, &op.key, op.after.as_deref());
            }
            *current = Some(encode_like(
                &document.text(),
                before,
                plan.originals
                    .get("GameUserSettings.ini")
                    .and_then(Option::as_ref),
            ));
        }
    }
    writes.retain(|file, bytes| plan.originals.get(file) != Some(bytes));
    Ok(writes)
}
