# Game Settings Master

[Русский](README.md) · [English](README.en.md) · [Support development](https://www.donationalerts.com/r/mike_saito)

**Game settings in focus**

Read and tune Unreal Engine game configs — with parameter descriptions, GPU-aware options, and backups.

`UE 4` · `UE 5`

## Features

**01 — Game library**  
Steam and Epic scan, manual add for UE games. The app finds your config folder automatically.

**02 — Parameter editor**  
Advanced Editor with two zones: **Basic** (GameUserSettings.ini: official sg.*, display/audio/window — like the in-game menu) and **Advanced** (Engine.ini/Game.ini/Scalability.ini r.* — engine modding with a warning). Recommended filter, quality labels, and sg.* tier tooltips.

**03 — GPU-aware filters**  
DLSS, FSR, ray tracing and Frame Generation — safe clamp for your GPU. No pointless options on weak hardware.

**04 — Backups**  
Snapshot before every apply. Roll back to the previous state in one click — no fear of breaking your config.

**05 — Parameter metadata catalog**  
Knows **113** human-curated keys (RU+EN), **~233** tier A/B overlays on the reference index, and **~563** stub reference rows until expanded. The editor injects curated GUS/Engine and recommended reference keys even when they are missing from the player's ini.

## Download

Windows · free · unsigned build

* [Download installer](https://github.com/MikeSaito/Game-Settings-Master/releases/latest/download/Game-Settings-Master_1.0.2-a_x64-setup.exe)
* [GitHub](https://github.com/MikeSaito/Game-Settings-Master)
* [Website](https://mikesaito.github.io/Game-Settings-Master/)

### First launch on Windows

The app is not commercially signed yet — SmartScreen may show a blue warning. That's normal for indie software.

1. Click **More info**
2. Then **Run anyway**

After the first run, Windows usually stops asking.

---

## Developer setup

### Requirements

| Tool | Purpose |
|------|---------|
| **Node.js** 20+ | Frontend, Tauri CLI, tests |
| **Rust** (stable) + **MSVC Build Tools** | Tauri backend (Windows) |
| **Python** 3.10+ | UE catalog build (`tools/ue-catalog-builder/`) |

### Quick start

```powershell
npm ci
powershell -File scripts/install-githooks.ps1

npm run tauri dev    # desktop app (Vite + Tauri)
npm test             # Vitest
npm run build        # production frontend
```

Landing separately: `npm run landing:dev` / `npm run landing:build`.

After changing IPC DTOs in Rust:

```powershell
npm run types:gen
```

### Repository layout

```
src/                    React SPA (@/ alias → src/)
  lib/                  api, core, routing, editor, game, gpu, settings
  components/           feature UI (advanced, library, layout, app, …)
  hooks/                app, game, editor
  pages/                router screens
src-tauri/src/          Rust: commands, ini, discovery, catalog
  core/                 models, errors, paths
landing/src/            marketing site (GitHub Pages)
tools/ue-catalog-builder/   Python ue_reference_index.json pipeline
tools/ue-reference/     local Epic ini snapshots (not fully in git)
docs/                   ARCHITECTURE.md, epic-clone-setup, parameter-sources
```

Full module map, import conventions, and where to add new code — [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).

### Parameter catalog (UE)

The app ships two layers:

| Layer | Files | Purpose |
|-------|-------|---------|
| **Curated (human)** | `engine.json`, `scalability.json`, `ue4.json`, `display.json`, … | **113** keys with full RU+EN titles/descriptions |
| **Tier overlays** | `tier_a_descriptions.json`, `tier_b_descriptions.json` → merged into reference | **~233** keys with human text on top of Epic defaults |
| **Reference index** | `ue_reference_index.json` | **726** merged engine keys (UE 4.27–5.8); **~563** remain stub until expanded |
| **Source registries** | `sg_registry_merged.json`, `gus_registry_merged.json` | Auto-generated from Epic `Scalability.cpp` / `GameUserSettings.h` |

**Lookup priority:** curated JSON → ini row → reference index (version-filtered) → key hints → auto-guess. Curated always wins on key collision.

**Editor injection:** bundled curated GUS (`sg.*`, display) + Engine/Scalability entries, then **every reference key applicable to the game's UE version** (see `applicable_by_version` in `merge_stats.json`). Advanced default filter: **Full catalog**.

Rebuild reference index after updating UE snapshots:

```powershell
# First-time / full catalog build — see docs/epic-clone-setup.md
.\scripts\fetch-ue-reference.ps1 -AutoTags

python tools/ue-catalog-builder/extract/sg_from_cpp.py --all-versions
python tools/ue-catalog-builder/extract/gus_from_header.py --all-versions
npm run catalog:build
npm run catalog:test
.\scripts\validate-catalog-stats.ps1
```

Without an Epic clone the app ships fixture snapshots (UE 4.27 + 5.4, **548+ keys**). Full fetch from 10 UE versions yields **726 merged engine keys**, source-extracted `sg.*`, and standard `UGameUserSettings` fields — see [`docs/epic-clone-setup.md`](docs/epic-clone-setup.md) and [`docs/parameter-sources.md`](docs/parameter-sources.md). Counts in `src-tauri/catalog/generated/merge_stats.json`.

Advanced Editor filters reference keys by detected `engine_version` (UE 4.27–5.8). Keys in your ini are always listed.

See [`tools/ue-reference/README.md`](tools/ue-reference/README.md), [`docs/parameter-sources.md`](docs/parameter-sources.md).

### Pre-PR checks

```powershell
npm test
npm run build
cd src-tauri; cargo test
python tools/ue-catalog-builder/test_build.py
npm run landing:build
```

## Documentation

| File | Contents |
|------|----------|
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | Code layout, imports, module boundaries |
| [`docs/epic-clone-setup.md`](docs/epic-clone-setup.md) | Epic UE clone and full catalog rebuild |
| [`docs/parameter-sources.md`](docs/parameter-sources.md) | Where parameter descriptions come from |
| [`tools/ue-catalog-builder/README.md`](tools/ue-catalog-builder/README.md) | Python catalog pipeline |

---

Game Settings Master v1.0.2-a
