# Parameter Sources / Источники параметров

## Русский

Game Settings Master собирает знания о параметрах Unreal Engine из нескольких мест, потому что в самом UE нет одного общего ini-файла со всеми настройками.

| Слой | Источник | Runtime ini | Панель |
|------|----------|-------------|--------|
| `sg.*` меню качества | `Scalability.cpp` + `scalability.json` | `GameUserSettings.ini` → `[ScalabilityGroups]` | Scalability |
| Display / user settings | `GameUserSettings.h` (`UPROPERTY(config)`) + `display.json` | `GameUserSettings.ini` → `[/Script/Engine.GameUserSettings]` | Scalability / Display |
| Engine CVars | `BaseEngine.ini`, `BaseScalability.ini` tier sections | `Engine.ini` / `Scalability.ini` → `[SystemSettings]` | Engine |
| Game-specific keys | Existing player ini files | Same file/section where found | Editable synthetic rows |

Приоритет данных: curated JSON → metadata из Epic source → reference index → auto-guess.

Ключи, найденные в ini игрока, всегда показываются как `present_in_ini`. Если для них нет каталога, приложение создаёт synthetic row: `known=false`, `editable=true`, категория угадывается по имени ключа и секции. Сложные структурные значения вроде key bindings остаются read-only, чтобы не сломать вложенный формат игры.

Почему `sg.*` не парсятся из `BaseScalability.ini`: официальные группы качества регистрируются в C++ (`Engine/Source/Runtime/Engine/Private/Scalability.cpp`) через `TEXT("sg.ShadowQuality")` и похожие CVars. `BaseScalability.ini` в основном хранит tier mapping: какие `r.*` применяются на Low/Medium/High/Epic.

Почему display-поля не парсятся из base ini: `ResolutionSizeX`, `FullscreenMode`, `bUseVSync`, HDR/audio/benchmark поля объявлены как `UPROPERTY(config)` в `UGameUserSettings`. Поэтому registry строится из `GameUserSettings.h`, а apply пишет их в `GameUserSettings.ini`.

Game-specific параметры (`DLSS*`, `FSR*`, `TSR*`, `XeSS*`, `RayTracing*`, `Lumen*`, `Upscaling*`) часто живут в `/Script/GameTitle.*` секциях и не поставляются Epic. Такие ключи распознаются эвристикой как Rendering и остаются редактируемыми там, где были найдены.

### Basic vs Advanced и конфликты

**Базовое** редактирует только слой `GameUserSettings.ini`: `sg.*`, display/window/audio и game-specific ключи, которые уже лежат в GUS. Такой apply не должен создавать `Engine.ini`.

**Расширенное** редактирует низкоуровневые `Engine.ini`, `Game.ini`, `Scalability.ini` и `r.*` overrides. Эти ключи могут конфликтовать с `sg.*`: например, `sg.ShadowQuality=3` задаёт tier теней, а ручной `r.Shadow*` override может частично переопределить этот tier. UI показывает warning chip для таких пересечений.

## English

Game Settings Master combines Unreal Engine parameter knowledge from multiple sources because Unreal does not expose one ini file containing every setting.

| Layer | Source | Runtime ini | Panel |
|------|--------|-------------|-------|
| `sg.*` quality menu | `Scalability.cpp` + `scalability.json` | `GameUserSettings.ini` → `[ScalabilityGroups]` | Scalability |
| Display / user settings | `GameUserSettings.h` (`UPROPERTY(config)`) + `display.json` | `GameUserSettings.ini` → `[/Script/Engine.GameUserSettings]` | Scalability / Display |
| Engine CVars | `BaseEngine.ini`, `BaseScalability.ini` tier sections | `Engine.ini` / `Scalability.ini` → `[SystemSettings]` | Engine |
| Game-specific keys | Existing player ini files | Same file/section where found | Editable synthetic rows |

Data priority: curated JSON → Epic source metadata → reference index → auto-guess.

Keys found in the player's ini are always shown as `present_in_ini`. If the catalog does not know them, the app creates a synthetic row: `known=false`, `editable=true`, with a category guessed from the key and section. Complex structured values such as key bindings remain read-only to avoid corrupting nested game data.

Why `sg.*` is not fetched from `BaseScalability.ini`: official quality groups are registered in C++ (`Engine/Source/Runtime/Engine/Private/Scalability.cpp`) via `TEXT("sg.ShadowQuality")` and similar CVars. `BaseScalability.ini` mostly stores tier mappings: which `r.*` values are applied for Low/Medium/High/Epic.

Why display fields are not fetched from base ini: `ResolutionSizeX`, `FullscreenMode`, `bUseVSync`, HDR/audio/benchmark fields are declared as `UPROPERTY(config)` in `UGameUserSettings`. The registry is therefore extracted from `GameUserSettings.h`, and apply writes them to `GameUserSettings.ini`.

Game-specific parameters (`DLSS*`, `FSR*`, `TSR*`, `XeSS*`, `RayTracing*`, `Lumen*`, `Upscaling*`) often live in `/Script/GameTitle.*` sections and are not shipped by Epic. The app detects these keys heuristically as Rendering and keeps them editable in the file/section where they were found.

### Availability (runtime)

For a detected `engine_version` (UE 4.27–5.8), `get_game_parameters` returns:

1. Every key found in the player's ini (`present_in_ini=true`)
2. All bundled curated entries (human RU/EN)
3. Every reference index entry where `reference_applies_to_version` is true (full version slice — not all 726 keys for every game)

Typical counts per UE version (see `merge_stats.json` → `applicable_by_version`): **~595 (4.27)** … **~661 (5.4)** … **~706 (5.8)**.

Hidden keys (`HIDDEN_UE_MANUAL_KEYS`) stay excluded. Curated metadata wins on key collision.

### Description tiers

| Tier | Source | Quality badge |
|------|--------|---------------|
| Human | `engine.json`, `scalability.json`, `display.json`, tier A | `human` |
| Semi | tier B + frequency-expanded overlays | `semi` |
| Auto | tier C template (category, range, default) | `auto` |

Merge order: **curated > tier_a > tier_b > tier_c > auto**. Goal: zero bare «see Unreal documentation» stubs.

### Basic vs Advanced and Conflicts

**Basic** edits only the `GameUserSettings.ini` layer: `sg.*`, display/window/audio, and game-specific keys already stored in GUS. A Basic apply must not create `Engine.ini`.

**Advanced** edits low-level `Engine.ini`, `Game.ini`, `Scalability.ini`, and `r.*` overrides. These keys may conflict with `sg.*`: for example, `sg.ShadowQuality=3` sets the shadow tier, while a manual `r.Shadow*` override can partially override that tier. The UI shows a warning chip for those overlaps.
