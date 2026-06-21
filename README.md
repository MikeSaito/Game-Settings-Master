# Game Settings Master

[Русский](README.md) · [English](README.en.md) · [Поддержите разработку](https://www.donationalerts.com/r/mike_saito)

**Настройки игр в фокусе**

Читайте и настраивайте ini игр на Unreal Engine — с описаниями параметров, фильтрами под видеокарту и резервными копиями.

`UE 4` · `UE 5`

## Возможности

**01 — Библиотека игр**  
Сканирование Steam и Epic, ручное добавление UE-игр. Приложение само находит папку конфигурации.

**02 — Редактор параметров**  
Редактор с двумя зонами: **Базовое** (GameUserSettings.ini: официальные sg.*, экран/аудио/окно — как меню игры) и **Расширенное** (Engine.ini/Game.ini/Scalability.ini r.* — настройка движка с предупреждением). Фильтр «Рекомендуемые», подписи качества и подсказки по уровням для sg.*.

**03 — Фильтры под видеокарту**  
DLSS, FSR, трассировка лучей и генерация кадров — безопасное ограничение под ваш GPU. Без бессмысленных опций на слабом железе.

**04 — Резервные копии**  
Снимок конфигов перед каждым применением. Откат к предыдущему состоянию одним кликом — без страха сломать настройки.

**05 — Каталог описаний параметров**  
113 ключей с ручными описаниями (RU+EN), ~233 наложения уровней A/B на справочный индекс, остальные ~563 строки справочника — заглушки до расширения. Редактор подмешивает подготовленные записи GUS/Engine и рекомендуемый справочник даже без строк в ini игрока.

## Скачать

Windows · бесплатно · без подписи издателя

* [Скачать установщик](https://github.com/MikeSaito/Game-Settings-Master/releases/latest/download/Game-Settings-Master_1.0.2-a_x64-setup.exe)
* [GitHub](https://github.com/MikeSaito/Game-Settings-Master)
* [Сайт](https://mikesaito.github.io/Game-Settings-Master/)

### Первый запуск в Windows

Приложение пока без коммерческой подписи — SmartScreen может показать синее предупреждение. Для indie-софта это нормально.

1. Нажмите **Подробнее**
2. Затем **Выполнить в любом случае**

После первого запуска Windows обычно больше не спрашивает.

---

## Разработка

```powershell
npm ci
powershell -File scripts/install-githooks.ps1
```

### Каталог параметров (UE)

В приложении два слоя:

| Слой | Файлы | Назначение |
|------|-------|------------|
| **Ручные записи** | `engine.json`, `scalability.json`, `ue4.json`, `display.json`, … | **113** ключей с полными описаниями RU+EN |
| **Наложения уровней** | `tier_a_descriptions.json`, `tier_b_descriptions.json` → в справочник | **~233** ключей с человекочитаемым текстом поверх дефолтов Epic |
| **Справочный индекс** | `ue_reference_index.json` | **726** объединённых ключей движка (UE 4.27–5.8); **~563** остаются заглушками до расширения |
| **Исходные реестры** | `sg_registry_merged.json`, `gus_registry_merged.json` | Автогенерация из Epic `Scalability.cpp` / `GameUserSettings.h` |

**Приоритет поиска:** ручные JSON → строка ini → справочный индекс (с фильтром по версии) → подсказки по ключам → авто-угадывание. Ручные записи всегда побеждают при коллизии ключей.

**Подмешивание в редакторе:** подготовленные GUS (`sg.*`, display) + Engine/Scalability, затем **каждый справочный ключ, применимый к версии UE игры** (см. `applicable_by_version` в `merge_stats.json`). Фильтр расширенного режима по умолчанию: **Полный каталог**.

Пересборка справочного индекса после обновления снимков UE:

```powershell
# Первый раз / полная сборка каталога — см. docs/epic-clone-setup.md
.\scripts\fetch-ue-reference.ps1 -AutoTags
# или: -EngineRoot "D:\UnrealEngine" -AutoTags

python tools/ue-catalog-builder/extract_sg_from_cpp.py --all-versions
python tools/ue-catalog-builder/extract_gus_from_header.py --all-versions
npm run catalog:build
npm run catalog:test
.\scripts\validate-catalog-stats.ps1
```

Без клона Epic приложение поставляется с фикстурными снимками (UE 4.27 + 5.4, **548+ ключей**). Полная выгрузка из 10 версий UE даёт **726 объединённых ключей движка**, извлечённые из исходников `sg.*` и стандартные поля `UGameUserSettings` — см. [`docs/epic-clone-setup.md`](docs/epic-clone-setup.md) и [`docs/parameter-sources.md`](docs/parameter-sources.md). Счётчики в `src-tauri/catalog/generated/merge_stats.json`.

Расширенный редактор фильтрует справочные ключи по обнаруженной `engine_version` (UE 4.27–5.8). Ключи из ваших ini всегда в списке.

Подробнее: `tools/ue-reference/README.md`. Ручные записи всегда побеждают при коллизии ключей.

После изменения IPC DTO в Rust:

```powershell
npm run types:gen
```

---

Game Settings Master v1.0.2-a
