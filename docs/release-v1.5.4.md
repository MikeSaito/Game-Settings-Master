## Обновление 1.5.4

- Исправлены конфликты одновременного сохранения профилей и пресетов. Занятое другим экземпляром хранилище теперь возвращает ошибку без бесконечного ожидания.
- Исправлено освобождение обработчиков фокуса окна и обработка запоздавших событий.
- При потере фокуса приостанавливается опрос восстановления и предупреждений поиска игр. Наблюдение за запуском и завершением игры продолжает работать в фоне.
- Поиск игр, чтение журналов восстановления и запросы диагностики перенесены из потока окна.
- Диагностика читает и разбирает каждый INI-файл один раз за проверку, сохраняя корректную обработку повторяющихся параметров и привязок.

Windows x64. Обновление внутри приложения подписано; подпись издателя Windows (Authenticode) пока отсутствует.

## Update 1.5.4

- Fixed conflicts between concurrent profile and preset saves. A store held by another instance now returns an error without waiting indefinitely.
- Fixed cleanup of window focus listeners and handling of delayed focus events.
- Recovery and discovery warning polling pause while the app is unfocused. Game launch and exit observation continues in the background.
- Moved game discovery, recovery journal reads, and diagnostic requests off the window thread.
- Diagnostics read and parse each INI file once per check while preserving duplicate parameter and input binding handling.

Windows x64. In-app updates are signed; Windows publisher signing (Authenticode) is not yet available.
