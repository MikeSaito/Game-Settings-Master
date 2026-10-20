# Game Settings Master v1.4.0

Текстовый редактор INI в отдельной вкладке, сохранение через предпросмотр и обновления зависимостей.

**Дата:** 2026-10-20

## Новые возможности

- Вкладка «Файлы» открывает Engine.ini, GameUserSettings.ini, Scalability.ini, Game.ini и Input.ini выбранной игры как обычный текст.
- Поиск по файлу, отдельные черновики для каждого INI и предпросмотр всех изменённых файлов по Ctrl+S. При переключении файлов и вкладок редактора черновики сохраняются в памяти.
- Перед записью показывается полный текст «было → станет». Можно исключить любой файл; его черновик останется для дальнейшей работы. Отсутствующий файл создаётся при сохранении непустого текста.
- Сохраняются комментарии, повторяющиеся секции и ключи, порядок операций INI-массивов, исходная кодировка и переводы строк вне изменённого участка. Поддерживаются UTF-8, UTF-8 BOM и UTF-16 LE, до 1 МБ на файл.

## Надёжность и безопасность

- Изменение файла на диске или активной папки конфигов блокирует запись устаревшего черновика. Перечитывание файла с несохранёнными правками требует явного отказа от черновика.
- Сохранение использует общий механизм предпросмотра, автоматических бэкапов и отката при ошибке. Пока игра запущена, файлы можно читать, а запись заблокирована.
- Диагностика отслеживает изменения сохранённых файлов по контрольным суммам. Полный текст INI в данные диагностики не записывается.
- Обновлены зависимости приложения и Tauri, включая исправление безопасности TLS в механизме обновления.

## Особенности

Черновики хранятся, пока открыт редактор выбранной игры. Файл применяется целиком; ручные изменения требуют проверки синтаксиса пользователем. Экспорт полного текста в структурированные пресеты не предусмотрен.

Windows x64. Подпись обновления проверяется приложением; подписи издателя Windows у установщика нет.

Проверено: 243 теста Rust (ещё один существующий тест пропущен), 213 тестов фронтенда, 23 сценария Playwright на RU/EN, 12 тестов сборщика каталога, production-сборка интерфейса, Clippy, форматирование и синхронизация типов. Сценарии записи, кодировок и отката проверены на временных конфигурациях. Установка поверх предыдущей версии на реальной игровой системе отдельно не проверялась.

---

## English

A dedicated INI text editor, saves through the shared preview, and dependency updates.

### New features

- The Files tab edits Engine.ini, GameUserSettings.ini, Scalability.ini, Game.ini and Input.ini for the selected game as source text.
- Search, separate drafts per file, and Ctrl+S to preview all modified files. Drafts remain in memory when switching files or editor tabs.
- Full before/after text is shown before writing. Individual files can be excluded without losing their drafts. Missing files are created when saving nonempty text.
- Comments, duplicate sections and keys, INI array directive order, source encoding and line endings outside the edited range are preserved. Supports UTF-8, UTF-8 BOM and UTF-16 LE files up to 1 MiB.

### Reliability and security

- Source file or active config folder changes block stale saves. Reloading a modified file requires explicitly discarding its draft.
- Saves share the existing preview, automatic backup and rollback transaction. Reading remains available while the game runs; writing is blocked.
- Diagnostics track whole-file checksums without storing raw INI text.
- Application and Tauri dependencies were updated, including a TLS security fix in the updater.

### Notes

Drafts remain while the selected game editor is open. Files are applied in full; users must review manual syntax changes. Raw text is not exported to structured presets.

Windows x64. Updater signatures are verified by the app; the installer has no Windows publisher signature.

Validation: 243 Rust tests passed (one existing test ignored), 213 frontend tests, 23 RU/EN Playwright scenarios, 12 catalog builder tests, production frontend build, Clippy, formatting and generated bindings. Writes, encoding and rollback scenarios use temporary configs. An in-place upgrade on an actual gaming system has not been separately validated.
