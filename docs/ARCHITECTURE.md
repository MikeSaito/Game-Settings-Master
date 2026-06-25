# Architecture — Game Settings Master

Карта репозитория для контрибьюторов. Поведение приложения не меняется при переносе файлов — важны границы модулей и соглашения об импортах.

## Обзор

| Область | Путь | Назначение |
|---------|------|------------|
| Desktop UI | [`src/`](../src/) | React + Tauri frontend |
| Backend | [`src-tauri/src/`](../src-tauri/src/) | Rust: IPC, ini, discovery, catalog |
| Landing | [`landing/src/`](../landing/src/) | Статический маркетинговый сайт (GitHub Pages) |
| Catalog builder | [`tools/ue-catalog-builder/`](../tools/ue-catalog-builder/) | Python: сборка `ue_reference_index.json` |
| Shared constants | [`shared/`](../shared/) | JSON для Rust (`include_str!`) и TS (`@shared/`) — rendering key markers и др. |
| UE snapshots | [`tools/ue-reference/`](../tools/ue-reference/) | Локальные ini-фикстуры и клоны движка |

## Frontend (`src/`)

### Импорты

- Алиас **`@/`** → `src/` (см. [`tsconfig.json`](../tsconfig.json), [`vite.config.ts`](../vite.config.ts))
- Пример: `import { getGameParameters } from "@/lib/api"`, `import { Button } from "@/components/ds/Button"`
- Внутри одной папки допустимы относительные `./` импорты; между модулями — `@/`
- Корневой barrel [`src/lib/index.ts`](../src/lib/index.ts) — `core`, `routing`, `editor`, `game`, `gpu`, `settings`; IPC отдельно из `@/lib/api`
- [`src/components/ds/index.ts`](../src/components/ds/index.ts) — barrel design system (по желанию)

### `src/lib/` — доменные модули

| Папка | Что класть |
|-------|------------|
| [`api/`](../src/lib/api/) | Tauri `invoke`, bindings, tauri runtime/dialog |
| [`core/`](../src/lib/core/) | Типы, ошибки, `cn`, queryClient, `APP_VERSION` |
| [`routing/`](../src/lib/routing/) | React Router paths, editor panels, `openGameEditor`, navigation |
| [`editor/`](../src/lib/editor/) | CVars, фильтры, зависимости параметров (`paramDependencies/`) |
| [`shared/`](../src/lib/shared/) | TS-обёртки над JSON из [`shared/`](../shared/) (rendering key markers) |
| [`game/`](../src/lib/game/) | GameProfile, covers, prefetch workspace |
| [`gpu/`](../src/lib/gpu/) | GPU-aware visibility и фильтры |
| [`settings/`](../src/lib/settings/) | Настройки приложения (тема, язык) |

В каждой подпапке — `index.ts` (barrel). Тесты `*.test.ts` лежат рядом с кодом.

### Редактор и бэкапы

- Единый URL редактора: `/game/:id/advanced`
- Активная панель (`basic` | `advanced` | `backups`) — в `sessionStorage` (`gsm-editor-panel:*`)
- Legacy `/game/:id/backups` → redirect на `/advanced` + panel `backups` (как wizard/reshade)
- `GameTabRoute` — только `"advanced"`; бэкапы не отдельный URL-tab
- UI бэкапов встроен в [`AdvancedEditor`](../src/pages/AdvancedEditor.tsx) ([`BackupsPanel`](../src/components/backups/BackupsPanel.tsx))
- Переход в редактор: [`openGameEditor()`](../src/lib/routing/navigation.ts)

### `src/components/` — UI по фичам

| Папка | Содержимое |
|-------|------------|
| `advanced/` | EditorModeBar, EditorSidebar, ParameterList, ParameterRow (+ utils/control), apply bar |
| `backups/` | BackupRow, BackupSectionTitle, BackupsPanel (embedded в редакторе) |
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
| `game/` | `useGameLaunch`, `useGameRunning`, `useRunningExeName`, `useBackupMutations`, `useActiveGameIdRef`, `useSelectedGame` |
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
core/          app_error (AppErrorCode, AppInvokeError), models, process_util, resource_paths
  app_error.rs        structured IPC errors { code, message }
  app_error_tests.rs  error codes + legacy marker roundtrip tests
  resource_paths_tests.rs dev catalog dir smoke test
i18n/          RU/EN строки backend
profiles/      сохранённые профили игр (games.json, overrides.json)
  storage.rs        app_data_dir, profiles_path, write_json_atomic
  trust.rs          validate_profile_paths, resolve_trusted_profile
  persist.rs        load/save/remove profiles, prune_stale_saved_profiles
  overrides.rs      load/save/delete overrides, validate_override_payload
  profiles_tests.rs IPC security + validation tests
backup/        снимки ini перед apply/reset
  paths.rs          backup_store_dir, OVERRIDE_INI_FILES
  snapshot.rs       backup_config_dir, backup_all_targets, list_backups
  restore.rs        restore_backup, rollback_apply_snapshot
  reset.rs          reset_config_all_targets
  backup_tests.rs   restore/reset safety tests
commands/      Tauri IPC handlers (все возвращают Result<_, AppInvokeError>)
  helpers/          IPC validation, trusted paths, custom apply guards
    guard.rs          guard_config_dir_for_write, guard_write_context
    trust.rs          validate_install_dir_for_game
    exe.rs            resolve_write_exe_name
    custom_changes.rs validate_custom_changes_payload
    ipc_tests.rs      guard + payload injection tests
  games/            scan, profile CRUD, covers, config dir
    scan.rs             scan_games
    profile.rs          save/remove game profile
    manual.rs           add_manual_game
    config_dir.rs       set_game_config_dir
    covers.rs           import/remove cover, open folder
    games_tests.rs      IPC guard tests for profile CRUD
  config/           read ini, apply custom, overrides
discovery/     Steam/Epic scan, UE detect
  scan_all.rs       orchestrate steam + epic + manual scan
  dedupe.rs         merge profiles by install_dir / app_id
  manual.rs         add_manual_game_profile
  enrich.rs         post-scan profile enrichment
  discovery_tests.rs dedupe + manual validation tests
  known_games.rs    curated game_id → config hints
  known_games_tests.rs app_id resolution + PUBG config path tests
  mtime_snapshot.rs library folder mtime for cache invalidation
  registry/         cached scan_all + find_game_by_id
    cache.rs          GAME_SCAN_CACHE_TTL, cached_scan_all_games
    lookup.rs         find_game_by_id
    registry_tests.rs cache TTL + find_game_by_id tests
    registry_mtime_tests.rs steam mtime invalidation tests
  steam/            Steam library scan
    paths.rs          libraryfolders, common app paths
    manifest.rs       appmanifest_*.acf parse
    signal.rs         mtime collection for cache
  epic/             Epic Games Store scan
    paths.rs          manifests, launcher data
    manifest.rs       *.item parse, app name validation
    signal.rs         manifest mtime signals
    epic_tests.rs     manifest validation tests
  config_index/     LocalAppData Saved/Config index
    scan.rs           scan_local_appdata_configs
    matcher.rs        match_config_from_index
    types.rs          ConfigIndexEntry
    config_index_tests.rs normalize + match tests
  ue_detect/        is this install a UE game?
    markers.rs        content.paks, Engine folder heuristics
    executables.rs    *-Win64-Shipping.exe detection
    non_game.rs       Fab plugin, engine-only installs
    ue_detect_tests.rs UE marker + non-game install tests
  ue_version/       engine_family + semver from build files
    parse.rs          Build.version, ProjectVersion
    heuristics.rs     IOStore, WindowsNoEditor fallbacks
    types.rs          UeSemver
    ue_version_tests.rs build.version + heuristic tests
ini/           parse / write / patch ini
  parser.rs         parse_ini, coalesce sections
  parser_tests.rs   float equality + section merge tests
  encoding.rs       UTF-8/UTF-16 LE read/write
  encoding_tests.rs UTF-16 roundtrip test
  paths.rs          validate_config_dir, pack ini paths
  paths_tests.rs    Saved segment + validation tests
  platform.rs       pick_platform_config_dir, apply targets
  platform_tests.rs UE4/UE5 platform dir selection tests
  patch/            line-by-line patch preserving preamble
    sections.rs     scan_sections, line_key
    text.rs         patch_ini_text
    mirror.rs       expand_mirror_key_updates
    patch_tests.rs  preamble + mirror integration tests
  writer/           merge + serialize ini files
    merge.rs          merge_ini, remove_ini_keys
    serialize.rs      write_ini_file_with_encoding_hint
    writer_tests.rs   merge + UTF-16 inheritance tests
fs_util/       file I/O, path safety, process checks
  io/               read/write bytes, BOM, Windows shared access
    read.rs           read_file_bytes, read_utf8_text_file
    write.rs          write_file_bytes, write_file_bytes_opts
    atomic.rs         temp-file atomic replace
    permissions.rs    clear_readonly, format_io_error
  path_safety.rs    ini filename/key validation, safe_child_path
  process/          is_exe_running, kill_exe (Windows)
    cache.rs          running-process TTL cache
    snapshot.rs       Toolhelp32 process enumeration
    kill.rs           TerminateProcess + permission errors
  config.rs         ensure_config_writable probe
  fs_util_tests.rs  I/O + path safety unit tests
covers/        Steam CDN URLs, custom cover files
  steam.rs          steam_header_url (Steam CDN)
  custom.rs         import/remove custom cover, covers_dir cache
  enrich.rs         enrich_cover, merge_saved_cover
  covers_tests.rs   steam_header_url format test
display/       primary monitor resolution (Windows)
  display_tests.rs  resolution string parsing tests
presets/       apply custom changes to config dirs
  apply_dir.rs      merge + patch single config directory
  apply_targets.rs  multi-target apply with rollback
  apply_tests.rs    integration tests for apply pipeline
  diff.rs           ConfigDiffEntry computation
  resolve.rs        section resolution, screen size
  validate.rs       ini payload safety checks
catalog/       загрузка каталога и сборка параметров редактора
  loader.rs           get_game_parameters (оркестрация)
  loader_tests/       интеграционные тесты загрузчика
    catalog_load.rs     split catalog, lookup, cache reuse
    hidden_keys.rs      hidden CVars, DLSS sync keys
    ini_integration.rs  ini → GameParameter mapping
    injection.rs        catalog injection без ini
    version_filter.rs   UE version / reference filtering
  catalog_index/    кэш JSON, build_catalog_index, lookup_entry
    cache.rs          get_or_build_catalog_index, test invalidation
    load.rs           load_parameter_catalog_for_family, parse JSON files
    build.rs          build_catalog_index, catalog_id
    lookup.rs         lookup_entry, should_include_catalog_entry
  dedupe.rs           dedupe_parameters_by_file_key
  unknown.rs          unknown_parameter / unknown_ue_parameter
  types.rs            ParameterCatalogEntry, ReferenceEntry, CatalogIndex
  version.rs          UeSemver, reference_applies_to_version
  localize.rs         pick_localized, pick_title, description quality
  parameter_build/    entry/hint/reference → Parameter
    defaults.rs       catalog_default_value, derive_catalog_recommended
    catalog.rs        entry_to_parameter
    hint.rs           hint_to_parameter
    reference.rs      reference_to_parameter
    tiers.rs          attach_scalability_tier_hints
  injection.rs        inject_catalog_and_reference_parameters
  humanize/           CVar titles, ranges, categories, hidden keys
    rendering_markers.rs  shared game_rendering_key_markers.json loader
    cvar_title.rs     humanize_cvar_key
    ranges.rs         apply_known_range_patterns, infer_value_type
    categories.rs     infer_category, is_game_rendering_key
    hidden_keys.rs    is_hidden_ue_manual_key, UE5-only keys
    value_text.rs     is_opaque_struct_value, truncate_preview
  scalability_tiers/  sg.* tier hints from reference index
    load.rs             tiers JSON cache from disk
    hint.rs             tier_hint_for_key, build_tier_hint_pair
    types.rs            ScalabilityTierRow, UeSemver parse
    scalability_tiers_tests.rs sg.* tier hint coverage tests
scalability/   sg.* quality limits from ini + GUS
  constants.rs      QUALITY_INDEX_GROUPS, is_scalability_quality_index
  parse.rs          DefaultScalability.ini, GUS observed max
  detect.rs         detect_scalability_limits
  scalability_tests.rs preset limits + GUS parsing tests
launch/        Steam/Epic URI launch
  steam.rs          steam://rungameid, manifest lookup
  epic.rs           validate_epic_app_name, com.epicgames.launcher
  launch_tests.rs   steam id + epic app name validation
gpu/           DLSS/RT capability from adapter name
  nvidia.rs         RTX series detection
  priority.rs       discrete vs integrated GPU pick
  enumerate.rs      Windows registry adapter list
  gpu_tests.rs      DLSS/FG capability + GPU priority tests
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
cd src-tauri; cargo fmt --check; cargo clippy -- -D warnings; cargo test
npm test
npm run build
npm run catalog:test
npm run types:gen   # если менялись Rust DTO / AppInvokeError
npm run landing:build
```

CI (`.github/workflows/ci.yml`) дублирует `cargo fmt --check`, `cargo clippy`, `catalog:test`.

## Соглашения

1. **Не смешивать домены** — editor-логика не в `game/`, IPC не в `editor/`.
2. **Barrel `index.ts`** — публичный API модуля; внутри папки можно использовать относительные импорты.
3. **Co-located tests** — тест рядом с модулем, не в отдельном дереве зеркал.
4. **Без изменения данных каталога** в архитектурных PR — `src-tauri/catalog/*.json` только осознанно.
5. **`npm run types:gen`** пишет в `src/lib/api/bindings.ts` (включая `AppInvokeError`, `AppErrorCode`).
6. **IPC-ошибки** — backend отдаёт `{ code, message }`; frontend парсит через [`parseInvokeError()`](../src/lib/core/errors.ts), legacy marker `GSM_ERR_GAME_RUNNING:` поддерживается на переходный период.
7. **`__APP_VERSION__`** — глобал из `package.json` через `vite.config.ts`; тип в [`src/vite-env.d.ts`](../src/vite-env.d.ts).
