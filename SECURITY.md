# Security Policy

[Русский](#русский) · [English](#english)

---

## Русский

### Поддерживаемые версии

| Версия | Поддержка |
|--------|-----------|
| Последний [Release](https://github.com/MikeSaito/Game-Settings-Master/releases/latest) | ✅ |
| Старые версии | ❌ |

### Сообщить об уязвимости

**Не создавайте публичный Issue** для проблем безопасности.

Напишите на **chigikovden@gmail.com** с темой `GSM Security`.

Укажите:
- версию приложения и Windows;
- шаги воспроизведения;
- ожидаемое и фактическое поведение;
- скриншоты или логи (без личных данных).

Ожидайте ответ в течение **7 дней**. Если уязвимость подтверждена — исправление в ближайшем patch/minor релизе.

### Область

В scope: установщик GSM, Tauri-бэкенд, автообновление (`latest.json`), обработка ini/конфигов игр.

Out of scope: сторонние игры, Steam/Epic, проблемы SmartScreen без подписи издателя.

### Известные транзитивные зависимости (Dependabot)

**`glib` (Rust, CVE в gtk-rs 0.18)** — тянется через Tauri → GTK **только для Linux-сборки**. Релиз GSM — **только Windows**; `glib` не входит в установщик. Dependabot не может обновить до `0.20.0`, пока Tauri/wry не перейдут на gtk-rs 0.20+. Alert закрыт как допустимый риск для Windows-only дистрибуции.

---

## English

### Supported versions

| Version | Supported |
|---------|-----------|
| Latest [Release](https://github.com/MikeSaito/Game-Settings-Master/releases/latest) | ✅ |
| Older releases | ❌ |

### Report a vulnerability

**Do not open a public Issue** for security problems.

Email **chigikovden@gmail.com** with subject `GSM Security`.

Include:
- app and Windows version;
- steps to reproduce;
- expected vs actual behavior;
- screenshots or logs (no personal data).

Expect a reply within **7 days**. Confirmed issues are fixed in the next patch/minor release.

### Scope

In scope: GSM installer, Tauri backend, auto-update (`latest.json`), ini/game config handling.

Out of scope: third-party games, Steam/Epic, SmartScreen warnings without publisher signing.

### Known transitive dependencies (Dependabot)

**`glib` (Rust, gtk-rs 0.18 advisory)** — pulled in via Tauri → GTK for **Linux builds only**. GSM ships **Windows-only**; `glib` is not in the installer. Dependabot cannot bump to `0.20.0` until Tauri/wry move to gtk-rs 0.20+. Alert dismissed as tolerable risk for Windows-only distribution.

---
