export interface Copy {
  lang: "ru" | "en";
  title: string;
  description: string;
  hero: string;
  intro: string;
  nav: string[];
  download: string;
  source: string;
  trust: string[];
  warning: string;
  help: string;
  demo: {
    label: string;
    title: string;
    disclaimer: string;
    modes: string[];
    names: string[];
    on: string;
    off: string;
    high: string;
    epic: string;
    original: string;
    preview: string;
    edit: string;
    confirm: string;
    reset: string;
    before: string;
    after: string;
    selected: string;
    empty: string;
    done: string;
    result: string;
    include: string;
    idle: string;
    changed: string;
  };
  workflow: string;
  steps: string[];
  featuresTitle: string;
  featuresKicker: string;
  caption: string;
  features: { title: string; text: string; note: string }[];
  extrasTitle: string;
  extras: { title: string; text: string }[];
  compatTitle: string;
  compatText: string;
  compatLabels: string[];
  limits: { title: string; text: string }[];
  final: string;
  finalText: string;
  release: string;
  faqTitle: string;
  faq: { id: string; question: string; text: string }[];
  donate: string;
  donateText: string;
  skip: string;
}

export const en: Copy = {
  lang: "en",
  title: "Game Settings Master – Unreal Engine settings under control",
  description:
    "A UE4 and UE5 settings editor for Windows with change previews, selective restore, presets and post-game configuration diagnostics.",
  hero: "Unreal Engine settings – with previews and restore.",
  intro:
    "From familiar graphics settings to engine parameters in INI files. Change values, review every edit and restore the settings you need from a snapshot.",
  nav: ["Features", "Compatibility"],
  download: "Download for Windows",
  source: "Source on GitHub",
  trust: ["Free", "Open source", "No account"],
  warning: "Windows may show a SmartScreen warning.",
  help: "Installation and signatures",
  demo: {
    label: "INTERACTIVE EXAMPLE",
    title: "Try it before you download",
    disclaimer: "Interactive example. Files on your computer are not changed.",
    modes: ["Basic settings", "Engine parameters"],
    names: [
      "Vertical synchronization",
      "Shadow quality",
      "Shadow map resolution",
      "Lumen reflections",
    ],
    on: "On",
    off: "Off",
    high: "High",
    epic: "Epic",
    original: "Original value",
    preview: "Review changes",
    edit: "Back to settings",
    confirm: "Confirm example",
    reset: "Start again",
    before: "Before",
    after: "After",
    selected: "Selected edits",
    empty: "No changes selected",
    done: "Example complete",
    result:
      "The app creates a backup before writing. This example only shows the result of your selected edits.",
    include: "Include edit",
    idle: "Change a value to see the diff.",
    changed: "Draft edits",
  },
  workflow: "Four steps. Every edit in view.",
  steps: ["Choose a game", "Change values", "Review changes", "Apply"],
  featuresKicker: "CONTROL AT EVERY STEP",
  featuresTitle: "You decide what goes into the config.",
  caption: "App interface · demonstration data",
  features: [
    {
      title: "Review first. Apply second.",
      text: "See the file, section and value before and after each change. Exclude individual edits before writing.",
      note: "If the config changes after the preview, the app asks you to refresh the diff.",
    },
    {
      title: "Restore one setting. Or a whole snapshot.",
      text: "Compare your config with a named snapshot and restore a parameter, a file or the entire backup.",
      note: "Selective restore preserves other data. Restoring also goes through a preview.",
    },
    {
      title: "See what changed after the game.",
      text: "After the game exits, the app checks the settings you applied and reports changes or a new config folder.",
      note: "Monitoring works while the app is open. The source of a change may be unknown.",
    },
  ],
  extrasTitle: "More tools for the way you play.",
  extras: [
    {
      title: "Presets with context",
      text: "Save selected edits or a full profile. Descriptions, game builds and GPU requirements help assess compatibility before import.",
    },
    {
      title: "Choose a GPU for settings",
      text: "Select an adapter and check dedicated memory and hardware ray tracing. NVIDIA, AMD and Intel; unavailable information is marked explicitly.",
    },
    {
      title: "Classic input bindings",
      text: "Edit recognized key bindings, axes and sensitivity in Input.ini with the same previews and backups.",
    },
  ],
  compatTitle: "Support depends on the game.",
  compatText:
    "Finding a game does not mean every engine parameter is available in its build.",
  compatLabels: ["Engines", "Game sources", "Hardware"],
  limits: [
    {
      title: "Game discovery",
      text: "Steam, Epic, GOG, manual paths and accessible Xbox / Game Pass installations. Restricted installations may not be discovered.",
    },
    {
      title: "Upscaling and GPUs",
      text: "Hardware and game plugin support are separate. A graphics card alone does not add DLSS, FSR or XeSS to a game.",
    },
    {
      title: "Input support",
      text: "Recognized classic Input.ini entries are editable. Enhanced Input and custom game formats are outside this editor; ambiguous entries are read-only.",
    },
    {
      title: "Engine version",
      text: "Parameters and hints depend on the detected UE version. Some games ignore INI values or overwrite them on launch.",
    },
  ],
  final: "Your settings. Your call.",
  finalText:
    "Start with basic graphics or explore engine parameters. Review the changes before writing.",
  release: "What's new in this release",
  faqTitle: "Before your first launch",
  faq: [
    {
      id: "smartscreen",
      question: "Why might Windows show SmartScreen?",
      text: "The installer does not have a Windows publisher signature (Authenticode), so SmartScreen may warn about an unknown app. Download from the official GitHub Releases page. If you trust the source, choose “More info” → “Run anyway”. The updater signature lets the app verify update packages; it does not replace a Windows publisher signature.",
    },
    {
      id: "restore",
      question: "Can I restore my original settings?",
      text: "The app saves a backup before writing changes. Restore an individual parameter, a file or an entire snapshot through a preview. Parameter values still need to suit the game's capabilities.",
    },
    {
      id: "games",
      question: "Does every setting work in every UE game?",
      text: "No. Available features depend on the engine version, game build and plugins. Compatibility hints cannot guarantee that a game will use every value.",
    },
    {
      id: "diagnostics",
      question: "Does the app need to stay open?",
      text: "Yes, to monitor game launches and exits. Diagnostics report detected changes after a game, but cannot always identify their source.",
    },
  ],
  donate: "Support the project",
  donateText: "For continued development and a Windows publisher signature.",
  skip: "Skip to content",
};

export const ru: Copy = {
  lang: "ru",
  title: "Game Settings Master – настройки Unreal Engine под контролем",
  description:
    "Редактор настроек UE4 и UE5 для Windows: предпросмотр изменений, частичный откат, пресеты и диагностика конфигов после игры.",
  hero: "Настройки Unreal Engine – с предпросмотром и откатом",
  intro:
    "От графики в меню игры до параметров движка в INI. Меняйте значения, проверяйте каждую правку и возвращайте нужные настройки из снимка.",
  nav: ["Возможности", "Совместимость"],
  download: "Скачать для Windows",
  source: "Исходники на GitHub",
  trust: ["Бесплатно", "Открытый код", "Без аккаунта"],
  warning: "Windows может показать предупреждение SmartScreen.",
  help: "Об установке и подписи",
  demo: {
    label: "ИНТЕРАКТИВНЫЙ ПРИМЕР",
    title: "Попробуйте перед скачиванием",
    disclaimer:
      "Интерактивный пример. Файлы на вашем компьютере не изменяются.",
    modes: ["Базовые настройки", "Параметры движка"],
    names: [
      "Вертикальная синхронизация",
      "Качество теней",
      "Разрешение карты теней",
      "Отражения Lumen",
    ],
    on: "Включено",
    off: "Выключено",
    high: "Высокое",
    epic: "Эпическое",
    original: "Исходное значение",
    preview: "Проверить изменения",
    edit: "Вернуться к настройкам",
    confirm: "Подтвердить пример",
    reset: "Начать заново",
    before: "Было",
    after: "Станет",
    selected: "Выбрано правок",
    empty: "Нет выбранных изменений",
    done: "Пример завершён",
    result:
      "В приложении перед записью создаётся резервная копия. Здесь показан только результат выбранных правок.",
    include: "Включить правку",
    idle: "Измените значение, чтобы увидеть дифф.",
    changed: "Правок в черновике",
  },
  workflow: "Четыре шага. Каждая правка на виду.",
  steps: [
    "Выберите игру",
    "Измените значения",
    "Проверьте изменения",
    "Примените",
  ],
  featuresKicker: "КОНТРОЛЬ НА КАЖДОМ ЭТАПЕ",
  featuresTitle: "Вы решаете, что попадёт в конфиг.",
  caption: "Интерфейс приложения · демонстрационные данные",
  features: [
    {
      title: "Сначала посмотрите. Потом примените.",
      text: "Файл, секция и значение до и после изменения. Исключите отдельные правки перед записью.",
      note: "Если конфиг изменился после предпросмотра, приложение попросит обновить дифф.",
    },
    {
      title: "Верните одну настройку. Или весь снимок.",
      text: "Сравните конфиг с именованным снимком и восстановите нужный параметр, файл или резервную копию целиком.",
      note: "Частичный откат сохраняет остальные данные. Восстановление тоже проходит через предпросмотр.",
    },
    {
      title: "Узнайте, что изменилось после игры.",
      text: "Приложение сравнивает применённые параметры после завершения игры и сообщает об изменениях или новой папке конфигов.",
      note: "Наблюдение работает, пока приложение открыто. Источник изменения может быть неизвестен.",
    },
  ],
  extrasTitle: "Больше возможностей для вашего сценария.",
  extras: [
    {
      title: "Пресеты с контекстом",
      text: "Сохраняйте выбранные правки или полный профиль. Описание, сборка игры и требования к GPU помогают проверить совместимость перед импортом.",
    },
    {
      title: "GPU для подбора настроек",
      text: "Выбирайте адаптер, учитывайте выделенную память и аппаратную трассировку. NVIDIA, AMD и Intel; недоступные сведения отмечаются явно.",
    },
    {
      title: "Классическое управление",
      text: "Редактируйте найденные привязки клавиш, осей и чувствительности из Input.ini с тем же предпросмотром и резервной копией.",
    },
  ],
  compatTitle: "Поддержка зависит от конкретной игры.",
  compatText:
    "Наличие игры в библиотеке не означает, что каждый параметр движка доступен в её сборке.",
  compatLabels: ["Движки", "Источники игр", "Оборудование"],
  limits: [
    {
      title: "Обнаружение игр",
      text: "Steam, Epic, GOG, ручные пути и доступные установки Xbox / Game Pass. Закрытые установки могут не обнаружиться.",
    },
    {
      title: "Апскейлинг и GPU",
      text: "Поддержка GPU и игрового плагина проверяется отдельно. Видеокарта сама по себе не добавляет DLSS, FSR или XeSS в игру.",
    },
    {
      title: "Управление",
      text: "Доступны распознанные классические записи Input.ini. Enhanced Input и собственные форматы игр не редактируются; неоднозначные записи доступны для просмотра.",
    },
    {
      title: "Версия движка",
      text: "Параметры и подсказки зависят от определённой версии UE. Некоторые игры игнорируют значения INI или перезаписывают их при запуске.",
    },
  ],
  final: "Ваши настройки. Ваше решение.",
  finalText:
    "Начните с базовой графики или откройте параметры движка. Перед записью проверьте изменения.",
  release: "Что нового в выпуске",
  faqTitle: "Перед первым запуском",
  faq: [
    {
      id: "smartscreen",
      question: "Почему Windows может показать SmartScreen?",
      text: "У установщика нет подписи издателя Windows (Authenticode), поэтому SmartScreen может предупредить о неизвестном приложении. Скачивайте файл из официального GitHub Releases. Если доверяете источнику, откройте «Подробнее» → «Выполнить в любом случае». Подпись обновлений проверяет подлинность пакета для приложения; она не заменяет подпись издателя Windows.",
    },
    {
      id: "restore",
      question: "Можно ли вернуть исходные настройки?",
      text: "Перед записью изменений приложение сохраняет резервную копию. Можно восстановить отдельный параметр, файл или снимок целиком через предпросмотр. Значения параметров всё равно должны соответствовать возможностям игры.",
    },
    {
      id: "games",
      question: "Будут ли работать все настройки в любой UE-игре?",
      text: "Нет. Возможности зависят от версии движка, сборки игры и плагинов. Подсказки не гарантируют, что игра применит каждое значение.",
    },
    {
      id: "diagnostics",
      question: "Нужно ли держать приложение открытым?",
      text: "Для наблюдения за запуском и завершением игры – да. Диагностика сообщает об обнаруженных изменениях после игры, но не всегда может установить их источник.",
    },
  ],
  donate: "Поддержать проект",
  donateText: "На дальнейшую разработку и подпись издателя Windows.",
  skip: "Перейти к содержимому",
};
