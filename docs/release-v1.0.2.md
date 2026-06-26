# Game Settings Master v1.0.2 — Release Notes

[Русский](#русский) · [English](#english)

---

## Русский

**Дата:** 2026-06-21  
**Тип:** bugfix / performance / catalog

### Исправления

- **Библиотека:** навигация через React Router `<Link>`, состояние панелей в `sessionStorage` — кнопка «Библиотека» работает с первого клика после возврата из игры
- **Окно Tauri:** `hide`/`show` перенесены в Rust-бэкенд; исправлена ошибка `window.hide not allowed`
- **GameWorkspaceContext:** устранён бесконечный цикл re-render (`Maximum update depth exceeded`)
- **Dev-режим:** окно не скрывается при потере фокуса во время разработки
- **Каталог:** исправлены повреждённые описания с `????`; улучшен humanize для CamelCase и аббревиатур (FPS, LOD, VSM, Lumen, Nanite)
- **Rust:** заменён regex с lookahead в `split_identifier_part` — устранён panic при загрузке каталога

### Производительность

- Кэш поисковых строк в расширенном редакторе
- Модульная фильтрация и сортировка с лучшей мемоизацией
- Стабильные props строк списка параметров (меньше лишних re-render)

### Каталог

- `display_overrides.json` — ручные RU/EN тексты для ключевых параметров
- Tier A v6 расширение и синхронизация humanize между Python, Rust и frontend

---

## English

**Date:** 2026-06-21  
**Type:** bugfix / performance / catalog

### Fixes

- **Library:** React Router `<Link>` navigation and `sessionStorage` panel state — Library button works on first click after returning from a game
- **Tauri window:** `hide`/`show` moved to Rust backend; fixed `window.hide not allowed` error
- **GameWorkspaceContext:** fixed infinite re-render loop (`Maximum update depth exceeded`)
- **Dev mode:** window no longer hides on focus loss during development
- **Catalog:** fixed corrupted `????` descriptions; improved CamelCase and acronym humanization (FPS, LOD, VSM, Lumen, Nanite)
- **Rust:** replaced regex with lookahead in `split_identifier_part` — fixed panic on catalog load

### Performance

- Search string cache in the advanced editor
- Modular filtering and sorting with better memoization
- Stable parameter list row props (fewer unnecessary re-renders)

### Catalog

- `display_overrides.json` — hand-written RU/EN texts for key parameters
- Tier A v6 expansion and humanize sync across Python, Rust, and frontend
