# Epic Unreal Engine clone — setup / Настройка клона UE

[English](#english) · [Русский](#русский)

---

## English

### Why

Game Settings Master builds `ue_reference_index.json` from Epic’s **BaseEngine.ini** and **BaseScalability.ini** across UE **4.27–5.8**. The repo ships a **bundled catalog** (**767** reference keys, tier A/B overlays). A local Epic git clone unlocks rebuilding that catalog from live engine sources.

### Prerequisites

1. [Epic Games account](https://www.epicgames.com/)
2. Link GitHub to Epic: [Unreal Engine on GitHub](https://www.unrealengine.com/en-US/ue-on-github) → accept license → connect GitHub
3. Git, Git LFS, ~100+ GB free disk space

### Clone

```powershell
# Option A — GitHub CLI (after Epic linked your account)
gh repo clone EpicGames/UnrealEngine D:\UnrealEngine

# Option B — git (use Epic credentials / PAT when prompted)
git clone https://github.com/EpicGames/UnrealEngine.git D:\UnrealEngine
cd D:\UnrealEngine
git lfs install
git lfs pull
```

Other common paths: `C:\UnrealEngine`, `%USERPROFILE%\UnrealEngine`, or set env `UE_ENGINE_ROOT`.

### Fetch reference snapshots

From the **Game Settings Master** repo root:

```powershell
.\scripts\fetch-ue-reference.ps1 -AutoTags
# or explicit path:
.\scripts\fetch-ue-reference.ps1 -EngineRoot "D:\UnrealEngine" -AutoTags

python tools/ue-catalog-builder/extract/sg_from_cpp.py --all-versions
python tools/ue-catalog-builder/extract/gus_from_header.py --all-versions
npm run catalog:build
npm run catalog:test
.\scripts\validate-catalog-stats.ps1
```

`-AutoTags` resolves release tags per version (4.27, 5.0 … 5.8) and extracts **BaseEngine.ini**, **BaseScalability.ini**, and registry source files via `git archive` (fast; no full worktree checkout).

**Expected catalog size:** ~725 merged unique CVars across 10 UE versions (BaseEngine.ini + BaseScalability.ini). Not 1000+ — most engine defaults live in tier sections and dedupe on merge.

### Troubleshooting

| Problem | Fix |
|---------|-----|
| `AutoTags requires a git clone` | Clone must contain `.git`; plain ZIP installs won’t work |
| `No git tag for UE X.Y` | Update clone: `git fetch --tags` |
| `403` on clone | Epic account not linked to GitHub or missing UE license |
| Config folder not found | Use repo root (folder with `Engine/Config`), not `Engine/` subfolder only |

---

## Русский

### Зачем

Каталог `ue_reference_index.json` собирается из **BaseEngine.ini** / **BaseScalability.ini** Epic для UE **4.27–5.8**. В релизе — **bundled catalog** (**767** reference keys, tier A/B overlays). Локальный git-клон Epic нужен для пересборки каталога из исходников движка.

### Что нужно

1. Аккаунт Epic Games  
2. Привязка GitHub к Epic: [UE on GitHub](https://www.unrealengine.com/en-US/ue-on-github) → лицензия → GitHub  
3. Git, Git LFS, ~100+ ГБ на диске  

### Клонирование

```powershell
gh repo clone EpicGames/UnrealEngine D:\UnrealEngine
# или
git clone https://github.com/EpicGames/UnrealEngine.git D:\UnrealEngine
cd D:\UnrealEngine
git lfs install
git lfs pull
```

Пути: `D:\UnrealEngine`, `C:\UnrealEngine`, `%USERPROFILE%\UnrealEngine`, переменная `UE_ENGINE_ROOT`.

### Fetch снимков

Из корня **Game Settings Master**:

```powershell
.\scripts\fetch-ue-reference.ps1 -AutoTags
python tools/ue-catalog-builder/extract/sg_from_cpp.py --all-versions
python tools/ue-catalog-builder/extract/gus_from_header.py --all-versions
npm run catalog:build
npm run catalog:test
.\scripts\validate-catalog-stats.ps1
```

`-AutoTags` — release-теги и `git archive` (быстро, без полного worktree).

### Частые ошибки

| Ошибка | Решение |
|--------|---------|
| Нет git-клона | Нужен `.git`, не ZIP-дистрибутив |
| Нет тега для версии | `git fetch --tags` в клоне |
| 403 при clone | Не привязан GitHub / нет лицензии UE |
| Config not found | Указывайте корень репозитория UE, не только `Engine/` |
