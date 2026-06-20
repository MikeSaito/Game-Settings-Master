# Game Settings Master

[Русский](README.md) · [English](README.en.md) · [Поддержите разработку](https://dalink.to/mike_saito)

**Настройки игр в фокусе**

Читайте и настраивайте ini/boot.config игр на UE и Unity — с описаниями параметров, фильтрами под GPU и бэкапами.

`UE 4` · `UE 5` · `Unity`

## Возможности

**01 — Библиотека игр**  
Сканирование Steam и Epic, ручное добавление. Приложение само находит папку конфигурации.

**02 — Редактор параметров**  
Главная вкладка игры: интерактивные ползунки, переключатели и списки для ключевых параметров UE4/UE5 и Unity — с описаниями, категориями и зависимостями.

**03 — GPU-aware фильтры**  
DLSS, FSR, ray tracing и Frame Generation — безопасный clamp под ваш GPU. Без бессмысленных опций на слабом железе.

**04 — Бэкапы**  
Snapshot перед каждым apply. Откат к предыдущему состоянию одним кликом — без страха сломать конфиг.

**05 — Каталог описаний параметров**  
Встроенные metadata для редактора (`src-tauri/catalog/`) — не готовые пресеты, а справочник ключей, секций и подсказок для Advanced Editor.

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

`install-githooks.ps1` (опционально) включает pre-commit: `npm test` + проверка синхронизации `bindings.ts`.

### Каталог параметров (UE / Unity)

Source of truth — `src-tauri/catalog/` (`ue4.json`, `engine.json`, `display.json`, `scalability.json`, `unity.json`, `key_hints.json`). Редактируйте JSON напрямую; отдельного VPS-сервера нет.

После изменения IPC DTO в Rust (`src-tauri/src/models.rs` и связанные типы):

```powershell
npm run types:gen
```

Закоммитьте `src/lib/bindings.ts` (CI: `scripts/verify-types-sync.ps1`).

---

Game Settings Master v1.0.0
