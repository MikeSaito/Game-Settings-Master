# Game Settings Master v1.0.7

Patch-релиз исправляет надёжность применения настроек и восстановления конфигурации.

**Дата:** 2026-09-10

## Исправлено

- Изменение VSync и других параметров больше не перезаписывает разрешение экрана.
- Изменения применяются только к активной платформенной папке конфигурации.
- JSON профилей и пресетов заменяется атомарно без предварительного удаления оригинала.
- Каждый backup получает уникальный ID; коллизии и смешанные снимки исключены.
- Старые backup автоматически переносятся в стабильное SHA-256 хранилище.
- Backend проверяет конфликтные комбинации по итоговому состоянию после apply.
- Сбой Steam или Epic scanner теперь возвращается как ошибка, а не как пустая библиотека.
- Папка с именем `Temp` вне системной временной директории больше не считается недопустимой.
- Тесты изолированы от пользовательских AppData и переменных окружения.
- Удалена недостоверная метка последнего применённого пресета.

## Проверки

- Rust tests, clippy и formatting
- TypeScript build и Vitest
- Catalog, i18n и generated-file verification
- Playwright E2E

---

This patch release improves the reliability of applying and restoring game configuration.

## Fixed

- Changing VSync or another setting no longer overwrites the screen resolution.
- Changes are written only to the active platform configuration directory.
- Profile and preset JSON files are replaced atomically without deleting the original first.
- Every backup has a unique ID, preventing collisions and mixed snapshots.
- Existing backups are migrated to a stable SHA-256 storage namespace.
- Backend combo validation now evaluates the effective post-apply state.
- Steam or Epic scanner crashes return an error instead of an empty library.
- A regular directory named `Temp` is no longer mistaken for the system temporary directory.
- Tests no longer use the user's AppData or mutate process environment variables.
- The stale last-applied-preset label was removed.
