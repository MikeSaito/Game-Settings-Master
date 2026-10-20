# Game Settings Master v1.4.1

Исправление надёжности сохранения конфигов.

- Перед изменением файлов приложение сохраняет журнал и исходные данные. После аварийного закрытия оно предлагает восстановление; запись в затронутую папку блокируется до его завершения.
- Восстановление можно повторить после нового сбоя. Оно возвращает исходные файлы и удаляет файлы, созданные прерванной операцией.
- Публикация защищена от перезаписи версий. Перед выпуском проверяются подпись обновления Tauri, установка, запуск и сквозные Windows-сценарии сохранения, восстановления и обновления.
- Согласованы общие правила валидации и типы операций.

Журнал защищает операции, выполненные начиная с этой версии. Он не позволяет восстановить незаписанный журнал старой версии. Многофайловая атомарность при отключении питания не гарантируется.

## English

Reliability hotfix for configuration writes. The app journals original files before writing, offers recovery after a crash, and blocks writes to affected folders until recovery completes. Recovery can resume after another interruption and removes files created by the interrupted operation.

Releases are immutable and gated on Tauri update signature verification, Windows installation and launch, and real desktop save, recovery and updater tests. Shared validation rules and typed operations keep frontend and backend behavior aligned.

Recovery covers operations journaled by this version. It cannot reconstruct unjournaled operations from older versions; multi-file atomicity during power loss is not guaranteed.
