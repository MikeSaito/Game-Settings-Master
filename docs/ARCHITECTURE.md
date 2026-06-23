# Architecture — Game Settings Master

Карта репозитория для контрибьюторов. Поведение приложения не меняется при переносе файлов — важны границы модулей и соглашения об импортах.

## Обзор

| Область | Путь | Назначение |
|---------|------|------------|
| Desktop UI | [`src/`](../src/) | React + Tauri frontend |
| Backend | [`src-tauri/src/`](../src-tauri/src/) | Rust: IPC, ini, discovery, catalog |
| Landing | [`landing/src/`](../landing/src/) | Статический маркетинговый сайт (GitHub Pages) |
| Catalog builder | [`tools/ue-catalog-builder/`](../tools/ue-catalog-builder/) | Python: сборка `ue_reference_index.json` |
| UE snapshots | [`tools/ue-reference/`](../tools/ue-reference/) | Локальные ini-фикстуры и клоны движка |

## Frontend (`src/`)

### Импорты

- Алиас **`@/`** → `src/` (см. [`tsconfig.json`](../tsconfig.json), [`vite.config.ts`](../vite.config.ts))
- Пример: `import { getGameParameters } from "@/lib/api"` или `import { cn } from "@/lib/core"`
- Корневой barrel [`src/lib/index.ts`](../src/lib/index.ts) — `core`, `routing`, `editor`, `game`, `gpu`, `settings`; IPC отдельно из `@/lib/api`

### `src/lib/` — доменные модули

| Папка | Что класть |
|-------|------------|
| [`api/`](../src/lib/api/) | Tauri `invoke`, bindings, tauri runtime/dialog |
| [`core/`](../src/lib/core/) | Типы, ошибки, `cn`, queryClient |
| [`routing/`](../src/lib/routing/) | React Router paths, editor panels, navigation |
| [`editor/`](../src/lib/editor/) | CVars, фильтры, humanize, зависимости параметров |
| [`game/`](../src/lib/game/) | GameProfile, covers, prefetch workspace |
| [`gpu/`](../src/lib/gpu/) | GPU-aware visibility и фильтры |
| [`settings/`](../src/lib/settings/) | Настройки приложения (тема, язык) |

В каждой подпапке — `index.ts` (barrel). Тесты `*.test.ts` лежат рядом с кодом.

### `src/components/` — UI по фичам

| Папка | Содержимое |
|-------|------------|
| `advanced/` | Расширенный редактор |
| `library/` | Сетка игр, тулбар |
| `layout/` | AppShell, header, sidebar |
| `settings/` | Панель настроек |
| `ds/` | Design system (кнопки, поля, панели, Toggle) |
| `app/` | ErrorBoundary, UpdateGate |
| `game/` | GameCover |

Удалён legacy-слой `components/ui/` — новый UI только через `ds/`.

### `src/hooks/`

| Папка | Хуки |
|-------|------|
| `app/` | settings, updater, debounce, background IPC gating |
| `game/` | running exe, game running poll |
| `editor/` | `useAdvancedEditorState` |

### Куда добавлять новый код

| Задача | Куда |
|--------|------|
| Новый IPC-метод | `src-tauri/src/commands/` + `src/lib/api/index.ts` |
| Новый CVar-фильтр | `src/lib/editor/` |
| Новый экран/панель | `src/components/<feature>/` + `src/pages/` |
| Новый роут | `src/lib/routing/routes.ts` + `src/App.tsx` |

## Rust backend (`src-tauri/src/`)

```
core/          app_error, models, process_util, resource_paths
i18n/          RU/EN строки backend
profiles/      сохранённые профили игр
commands/      Tauri IPC handlers
discovery/     Steam/Epic scan, UE detect
ini/           parse / write / patch ini
catalog/       загрузка и humanize каталога
```

`crate::models`, `crate::i18n`, `crate::profiles` — re-export из `lib.rs` для обратной совместимости.

## Catalog builder (`tools/ue-catalog-builder/`)

```
build.py           entry point
extract/           sg.* и GUS extractors
tier_a/            генераторы tier A overlay
analysis/          gaps, coverage
fixtures/          gen_fixtures.py
shared/            ue_versions.json
data/              tier overlays, display_overrides
generated/         промежуточные registry JSON
```

```powershell
npm run catalog:build
npm run catalog:test
```

## Landing (`landing/src/components/`)

```
layout/     SiteHeader, SiteFooter
sections/   Hero, Stats, FAQ, Download, …
shared/     CtaButtons, DownloadModal
```

## Проверка перед PR

```powershell
npm test
npm run build
cd src-tauri; cargo test
python tools/ue-catalog-builder/test_build.py
npm run landing:build
```

## Соглашения

1. **Не смешивать домены** — editor-логика не в `game/`, IPC не в `editor/`.
2. **Barrel `index.ts`** — публичный API модуля; внутри папки можно использовать относительные импорты.
3. **Co-located tests** — тест рядом с модулем, не в отдельном дереве зеркал.
4. **Без изменения данных каталога** в архитектурных PR — `src-tauri/catalog/*.json` только осознанно.
5. **`npm run types:gen`** пишет в `src/lib/api/bindings.ts` (не в корень `lib/`).
