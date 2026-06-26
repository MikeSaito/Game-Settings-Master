use std::path::Path;

pub fn clear_readonly(path: &Path) {
    if !path.exists() {
        return;
    }
    if let Ok(meta) = std::fs::metadata(path) {
        let mut perms = meta.permissions();
        if perms.readonly() {
            #[allow(clippy::permissions_set_readonly_false)]
            perms.set_readonly(false);
            let _ = std::fs::set_permissions(path, perms);
        }
    }
}

pub fn format_io_error(
    action_ru: &str,
    action_en: &str,
    path: &Path,
    err: std::io::Error,
) -> String {
    let action = crate::i18n::t(action_ru, action_en);
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.display().to_string());
    let (hint_ru, hint_en) = io_error_hint(&err);
    crate::i18n::t(
        &format!("Не удалось {action} {name}: {err}{hint_ru}"),
        &format!("Failed to {action} {name}: {err}{hint_en}"),
    )
}

fn io_error_hint(err: &std::io::Error) -> (&'static str, &'static str) {
    match err.raw_os_error() {
        Some(5) => (
            ". Доступ запрещён. Полностью закройте игру и лаунчер (Steam/Epic), отключите \
             игровые оверлеи (Steam/Discord/NVIDIA) и проверьте антивирус. Если игра в защищённой \
             папке (Program Files) — запустите приложение от имени администратора. \
             Также снимите атрибут «Только чтение» с файла.",
            ". Access denied. Fully close the game and launcher (Steam/Epic), disable \
             game overlays (Steam/Discord/NVIDIA), and check your antivirus. If the game is in a \
             protected folder (Program Files), run the app as administrator. \
             Also remove the Read-only attribute from the file.",
        ),
        Some(32) => (
            ". Файл занят другим процессом — закройте игру, лаунчер и оверлеи, затем повторите.",
            ". The file is in use by another process — close the game, launcher, and overlays, then try again.",
        ),
        _ => ("", ""),
    }
}
