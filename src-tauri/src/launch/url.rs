pub(crate) fn open_launch_url(url: &str) -> Result<(), String> {
    open::that(url).map_err(|e| {
        crate::i18n::t(
            &format!("Не удалось открыть лаунчер: {e}"),
            &format!("Failed to open launcher: {e}"),
        )
    })
}
