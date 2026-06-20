# QA Checklist — Game Settings Master v1.0.0

Ручная проверка перед релизом (548 bundled keys, fetch skip).  
Отмечайте `[x]` по мере прохождения.

---

## Установка и первый запуск

- [ ] Windows installer `Game-Settings-Master_1.0.0_x64-setup.exe` скачивается с GitHub Releases
- [ ] Приложение запускается после SmartScreen («Подробнее» → «Выполнить в любом случае») — см. README
- [ ] Язык RU/EN переключается, строки Advanced Editor на обоих языках

## Библиотека

- [ ] Scan Steam / Epic находит UE-игру **или** manual add с указанием config dir
- [ ] `engine_version` отображается / определяется для UE-игры
- [ ] Переход в Advanced Editor и Backups для выбранной игры

## Advanced Editor — Scalability

- [ ] Default tab = **Масштабируемость** / Scalability
- [ ] Видны sg.* (ShadowQuality, ViewDistanceQuality, …)
- [ ] **Tier hint** раскрывается под описанием (Low/Medium/High + r.* CVars)
- [ ] Toggle **«Рекомендуемые»** ON — короткий список; OFF — «Все из ini»
- [ ] Поиск с debounce (~180 ms) не лагает на длинном списке
- [ ] Category chips строятся только из текущей панели

## Advanced Editor — Engine

- [ ] Вкладка **Engine** показывает warning RU/EN + badge «Эксперт»
- [ ] Checkbox «Понял» скрывает warning (sessionStorage)
- [ ] Engine.ini CVars видны; engine toggles только на Engine tab
- [ ] Hash `#engine` / `#scalability` работает с back button

## Каталог и ini

- [ ] Параметр из ini игры виден даже если нет в reference catalog
- [ ] Reference-only ключ с `introduced_in` > engine_version скрыт (если не в ini)
- [ ] Curated title/description побеждает reference для того же key
- [ ] `catalog_recommended` попадает в фильтр «Рекомендуемые»

## Apply и бэкапы

- [ ] Apply custom создаёт backup (Backups tab)
- [ ] Restore из backup возвращает предыдущие значения
- [ ] Diff показывает изменённые keys

## GPU

- [ ] На AMD/NVIDIA без DLSS — опции DLSS скрыты или clamp
- [ ] `clampParamValue` не даёт выставить значение вне min/max catalog

## Регрессии

- [ ] Нет маршрутов wizard / ReShade / Unity / cloud presets
- [ ] `npm test` / cargo catalog tests green в CI

---

**Tester:** _______________  
**Date:** _______________  
**Build:** v1.0.0  
**Notes:**
