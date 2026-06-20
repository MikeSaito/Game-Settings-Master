# Game Settings Master

[Русский](README.md) · [English](README.en.md) · [Support development](https://dalink.to/mike_saito)

**Game settings in focus**

Read and tune Unreal Engine and Unity game configs — with parameter descriptions, GPU-aware options, and backups.

`UE 4` · `UE 5` · `Unity`

## Features

**01 — Game library**  
Steam and Epic scan, manual add. The app finds your config folder automatically.

**02 — Parameter editor**  
Main game tab: interactive sliders, toggles and dropdowns for key UE4/UE5 and Unity parameters — with descriptions, categories and dependencies.

**03 — GPU-aware filters**  
DLSS, FSR, ray tracing and Frame Generation — safe clamp for your GPU. No pointless options on weak hardware.

**04 — Backups**  
Snapshot before every apply. Roll back to the previous state in one click — no fear of breaking your config.

**05 — Parameter metadata catalog**  
Bundled descriptions for the editor (`src-tauri/catalog/`) — not ready-made presets, but a reference of keys, sections and hints for the Advanced Editor.

## Download

Windows · free · unsigned build

* [Download installer](https://github.com/MikeSaito/Game-Settings-Master/releases/latest/download/Game-Settings-Master_1.0.0_x64-setup.exe)
* [GitHub](https://github.com/MikeSaito/Game-Settings-Master)
* [Website](https://mikesaito.github.io/Game-Settings-Master/)

### First launch on Windows

The app is not commercially signed yet — SmartScreen may show a blue warning. That's normal for indie software.

1. Click **More info**
2. Then **Run anyway**

After the first run, Windows usually stops asking.

---

## Developer setup

```powershell
npm ci
powershell -File scripts/install-githooks.ps1
```

`install-githooks.ps1` (optional) enables pre-commit: `npm test` + `bindings.ts` sync check.

### Parameter catalog (UE / Unity)

Source of truth — `src-tauri/catalog/` (`ue4.json`, `engine.json`, `display.json`, `scalability.json`, `unity.json`, `key_hints.json`). Edit JSON directly; there is no separate VPS server.

After changing IPC DTOs in Rust (`src-tauri/src/models.rs` and related types):

```powershell
npm run types:gen
```

Commit `src/lib/bindings.ts` (CI: `scripts/verify-types-sync.ps1`).

---

Game Settings Master v1.0.0
