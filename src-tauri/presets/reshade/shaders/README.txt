Шейдеры подтягиваются скриптом из корня репозитория:

  .\scripts\fetch-reshade-shaders.ps1

Клонируется ветка nvidia репозитория crosire/reshade-shaders (pin commit в fetch-reshade-shaders.ps1).
Целевая папка: presets/reshade/shaders/Shaders/

Без шейдеров GSM ставит safe preset (Techniques= пусто).
Addon DLL: .\scripts\fetch-reshade-binaries.ps1
