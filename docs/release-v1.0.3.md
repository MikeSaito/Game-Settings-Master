# Game Settings Master v1.0.3 — Release Notes

[Русский](#русский) · [English](#english)

---

## Русский

**Дата:** 2026-06-27  
**Тип:** minor — UX, discovery, i18n, SEO, стабильность

### Новое

- **Язык по системе:** RU только при русской локали ОС; для всех остальных — EN (frontend + Rust-бэкенд через `sys-locale`)
- **Steam на всех дисках:** поиск библиотек Steam на локальных и съёмных дисках A–Z, не только на системном
- **Подсказка по конфигу (`ConfigPathHelp`):** в библиотеке, редакторе и бэкапах — как указать `Saved/Config/Windows`, примеры путей
- **Редактор:** фильтр по умолчанию — «Есть в ini» (`ini_only`); порядок фильтров в сайдбаре обновлён
- **Лендинг gsm-tool.com:** hreflang RU/EN, sitemap с alternate, robots.txt с Host, OG/Twitter meta; авто-редирект не-RU посетителей с `/` на `/en/` (боты не трогаем)
- **Брендинг:** обновлённые PNG-иконки (Tauri, favicon, og-image) из `public/logo.png`; в UI — SVG

### Исправления

- **React:** устранены циклы `Maximum update depth exceeded` в `useAppSettings`, `useEditorQueries`, `useEditorParamDraft`, `useAdvancedEditorState`, `AppWindowFocusProvider`
- **i18n:** синхронизация языка frontend ↔ backend без лишних `applyAppSettings` и гонок при старте
- **Rust-тесты:** backup/presets — двуязычные assert под новый дефолт EN

### Версии

| Файл | Версия |
|------|--------|
| `package.json` | 1.0.3 |
| `src-tauri/Cargo.toml` | 1.0.3 |
| `src-tauri/tauri.conf.json` | 1.0.3 |
| `landing/package.json` | 1.0.3 |
| README / landing | 1.0.3 |

### Проверки перед релизом

| Проверка | Результат |
|----------|-----------|
| `npm test -- --run` | ✅ 118/118 |
| `cargo test` (полный прогон) | ⚠️ 156/157 — `catalog_index_is_reused_for_same_engine_family` нестабилен при параллельном запуске; изолированно проходит |
| `landing` build | ✅ |
| Версии согласованы | ✅ |
| Форматирование `landing/src/main.ts` | ✅ исправлено |

### Установщик

`Game-Settings-Master_1.0.3_x64-setup.exe` — через GitHub Actions (`workflow_dispatch`) после push тега/релиза.

---

## English

**Date:** 2026-06-27  
**Type:** minor — UX, discovery, i18n, SEO, stability

### New

- **System language:** RU only when the OS locale is Russian; English for everything else (frontend + Rust backend via `sys-locale`)
- **Steam on all drives:** Steam library discovery on local and removable drives A–Z, not just the system drive
- **Config path help (`ConfigPathHelp`):** in library, editor, and backups — how to point at `Saved/Config/Windows`, with path examples
- **Editor:** default filter is “In ini” (`ini_only`); filter order in the sidebar updated
- **Landing gsm-tool.com:** hreflang RU/EN, sitemap with alternates, robots.txt Host, OG/Twitter meta; auto-redirect non-Russian visitors from `/` to `/en/` (search bots excluded)
- **Branding:** updated PNG icons (Tauri, favicon, og-image) from `public/logo.png`; SVG in the UI

### Fixes

- **React:** fixed `Maximum update depth exceeded` loops in `useAppSettings`, `useEditorQueries`, `useEditorParamDraft`, `useAdvancedEditorState`, `AppWindowFocusProvider`
- **i18n:** frontend ↔ backend language sync without redundant `applyAppSettings` or startup races
- **Rust tests:** backup/presets — bilingual asserts aligned with EN default

### Versions

| File | Version |
|------|---------|
| `package.json` | 1.0.3 |
| `src-tauri/Cargo.toml` | 1.0.3 |
| `src-tauri/tauri.conf.json` | 1.0.3 |
| `landing/package.json` | 1.0.3 |
| README / landing | 1.0.3 |

### Pre-release checks

| Check | Result |
|-------|--------|
| `npm test -- --run` | ✅ 118/118 |
| `cargo test` (full run) | ⚠️ 156/157 — `catalog_index_is_reused_for_same_engine_family` flaky under parallel run; passes in isolation |
| `landing` build | ✅ |
| Version strings aligned | ✅ |
| `landing/src/main.ts` formatting | ✅ fixed |

### Installer

`Game-Settings-Master_1.0.3_x64-setup.exe` — via GitHub Actions (`workflow_dispatch`) after pushing the release.
