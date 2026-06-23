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
- [`src/components/ds/index.ts`](../src/components/ds/index.ts) — barrel design system (по желанию)

### `src/lib/` — доменные модули

| Папка | Что класть |
|-------|------------|
| [`api/`](../src/lib/api/) | Tauri `invoke`, bindings, tauri runtime/dialog |
| [`core/`](../src/lib/core/) | Типы, ошибки, `cn`, queryClient, `APP_VERSION` |
| [`routing/`](../src/lib/routing/) | React Router paths, editor panels, `openGameEditor`, navigation |
| [`editor/`](../src/lib/editor/) | CVars, фильтры, humanize, зависимости параметров |
| [`game/`](../src/lib/game/) | GameProfile, covers, prefetch workspace |
| [`gpu/`](../src/lib/gpu/) | GPU-aware visibility и фильтры |
| [`settings/`](../src/lib/settings/) | Настройки приложения (тема, язык) |

В каждой подпапке — `index.ts` (barrel). Тесты `*.test.ts` лежат рядом с кодом.

### Редактор и бэкапы

- Единый URL редактора: `/game/:id/advanced`
- Активная панель (`basic` | `advanced` | `backups`) — в `sessionStorage` (`gsm-editor-panel:*`)
- Legacy `/game/:id/backups` → redirect на `/advanced` + panel `backups`
- UI бэкапов встроен в [`AdvancedEditor`](../src/pages/AdvancedEditor.tsx) (`Backups` с `embedded`)
- Переход в редактор: [`openGameEditor()`](../src/lib/routing/navigation.ts)

### `src/components/` — UI по фичам

| Папка | Содержимое |
|-------|------------|
| `advanced/` | EditorModeBar, EditorSidebar, ParameterList, apply bar |
| `backups/` | BackupRow, embedded backups panel |
| `library/` | Сетка игр, тулбар |
| `layout/` | AppShell, GameContextBar |
| `settings/` | Панель настроек |
| `ds/` | Design system (кнопки, поля, панели, Toggle) |
| `app/` | ErrorBoundary, UpdateGate, RouteLoading |
| `game/` | GameCover |

Удалён legacy-слой `components/ui/` — новый UI только через `ds/`.

### `src/hooks/`

| Папка | Хуки |
|-------|------|
| `app/` | settings, updater, debounce, background IPC gating |
| `game/` | `useGameLaunch`, `useGameRunning`, `useRunningExeName`, `useBackupMutations`, `useActiveGameIdRef` |
| `editor/` | `useAdvancedEditorState`, `useEditorQueries`, `useEditorFilteredParams`, `useEditorPanelState`, `useEditorMutations` |

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
catalog/       загрузка каталога; humanize.rs — CVar titles/ranges/categories
```

Внутренний код импортирует `crate::core::models`. `lib.rs` по-прежнему re-export'ит `models` и др. для совместимости.

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

Актуальные счётчики: [`merge_stats.json`](../src-tauri/catalog/generated/merge_stats.json).

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

URL установщика собирается в [`site.ts`](../landing/src/lib/site.ts) из `VITE_APP_VERSION`.

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
6. **`__APP_VERSION__`** — глобал из `package.json` через `vite.config.ts`; тип в [`src/vite-env.d.ts`](../src/vite-env.d.ts).
