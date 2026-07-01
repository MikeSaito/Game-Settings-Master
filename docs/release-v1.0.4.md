# Game Settings Master v1.0.4 — Release Notes

[Русский](#русский) · [English](#english)

---

## Русский

**Дата:** 2026-07-01  
**Тип:** minor — качество, E2E, конфликты sg/r, crash reports, синхронизация ini

### Новое

- **E2E (Playwright):** три сценария в CI — apply/restore из Basic, разрешение конфликта sg/r с применением, сброс override ini из бэкапов (`e2e/`, `src/e2e/mockInvoke.ts`, `playwright.config.ts`)
- **Opt-in crash reports:** локальное сохранение отчётов об ошибках без behavior tracking — Rust-модуль `crash_report/`, UI в настройках (`CrashReportsSection`), `ErrorBoundary` с отправкой stack trace при согласии пользователя
- **Список всех crash reports:** в настройках — все сохранённые отчёты (не только последний), GitHub / Copy на каждой строке
- **Actionable UI sg ↔ r.\*:** панель конфликтов (`SgEngineConflictPanel`) — tier preview, «сбросить r.\*, оставить sg.\*», i18n RU/EN
- **Workspace badge:** бейдж «только GUS» когда нет override ini (Engine / Scalability / Game / Input / DeviceProfiles)
- **Единый канон ini-файлов:** Rust `ALLOWED_CONFIG_INI_FILES` / `OVERRIDE_INI_FILES` в `path_safety.rs`, зеркало на frontend в `src/lib/ini/configFiles.ts`; `get_game_config` читает все 6 ini
- **Ini toggle в редакторе:** переключатель для Engine.ini, Scalability.ini, Game.ini (не только GUS)

### Исправления и качество

- **Flaky catalog test:** `serial_test` + `reset_catalog_build_count()` для `catalog_index_is_reused_for_same_engine_family`
- **Removals на Basic panel:** `buildCustomChanges` учитывает removals для Scalability/Game.ini; фильтр панели внутри функции (full draft для removals)
- **Pending keys конфликтов:** removals не считаются active override при детекте sg/r
- **Prefix matching sg→r:** `sgQualityToRPrefix` + `matchesSgRPrefixFamily` — не ловит посторонние ключи вроде `r.TextureStreamingPoolSize`
- **ErrorBoundary:** `componentStack` в payload; «saved» только после успешного submit crash report
- **Backups UI:** кнопка «Удалить Engine / Scalability ini» вместо «Сброс (только GUS)»; в описании — полный список override ini включая DeviceProfiles
- **Landing CSP:** meta CSP для production; `'unsafe-inline'` для Yandex Metrika
- **IDE:** `.vscode/settings.json` — `typescript.tsdk` → `node_modules/typescript/lib` (убирает ложную ошибку `ignoreDeprecations: "6.0"`)

### Ревью перед релизом

| Проверка | Результат |
|----------|-----------|
| Bugbot (финальный diff) | ⚠️ medium — CSP лендинга может блокировать доп. скрипты Metrika с `yastatic.net` и geo `connect-src`; при необходимости расширить директивы |
| Security review | ✅ medium+ не найдено |
| `npm test -- --run` | ✅ 136/136 |
| `cargo test` (полный параллельный) | ⚠️ 159/160 — `catalog_index_is_reused_for_same_engine_family` нестабилен при полном прогоне; изолированно и с `--test-threads=1` проходит |
| `npm run e2e` | ✅ 3/3 |
| IPC / guard (backups, config, covers) | ✅ ужесточены path checks в diff |

### Версии

| Файл | Версия |
|------|--------|
| `package.json` | 1.0.4 |
| `src-tauri/Cargo.toml` | 1.0.4 |
| `src-tauri/tauri.conf.json` | 1.0.4 |
| `landing/package.json` | 1.0.4 |
| README / landing | 1.0.4 |

### Установщик

`Game-Settings-Master_1.0.4_x64-setup.exe` — через GitHub Actions (`workflow_dispatch`) после push тега/релиза.

### Ключевые файлы изменений

| Область | Пути |
|---------|------|
| Конфликты sg/r | `src/lib/editor/sgEngineConflicts.ts`, `SgEngineConflictPanel.tsx` |
| Apply / removals | `src/lib/editor/buildCustomChanges.ts`, `engineParams.ts` |
| Ini lists | `src-tauri/src/fs_util/path_safety.rs`, `src/lib/ini/configFiles.ts` |
| Crash reports | `src-tauri/src/crash_report/`, `CrashReportsSection.tsx`, `useCrashReporting.ts` |
| E2E | `e2e/apply-restore.spec.ts`, `src/e2e/` |

---

## English

**Date:** 2026-07-01  
**Type:** minor — quality, E2E, sg/r conflicts, crash reports, ini sync

### New

- **E2E (Playwright):** three CI scenarios — basic apply/restore, sg/r conflict resolution + apply, override ini reset from backups (`e2e/`, `src/e2e/mockInvoke.ts`, `playwright.config.ts`)
- **Opt-in crash reports:** local error reports without behavior tracking — Rust `crash_report/`, settings UI (`CrashReportsSection`), `ErrorBoundary` with stack trace when the user opts in
- **All crash reports list:** settings show every saved report, not only the latest, with per-row GitHub / Copy actions
- **Actionable sg ↔ r.\* UI:** conflict panel (`SgEngineConflictPanel`) — tier preview, “reset r.\*, keep sg.\*”, RU/EN i18n
- **Workspace badge:** “GUS only” badge when no override ini files are present
- **Single ini canon:** Rust `ALLOWED_CONFIG_INI_FILES` / `OVERRIDE_INI_FILES` in `path_safety.rs`, mirrored in `src/lib/ini/configFiles.ts`; `get_game_config` reads all six ini files
- **Ini toggle in editor:** Engine.ini, Scalability.ini, Game.ini (not GUS-only)

### Fixes & quality

- **Flaky catalog test:** `serial_test` + `reset_catalog_build_count()` for `catalog_index_is_reused_for_same_engine_family`
- **Basic panel removals:** `buildCustomChanges` handles Scalability/Game.ini removals; panel filter applied inside the function (full draft for removals)
- **Conflict pending keys:** removals are not counted as active overrides in sg/r detection
- **sg→r prefix matching:** `sgQualityToRPrefix` + `matchesSgRPrefixFamily` — avoids false positives like `r.TextureStreamingPoolSize`
- **ErrorBoundary:** `componentStack` in payload; “saved” only after successful crash report submit
- **Backups UI:** button “Remove Engine / Scalability ini” instead of “Reset (GUS only)”; full override ini list including DeviceProfiles in copy
- **Landing CSP:** production meta CSP; `'unsafe-inline'` for Yandex Metrika
- **IDE:** `.vscode/settings.json` — `typescript.tsdk` → fixes spurious `ignoreDeprecations: "6.0"` IDE warning

### Pre-release review

| Check | Result |
|-------|--------|
| Bugbot (final diff) | ⚠️ medium — landing CSP may block extra Metrika scripts from `yastatic.net` and geo-specific `connect-src`; widen directives if analytics are required |
| Security review | ✅ no medium+ findings |
| `npm test -- --run` | ✅ 136/136 |
| `cargo test` (full parallel) | ⚠️ 159/160 — `catalog_index_is_reused_for_same_engine_family` flaky in full parallel run; passes isolated / with `--test-threads=1` |
| `npm run e2e` | ✅ 3/3 |
| IPC / guard (backups, config, covers) | ✅ tightened path checks in diff |

### Versions

| File | Version |
|------|---------|
| `package.json` | 1.0.4 |
| `src-tauri/Cargo.toml` | 1.0.4 |
| `src-tauri/tauri.conf.json` | 1.0.4 |
| `landing/package.json` | 1.0.4 |
| README / landing | 1.0.4 |

### Installer

`Game-Settings-Master_1.0.4_x64-setup.exe` — via GitHub Actions (`workflow_dispatch`) after pushing the release.

### Key changed areas

| Area | Paths |
|------|-------|
| sg/r conflicts | `src/lib/editor/sgEngineConflicts.ts`, `SgEngineConflictPanel.tsx` |
| Apply / removals | `src/lib/editor/buildCustomChanges.ts`, `engineParams.ts` |
| Ini lists | `src-tauri/src/fs_util/path_safety.rs`, `src/lib/ini/configFiles.ts` |
| Crash reports | `src-tauri/src/crash_report/`, `CrashReportsSection.tsx`, `useCrashReporting.ts` |
| E2E | `e2e/apply-restore.spec.ts`, `src/e2e/` |
