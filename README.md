# Game Settings Master

[Русский](README.md) · [English](README.en.md) · [Поддержите разработку](https://dalink.to/mike_saito)

**Настройки игр в фокусе**

Читайте и настраивайте ini игр на Unreal Engine — с описаниями параметров, фильтрами под GPU и бэкапами.

`UE 4` · `UE 5`

## Возможности

**01 — Библиотека игр**  
Сканирование Steam и Epic, ручное добавление UE-игр. Приложение само находит папку конфигурации.

**02 — Редактор параметров**  
Advanced Editor с двумя зонами: **Базовое** (GameUserSettings.ini: official sg.*, display/audio/window — как меню игры) и **Расширенное** (Engine.ini/Game.ini/Scalability.ini r.* — модинг движка с предупреждением). Фильтр «Рекомендуемые», quality labels и tier tooltips для sg.*.

**03 — GPU-aware фильтры**  
DLSS, FSR, ray tracing и Frame Generation — безопасный clamp под ваш GPU. Без бессмысленных опций на слабом железе.

**04 — Бэкапы**  
Snapshot перед каждым apply. Откат к предыдущему состоянию одним кликом — без страха сломать конфиг.

**05 — Каталог описаний параметров**  
113 human-curated ключей (RU+EN), ~233 tier A/B overlays на reference index, остальные ~563 reference — stub до расширения. Редактор подмешивает curated GUS/Engine и recommended reference даже без строк в ini игрока.

## Скачать

Windows · бесплатно · без подписи издателя

* [Скачать установщик](https://github.com/MikeSaito/Game-Settings-Master/releases/latest/download/Game-Settings-Master_1.0.0_x64-setup.exe)
* [GitHub](https://github.com/MikeSaito/Game-Settings-Master)
* [Сайт](https://mikesaito.github.io/Game-Settings-Master/)

### Первый запуск в Windows

Приложение пока без коммерческой подписи — SmartScreen может показать синее предупреждение. Для indie-софта это нормально.

1. Нажмите **Подробнее**
2. Затем **Выполнить в любом случае**

После первого запуска Windows обычно больше не спрашивает.

---

## Developer setup

```powershell
npm ci
powershell -File scripts/install-githooks.ps1
```

### Parameter catalog (UE)

The app ships two layers:

| Layer | Files | Purpose |
|-------|-------|---------|
| **Curated (human)** | `engine.json`, `scalability.json`, `ue4.json`, `display.json`, … | **113** keys with full RU+EN titles/descriptions |
| **Tier overlays** | `tier_a_descriptions.json`, `tier_b_descriptions.json` → merged into reference | **~233** keys with human text on top of Epic defaults |
| **Reference index** | `ue_reference_index.json` | **726** merged engine keys (UE 4.27–5.8); **~563** remain stub until expanded |
| **Source registries** | `sg_registry_merged.json`, `gus_registry_merged.json` | Auto-generated from Epic `Scalability.cpp` / `GameUserSettings.h` |

**Lookup priority:** curated JSON → ini row → reference index (version-filtered) → key hints → auto-guess. Curated always wins on key collision.

**Editor injection:** bundled curated GUS (`sg.*`, display) + Engine/Scalability entries, then **every reference key applicable to the game's UE version** (see `applicable_by_version` in `merge_stats.json`). Advanced default filter: **Полный каталог**.

Rebuild reference index after updating UE snapshots:

```powershell
# First-time / full catalog build — see docs/epic-clone-setup.md
.\scripts\fetch-ue-reference.ps1 -AutoTags
# or: -EngineRoot "D:\UnrealEngine" -AutoTags

python tools/ue-catalog-builder/extract_sg_from_cpp.py --all-versions
python tools/ue-catalog-builder/extract_gus_from_header.py --all-versions
npm run catalog:build
npm run catalog:test
.\scripts\validate-catalog-stats.ps1
```

Without an Epic clone the app ships fixture snapshots (UE 4.27 + 5.4, **548+ keys**). Full fetch from 10 UE versions yields **726 merged engine keys**, source-extracted `sg.*`, and standard `UGameUserSettings` fields — see [`docs/epic-clone-setup.md`](docs/epic-clone-setup.md) and [`docs/parameter-sources.md`](docs/parameter-sources.md). Counts in `src-tauri/catalog/generated/merge_stats.json`.

Advanced Editor filters reference keys by detected `engine_version` (UE 4.27–5.8). Keys in your ini are always listed.

See `tools/ue-reference/README.md`. Curated entries always win on key collision.

After changing IPC DTOs in Rust:

```powershell
npm run types:gen
```

---

Game Settings Master v1.0.0
