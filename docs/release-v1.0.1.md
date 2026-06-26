# Game Settings Master v1.0.1 — Release Notes

[Русский](#русский) · [English](#english)

---

## Русский

**Дата:** 2026-06-21  
**Тип:** bugfix / polish

### Исправления

- **Библиотека:** кнопка «Библиотека» стабильно возвращает к списку игр
- **Тема:** по умолчанию «Системная» вместо тёмной
- **Расширенный редактор:** скролл не сбрасывается при изменении параметра
- **Расширенный режим:** кнопки «Добавить в ini» / «Удалить из ini» вместо тумблера
- **Показатели:** перевод и humanize для CVars (RU/EN), tier hints без прыжков layout при hover
- **Настройки:** выбор языка через выпадающий список; убрана «Стартовая вкладка редактора»

### Производительность

- Тяжёлые IPC-запросы приостанавливаются, когда окно GSM в фоне
- Опрос «игра запущена» — раз в 10 с (было 4 с)
- Обновление с диска при возврате фокуса — не чаще 1 раза в минуту

### Автотесты

- `npm test`: 96 passed
- `cargo test`: 149 passed

---

## English

**Date:** 2026-06-21  
**Type:** bugfix / polish

### Fixes

- **Library:** nav reliably returns to the game list
- **Theme:** default is System instead of dark
- **Advanced Editor:** scroll position preserved when editing a parameter
- **Advanced mode:** Add to ini / Remove from ini buttons instead of a toggle
- **Parameters:** RU/EN humanized CVar labels; tier hint row no longer jumps on hover
- **Settings:** language dropdown; removed default editor tab setting

### Performance

- Heavy IPC queries pause while GSM window is in background
- Game-running poll every 10 s (was 4 s)
- Disk refresh on focus throttled to once per minute

### Automated tests

- `npm test`: 96 passed
- `cargo test`: 149 passed
