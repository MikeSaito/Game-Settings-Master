# Game Settings Master

**Настройки игр в фокусе**

Мастер графики для Unreal Engine, Unity и авторских разборов других игр — без ручного ковыряния в конфигах.

`UE 4` · `UE 5` · `Unity` · `ReShade` · `Авторские разборы`

## Возможности

**01 — Библиотека игр**  
Сканирование Steam и Epic, ручное добавление. Приложение само находит папку конфигурации.

**02 — Пресеты в один клик**  
От Ultra Low до Ultra High — с предпросмотром diff до применения. Видно каждую правку в конфигах.

**03 — Умная настройка**  
DLSS, FSR, ray tracing и Frame Generation — безопасный clamp под ваш GPU. Без бессмысленных опций на слабом железе.

**04 — Ручной редактор**  
Более сотни параметров с описаниями, категориями и зависимостями.

**05 — Бэкапы**  
Snapshot перед каждым apply. Откат к предыдущему состоянию одним кликом — без страха сломать конфиг.

**06 — Облачные пресеты**  
Контент с сервера синхронизируется без релиза приложения. Offline — встроенный fallback из кэша.

**07 — ReShade**  
Установка post-processing в папку игры: пресеты Performance, Clarity и Cinematic, авторские ini для отдельных игр (например Subnautica 2). Выбор графического API, запуск с ReShade или без — proxy снимается, когда эффекты не нужны.

## Скачать

Windows · бесплатно · без подписи издателя

- [Скачать установщик](https://github.com/MikeSaito/Game-Settings-Master/releases/latest/download/Game-Settings-Master_0.2.0_x64-setup.exe)
- [GitHub](https://github.com/MikeSaito/Game-Settings-Master)
- [Сайт](https://mikesaito.github.io/Game-Settings-Master/)

### Первый запуск в Windows

Приложение пока без коммерческой подписи — SmartScreen может показать синее предупреждение. Для indie-софта это нормально.

1. Нажмите **Подробнее**
2. Затем **Выполнить в любом случае**

После первого запуска Windows обычно больше не спрашивает.

---

## ReShade (локальная разработка)

В git **нет** настоящих ReShade DLL и шейдеров (см. `.gitignore`). Перед `tauri build` нужен `npm run reshade:setup` — иначе сборка упадёт на проверке бандла. GSM **не установит** stub proxy в папку игры: DLL &lt; 64 KB блокируется до записи.

### Быстрый setup

```powershell
npm run reshade:setup
```

Скрипт клонирует шейдеры (`crosire/reshade-shaders`), скачивает addon DLL с [reshade.me](https://reshade.me) и проверяет бандл перед сборкой. Нужен **7-Zip** (в CI ставится автоматически).

### Вручную

1. **Шейдеры** (эффекты пресетов, опционально для dev):
   ```powershell
   .\scripts\fetch-reshade-shaders.ps1
   ```
   Цель: `src-tauri/presets/reshade/shaders/Shaders/*.fx`

2. **Addon DLL** (обязательно для установки ReShade в игру):
   - Скачайте с [reshade.me](https://reshade.me)
   - Положите в `src-tauri/presets/reshade/bin/`:
     - `dxgi.dll` — DX12 (UE5, Forza Horizon 6 и др.)
     - `d3d11.dll` — DX11
     - остальные API — см. `src-tauri/presets/reshade/ATTRIBUTION.txt`

3. Перезапустите `npm run tauri dev`. На вкладке ReShade badge **«Бандл DLL OK»** — можно ставить.

### Если игра не стартует после старого dev-теста

На вкладке ReShade → **«Удалить»** (снимает proxy из папки игры). Либо **«Играть без ReShade»** в шапке.

---

## ReShade — лицензия и авторы

Game Settings Master может устанавливать **ReShade** (сторонний post-processing injector) в папку игры по запросу пользователя.

| Компонент | Автор | Лицензия |
|-----------|--------|----------|
| ReShade addon (DLL) | [Patrick Mours (crosire)](https://reshade.me) | BSD 3-Clause |
| Шейдеры `.fx` | [crosire/reshade-shaders](https://github.com/crosire/reshade-shaders) и авторы файлов | см. заголовки `.fx` |

**Полный текст лицензии ReShade** (обязателен при распространении бинарников):

- В репозитории: [`src-tauri/presets/reshade/LICENSE-ReShade.txt`](src-tauri/presets/reshade/LICENSE-ReShade.txt)
- В установленном приложении: `presets/reshade/LICENSE-ReShade.txt` рядом с ресурсами GSM
- Сводка и шейдеры: [`ATTRIBUTION.txt`](src-tauri/presets/reshade/ATTRIBUTION.txt), [`shaders/THIRD-PARTY-NOTICES.txt`](src-tauri/presets/reshade/shaders/THIRD-PARTY-NOTICES.txt)

GSM **не связан** с ReShade и **не одобрен** авторами ReShade. Использование ReShade в онлайн-играх — на свой риск.

Пресеты GSM (Performance / Clarity / Cinematic) используют эффекты **Clarity** (Ioxa), **Vignette** (CeeJay.dk), **AdaptiveSharpen** (bacondither) — см. заголовки в `presets/reshade/shaders/Shaders/`.

---

Game Settings Master v0.2.0
