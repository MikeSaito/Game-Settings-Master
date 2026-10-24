<p align="center">
  <img src="landing/public/logo.svg" width="88" alt="Game Settings Master">
</p>

<h1 align="center">Game Settings Master</h1>

<p align="center">
  <a href="README.md">English</a> ·
  <a href="README.ru.md">Русский</a> ·
  <a href="https://gsm-tool.com/en/">Website</a> ·
  <a href="https://www.donationalerts.com/r/mike_saito">Donate</a>
</p>

<p align="center">
  <a href="https://github.com/MikeSaito/Game-Settings-Master/releases"><img src="https://img.shields.io/github/v/release/MikeSaito/Game-Settings-Master?style=flat-square&label=release" alt="Release"></a>
  <img src="https://img.shields.io/badge/Windows-0078D4?style=flat-square&logo=windows&logoColor=white" alt="Windows">
  <img src="https://img.shields.io/badge/license-MIT-blue?style=flat-square" alt="MIT">
  <a href="https://gsm-tool.com/"><img src="https://img.shields.io/badge/site-gsm--tool.com-b87a14?style=flat-square" alt="gsm-tool.com"></a>
</p>

<p align="center">
  <strong>Ini editor for Unreal Engine games</strong><br>
  Edit GameUserSettings and Engine.ini in one place – with hints, backups, and filters matched to your GPU.
</p>

<p align="center">
  <code>UE 4.27–5.8</code> · <code>Steam</code> · <code>Epic</code> · <code>DLSS</code> · <code>FSR</code>
</p>

---

## Screenshots

Current application interface with demonstration data. Click an image to view it at full size.

<table>
  <tr>
    <td width="50%" align="center">
      <strong>Game library</strong><br>
      <a href="docs/screenshots/en/library.webp"><img src="docs/screenshots/en/library.webp" width="400" alt="Game library"></a>
    </td>
    <td width="50%" align="center">
      <strong>Basic editor</strong><br>
      <a href="docs/screenshots/en/editor-basic.webp"><img src="docs/screenshots/en/editor-basic.webp" width="400" alt="Basic editor"></a>
    </td>
  </tr>
  <tr>
    <td width="50%" align="center">
      <strong>Advanced editor</strong><br>
      <a href="docs/screenshots/en/editor-advanced.webp"><img src="docs/screenshots/en/editor-advanced.webp" width="400" alt="Advanced editor"></a>
    </td>
    <td width="50%" align="center">
      <strong>INI text editor</strong><br>
      <a href="docs/screenshots/en/editor-files.webp"><img src="docs/screenshots/en/editor-files.webp" width="400" alt="INI text editor"></a>
    </td>
  </tr>
  <tr>
    <td width="50%" align="center">
      <strong>Change preview</strong><br>
      <a href="docs/screenshots/en/preview.webp"><img src="docs/screenshots/en/preview.webp" width="400" alt="Change preview"></a>
    </td>
    <td width="50%" align="center">
      <strong>Backups</strong><br>
      <a href="docs/screenshots/en/backups.webp"><img src="docs/screenshots/en/backups.webp" width="400" alt="Backups"></a>
    </td>
  </tr>
</table>

---

## What it does

**Game library** – scans Steam and Epic libraries, supports manual games, finds config folders, and shows engine info and cover art.

**Basic editor** – GameUserSettings.ini: scalability presets, resolution, window mode, VSync – the same knobs you see in-game.

**Advanced editor** – Engine.ini CVars with descriptions, warnings, tier hints, and a Recommended filter for your setup.

**INI text editor** – a separate Files tab for five game INI files, with search, per-file drafts, full-text preview and backups. Preserves comments, duplicate entries and supported encodings; files up to 1 MiB.

**Backups** – automatic snapshot before every apply; restore the previous state in one click.

**Built-in reference** – parameter hints in English and Russian, tuned to the game’s Unreal Engine version.

**GPU-aware UI** – DLSS, FSR, ray tracing, and frame generation options appear only when your graphics card supports them.

---

## Download

Free · Windows · MIT · open source

| | |
|---|---|
| Latest release | [**Download for Windows**](https://github.com/MikeSaito/Game-Settings-Master/releases/latest) |
| Website | [gsm-tool.com](https://gsm-tool.com/en/) |

### First launch (SmartScreen)

The installer is not commercially signed. Windows may show a blue SmartScreen prompt – this is normal for independent software.

1. Click **More info**
2. Click **Run anyway**

After the first successful launch, Windows usually stops showing the warning.

---

<p align="center">
  <a href="LICENSE">MIT License</a> © 2026 Mike Saito
</p>
