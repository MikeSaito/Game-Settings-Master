## Исправления запуска

- Убрана начальная черно-белая вспышка: окно появляется после подготовки интерфейса, с фоном и оформлением выбранной темы.
- Добавлен общий экран ожидания для проверки обновлений, загрузки интерфейса и поиска игр. Видны текущий этап и индикатор активности.
- Загрузка обновления показывает реальный прогресс, а проверка и установка отображаются отдельным этапом. Если размер загрузки неизвестен, индикатор остается активным.
- При ошибке проверки обновлений можно повторить попытку или продолжить офлайн.
- Убрана зависимость первого экрана от загрузки шрифтов из интернета. При сбое загрузки интерфейса доступен повторный запуск.
- Обновлены зависимости приложения и инструментов сборки, включая исправления безопасности Tauri CLI.

Windows x64. Обновление внутри приложения подписано; подпись издателя Windows (Authenticode) пока отсутствует.

## Startup fixes

- Removed the initial black-and-white flash by revealing the window after the interface is prepared, using the selected theme's background and window appearance.
- Added a consistent waiting screen for update checks, interface loading and game discovery, with the current stage and an activity indicator.
- Update downloads show actual progress; verification and installation have a separate stage. Unknown download sizes keep an active indicator.
- Failed update checks offer retry and offline recovery.
- Startup no longer depends on downloading web fonts. If the interface cannot load, the initial screen offers a reload action.
- Updated application and build dependencies, including Tauri CLI security fixes.

Windows x64. In-app updates are signed; Windows publisher signing (Authenticode) is not yet available.
