# Game Settings Master v1.0.8

Патч исправляет сохранность правок в редакторе и восстановление бэкапов.

**Дата:** 2026-10-01

## Исправлено

- Применение Basic и Advanced записывает и удаляет ключи только своей панели.
- Обновление конфигов после применения сохраняет несохранённые правки другой панели и изменения, внесённые во время операции.
- Переключатели одинаковых ключей в разных секциях INI работают независимо.
- Исходный список ключей сохраняется между запусками: параметры, добавленные после первого открытия в этой версии, остаются доступными для удаления.
- Список бэкапов использует активную платформенную папку. Если активная папка изменилась после выбора снимка, восстановление возвращает его в исходную папку.
- Кэш параметров обновляется после применения, даже если пользователь уже переключился на другую игру.

Существующие ключи при первом открытии после обновления считаются исходными: предыдущие версии не сохраняли сведения об их происхождении.

---

This patch preserves editor changes and fixes backup restoration.

## Fixed

- Basic and Advanced apply both writes and removals only within their own panel.
- Refreshing configuration after apply preserves unsaved changes in other panels and edits made while the operation is pending.
- Identical keys in different INI sections have independent toggles.
- The initial key baseline persists across app restarts. Keys added after first opening the configuration in this version remain removable.
- Backups are listed for the active platform directory. If the active directory changes after selecting a snapshot, restoration uses the snapshot's original directory.
- Parameter caches refresh after apply even when the user has switched to another game.

Keys already present when first opening a configuration after updating are treated as original. Earlier versions did not persist their origin.
