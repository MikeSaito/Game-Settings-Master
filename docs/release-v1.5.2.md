## Обновление 1.5.2

- Обновлена TLS-библиотека механизма обновления приложения до rustls 0.23.46.
- Исправлен неиспользуемый экспорт функции сканирования в тестовой Windows-сборке. CI теперь проверяет Clippy с запретом предупреждений как для обычной сборки, так и для `native-test`.

Windows x64. Обновление внутри приложения подписано; подпись издателя Windows (Authenticode) пока отсутствует.

## Update 1.5.2

- Updated the application's updater TLS library to rustls 0.23.46.
- Fixed an unused discovery export in the native Windows test build. CI now runs Clippy with warnings denied for both the default and `native-test` configurations.

Windows x64. In-app updates are signed; Windows publisher signing (Authenticode) is not yet available.
