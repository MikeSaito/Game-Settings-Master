# Game Settings Master

[Русский](README.md) · [English](README.en.md)

**Game settings in focus**

A graphics master for Unreal Engine, Unity and author-curated breakdowns of other games — no manual digging through config files.

`UE 4` · `UE 5` · `Unity` · `ReShade` · `Author breakdowns`

## Features

**01 — Game library**  
Scans Steam and Epic, plus manual adding. The app finds the config folder for you.

**02 — Author presets**  
Author-verified presets for select games (e.g. Forza Horizon 6) — apply in one click with a diff preview. Every change visible in configs.

**03 — Smart tuning**  
DLSS, FSR, ray tracing and Frame Generation — safely clamped to your GPU. No pointless options on weak hardware.

**04 — Manual editor**  
Interactive sliders, toggles and dropdowns for key UE4/UE5 parameters — with descriptions, categories and dependencies.

**05 — Backups**  
A snapshot before every apply. Roll back to the previous state in one click — no fear of breaking your config.

**06 — Cloud presets**  
Content syncs from the server without an app release. Offline — a built-in cache fallback.

**07 — ReShade**  
Installs post-processing into the game folder: Performance, Clarity and Cinematic presets, plus author ini packs for specific games (e.g. Subnautica 2). Choose the graphics API, launch with or without ReShade — the proxy is removed when effects are not needed.

## Download

Windows · free · no publisher signature

* [Download the installer](https://github.com/MikeSaito/Game-Settings-Master/releases/latest/download/Game-Settings-Master_0.3.1_x64-setup.exe)
* [GitHub](https://github.com/MikeSaito/Game-Settings-Master)
* [Website](https://mikesaito.github.io/Game-Settings-Master/)

### First launch on Windows

The app is not yet commercially signed — SmartScreen may show a blue warning. For indie software this is normal.

1. Click **More info**
2. Then **Run anyway**

After the first launch Windows usually stops asking.

\---

## ReShade (local development)

The git repo does **not** contain the real ReShade DLLs and shaders (see `.gitignore`). Before `tauri build` you need `npm run reshade:setup` — otherwise the build fails at the bundle check. GSM will **not** install a stub proxy into the game folder: a DLL < 64 KB is blocked before it is written.

### Quick setup

```powershell
npm run reshade:setup
```

The script clones the shaders (`crosire/reshade-shaders`), downloads the addon DLL from [reshade.me](https://reshade.me) and verifies the bundle before building. Requires **7-Zip** (installed automatically in CI).

### Manual

1. **Shaders** (preset effects, optional for dev):

```powershell
   .\\\\\\\\scripts\\\\\\\\fetch-reshade-shaders.ps1
   ```

Target: `src-tauri/presets/reshade/shaders/Shaders/\\\\\\\*.fx`

2. **Addon DLL** (required to install ReShade into a game):

   * Download it from [reshade.me](https://reshade.me)
   * Place it in `src-tauri/presets/reshade/bin/`:

     * `dxgi.dll` — DX12 (UE5, Forza Horizon 6, etc.)
     * `d3d11.dll` — DX11
     * other APIs — see `src-tauri/presets/reshade/ATTRIBUTION.txt`
3. Restart `npm run tauri dev`. On the ReShade tab the **"DLL bundle OK"** badge means you can install.

   ### If a game does not start after an old dev test

   On the ReShade tab → **"Remove"** (strips the proxy from the game folder). Or **"Play without ReShade"** in the header.

   \---

   ## ReShade — license and authors

   Game Settings Master can install **ReShade** (a third-party post-processing injector) into a game folder at the user's request.

|Component|Author|License|
|-|-|-|
|ReShade addon (DLL)|[Patrick Mours (crosire)](https://reshade.me)|BSD 3-Clause|
|`.fx` shaders|[crosire/reshade-shaders](https://github.com/crosire/reshade-shaders) and the file authors|see the `.fx` headers|

**Full ReShade license text** (required when distributing binaries):

* In the repository: [`src-tauri/presets/reshade/LICENSE-ReShade.txt`](src-tauri/presets/reshade/LICENSE-ReShade.txt)
* In the installed app: `presets/reshade/LICENSE-ReShade.txt` next to the GSM resources
* Summary and shaders: [`ATTRIBUTION.txt`](src-tauri/presets/reshade/ATTRIBUTION.txt), [`shaders/THIRD-PARTY-NOTICES.txt`](src-tauri/presets/reshade/shaders/THIRD-PARTY-NOTICES.txt)

GSM is **not affiliated** with ReShade and is **not endorsed** by the ReShade authors. Using ReShade in online games is at your own risk.

The GSM presets (Performance / Clarity / Cinematic) use the **Clarity** (Ioxa), **Vignette** (CeeJay.dk) and **AdaptiveSharpen** (bacondither) effects — see the headers in `presets/reshade/shaders/Shaders/`.

\---

Game Settings Master v0.3.1
