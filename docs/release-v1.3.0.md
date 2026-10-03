# Game Settings Master v1.3.0

Предпросмотр изменений Unreal Engine, расширенные бэкапы и пресеты, выбор видеокарты, диагностика после игры и новые источники установок.

**Дата:** 2026-10-03

## Новые возможности

- Перед применением показывается список «было → станет», включая добавления и удаления. Отдельные правки можно исключить; связанные параметры выбираются вместе. Если конфиги или контекст игры изменились, предпросмотр требуется обновить.
- Именованные снимки конфигов, сравнение снимков между собой и с текущими файлами. Восстановление отдельного параметра, файла или полного снимка проходит через предпросмотр.
- Пресеты сохраняют выбранные правки или текущий профиль редактируемых настроек, включая черновик. Добавлены описание, сборка игры, версия UE и требования к GPU. Старые JSON-пресеты продолжают читаться.
- Выбор GPU для каждой игры, отдельное отображение выделенной и общей памяти. Аппаратная трассировка определяется через D3D12 независимо от DLSS, включая AMD и Intel. Выбор GPU влияет на проверку настроек; назначение видеокарты самой игре остаётся в Windows или настройках игры.
- Пока GSM открыт, диагностика отслеживает запуск и завершение игры, затем проверяет изменённые параметры и активную папку конфигов. Последний результат сохраняется между запусками приложения.
- Обнаружение установленных игр GOG и доступных установок Xbox/Game Pass. Зарегистрированные пакеты запускаются через Windows; для ручных игр и GOG можно выбрать EXE внутри папки установки.
- Редактор классических ActionMappings, AxisMappings и AxisConfig в Input.ini: клавиши, модификаторы, чувствительность и параметры осей. Повторяющиеся привязки отображаются отдельно.

## Исправления

- Работа с INI сохраняет повторяющиеся секции и ключи, комментарии, порядок строк, окончания строк и поддерживаемую кодировку. Исправлено удаление нескольких параметров из разных секций.
- Чтение конфигов, предпросмотр и создание снимков сохраняют исходные атрибуты файлов. При ошибке записи затронутые файлы восстанавливаются из бэкапа; пустой набор правок не создаёт бэкап.
- Вкладка управления показывает фактический путь Input.ini и различает отсутствующий файл, пустой файл и файл без классических привязок. Для PUBG объясняется отдельный формат CustomInputSettins в GameUserSettings.ini.
- Раскрытые параметры сохранения пресета прокручиваются внутри панели. Кнопки сохранения и применения остаются доступными при длинных предупреждениях и увеличенном шрифте.

## Область поддержки

Enhanced Input и собственные форматы управления игр, включая PUBG CustomInputSettins, пока не редактируются. Неоднозначные операции массивов Input.ini доступны только для просмотра. Обнаружение Xbox ограничено доступными установками; закрытые папки не разблокируются. Наличие подходящей GPU не подтверждает поддержку DLSS/FSR/XeSS конкретной игрой.

Проверки: 238 Rust-тестов, 206 TypeScript-тестов и 21 сценарий Playwright. Аппаратное обнаружение проверено на NVIDIA RTX 4060 Ti и встроенной AMD Radeon; физические Intel, гибридные ноутбуки, реальные запуски новых магазинов и обновление с 1.0.8 требуют дополнительной проверки.

---

Unreal Engine change previews, expanded backups and presets, GPU selection, post-game diagnostics and additional installation sources.

## New features

- Review before/after values, additions and removals before applying changes. Exclude individual edits or select linked parameters together. Changed files or game context require a refreshed preview.
- Name configuration snapshots, compare snapshots or current files, and preview restoration of one parameter, one file or a complete snapshot.
- Save selected edits or the current profile of editable settings, including drafts. Presets include descriptions, game builds, UE versions and GPU requirements. Legacy JSON presets remain supported.
- Select a GPU per game and view dedicated and shared memory separately. D3D12 ray tracing detection works independently of DLSS, including AMD and Intel. Selection affects settings validation; Windows or the game determines which adapter runs the game.
- While GSM is open, diagnostics track game launch and exit, then check applied values and the active configuration folder. The last result persists across app restarts.
- Discover GOG and accessible Xbox/Game Pass installations. Registered packages use native Windows activation; manual games and GOG support a selected executable inside the installation folder.
- Edit classic ActionMappings, AxisMappings and AxisConfig in Input.ini, including keys, modifiers, sensitivity and axis properties. Repeated bindings retain separate identities.

## Fixes

- INI changes preserve duplicate sections and keys, comments, line order, line endings and supported encodings. Multiple parameter removals across sections are handled correctly.
- Reading, previewing and snapshot creation preserve original file attributes. Failed writes restore affected files; empty selections create no backup.
- The controls tab shows the resolved Input.ini path and distinguishes missing files, empty files and files without classic bindings. PUBG's CustomInputSettins in GameUserSettings.ini is identified as a separate format.
- Expanded preset metadata scrolls within its panel. Save/apply controls remain accessible with long warnings and larger text.

## Coverage

Enhanced Input and game-specific control formats, including PUBG CustomInputSettins, are not yet editable. Ambiguous Input.ini array operations are read-only. Xbox discovery covers accessible installations without changing protected folder permissions. GPU hardware does not establish a game's DLSS/FSR/XeSS plugin support.

Validation: 238 Rust tests, 206 TypeScript tests and 21 Playwright scenarios. Hardware detection was exercised on NVIDIA RTX 4060 Ti and integrated AMD Radeon. Physical Intel and hybrid laptops, actual launches from the additional stores and upgrade from 1.0.8 still need additional validation.
