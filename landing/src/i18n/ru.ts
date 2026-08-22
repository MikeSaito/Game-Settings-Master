import type { LocaleStrings } from "./types";

export const ru: LocaleStrings = {
  lang: "ru",
  htmlLang: "ru",
  siteName: "Game Settings Master",
  meta: {
    title: "Game Settings Master – редактор конфигов Unreal Engine",
    description:
      "Интерактивная презентация редактора GameUserSettings.ini и Engine.ini: библиотека игр, Basic/Advanced, бэкапы, фильтры DLSS/FSR.",
    keywords:
      "настройки игр, Unreal Engine, UE4, UE5, DLSS, FSR, редактор конфигов, ini",
    ogLocale: "ru_RU",
  },
  chrome: {
    aria: "Скролл-презентация",
    langLabel: "Язык",
    ru: "RU",
    en: "EN",
    download: "Скачать",
  },
  story: {
    scrollHint: "Листайте вниз",
    hero: {
      kicker: "Редактор конфигов Unreal Engine",
      title: "Все настройки игр Unreal Engine в одном окне",
      text: "Библиотека находит игры сама. Basic как меню игры, Advanced с CVars, бэкапы перед каждой записью.",
    },
    scan: {
      kicker: "01 · Сканирование",
      title: "Игры на Unreal Engine найдутся сами",
      text: "Steam, Epic и ручные пути. Версия движка определяется сразу.",
      scanning: "Сканирование дисков…",
      counter: (found: number) => {
        const mod10 = found % 10;
        const mod100 = found % 100;
        const word =
          mod10 === 1 && mod100 !== 11
            ? "игра"
            : mod10 >= 2 && mod10 <= 4 && (mod100 < 12 || mod100 > 14)
              ? "игры"
              : "игр";
        return `Найдено: ${found} ${word}`;
      },
    },
    backup: {
      kicker: "02 · Бэкапы",
      title: "Сломать игру невозможно",
      text: "Снимок конфигов перед каждой записью. Откат в один клик.",
      badge: "backup_success",
      now: "только что",
    },
    modes: {
      kicker: "03 · Basic и Advanced",
      title: "От простых тумблеров до глубокого твикинга",
      text: "Basic как меню игры. Advanced: CVars с подсказками под версию UE.",
      fps: "FPS",
      tooltip: "UE 5.3: параметр поддерживается",
    },
    final: {
      kicker: "04 · Установка",
      title: "Забудь про Блокнот",
      text: "Бесплатно, open source, без аккаунта. Исходники на GitHub.",
      tagline: "Настраивай как профи",
      badges: ["Бесплатно", "Open source", "Без аккаунта"],
    },
    flow: {
      faqTitle: "Частые вопросы",
    },
  },
  mock: {
    windowTitle: "Game Settings Master",
    navLibrary: "Библиотека",
    navEditor: "Редактор",
    navSettings: "Настройки",
    search: "Поиск или название для ручного добавления…",
    scan: "Сканировать",
    add: "Добавить",
    libTitle: "Библиотека игр",
    libSubtitle: "Steam · Epic · Unreal Engine · ручное добавление",
    badgeTotal: (n: number) => `${n} всего`,
    badgeConfig: (n: number) => `${n} с конфигом`,
    badgeUe: (n: number) => `${n} UE`,
    badgeCover: (n: number) => `${n} с обложкой`,
    configOk: "Конфиг готов",
    configMissing: "Нужен конфиг",
    select: "Выбрать",
    cover: "Обложка",
    sourceSteam: "Steam",
    sourceEpic: "Epic",
    sourceManual: "Вручную",
    ctxPlay: "Играть",
    ctxConfig: "Конфиг",
    tabs: {
      basic: "Базовое",
      advanced: "Расширенное",
      presets: "Пресеты",
      backups: "Бэкапы",
    },
    basicHint: "Пользовательские настройки: как в меню игры",
    advancedHint: "Параметры движка и консольные переменные: для опытных",
    backupsHint: "Снимки конфигов перед применением и откат",
    basicSafe: "Только GameUserSettings.ini",
    advancedWarn: "Engine.ini: экспертные переменные могут конфликтовать с настройками игры.",
    paramsForEngine: "725 параметров для UE 5.0",
    known: "6 в справочнике",
    sgLimits: "sg макс. 3 · масштаб 25–200%",
    engineShort: "В ini: 38 из 142.",
    changes: (n: number) => {
      const mod10 = n % 10;
      const mod100 = n % 100;
      const word =
        mod10 === 1 && mod100 !== 11
          ? "изменение"
          : mod10 >= 2 && mod10 <= 4 && (mod100 < 12 || mod100 > 14)
            ? "изменения"
            : "изменений";
      return `${n} ${word}`;
    },
    discard: "Сбросить",
    applyBasic: "Применить в пользовательские настройки",
    applyAdvanced: "Применить (движок / масштабируемость)",
    inIni: "В ini",
    removeFromIni: "Удалить из ini",
    detail: {
      title: "Детали параметра",
      key: "r.DLSS.Enable",
      current: ["Текущее значение", "1"],
      type: ["Тип", "Boolean"],
      range: ["Диапазон", "0..1"],
      tier: "Tier A",
      desc: "DLSS-апскейлинг через Engine.ini: меньше нагрузка на GPU, выше FPS.",
      compat: ["Совместимость", "UE 5.0+"],
    },
    backupsCount: (n: number) => {
      const mod10 = n % 10;
      const mod100 = n % 100;
      const word =
        mod10 === 1 && mod100 !== 11
          ? "копия"
          : mod10 >= 2 && mod10 <= 4 && (mod100 < 12 || mod100 > 14)
            ? "копии"
            : "копий";
      return `${n} ${word}`;
    },
    howTitle: "Как это работает",
    howBody:
      "Перед каждым применением пресета или ручных правок создаётся снимок ini-файлов в локальной папке данных приложения.",
    listTitle: "Список резервных копий",
    listDesc: "Новые сверху.",
    restore: "Восстановить",
    gpu: "RTX 4070",
    newBackupId: "2026-08-22_20-15-04",
    games: [
      {
        id: "g1",
        name: "S.T.A.L.K.E.R. 2: Heart of Chornobyl",
        source: "steam",
        engine: "UE 5",
        version: "5.0",
        ok: true,
        fav: true,
        path: "C:\\Games\\Stalker2",
        config: "C:\\Games\\Stalker2\\Saved\\Config\\Windows",
        hue: 38,
      },
      {
        id: "g2",
        name: "Black Myth: Wukong",
        source: "steam",
        engine: "UE 5",
        version: "5.0",
        ok: true,
        path: "C:\\Games\\Wukong",
        config: "C:\\Games\\Wukong\\Saved\\Config\\Windows",
        hue: 14,
      },
      {
        id: "g3",
        name: "Fortnite",
        source: "epic",
        engine: "UE 5",
        version: "5.1",
        ok: true,
        path: "C:\\Games\\Fortnite",
        config: "C:\\Games\\Fortnite\\Saved\\Config\\Windows",
        hue: 262,
      },
      {
        id: "g4",
        name: "The Finals",
        source: "steam",
        engine: "UE 5",
        version: "5.2",
        ok: true,
        path: "D:\\Steam\\TheFinals",
        config: "D:\\Steam\\TheFinals\\Saved\\Config\\Windows",
        hue: 352,
      },
      {
        id: "g5",
        name: "Satisfactory",
        source: "steam",
        engine: "UE 5",
        version: "5.3",
        ok: true,
        path: "D:\\Steam\\Satisfactory",
        config: "D:\\Steam\\Satisfactory\\Saved\\Config\\Windows",
        hue: 24,
      },
      {
        id: "g6",
        name: "Palworld",
        source: "steam",
        engine: "UE 5",
        version: "5.1",
        ok: false,
        path: "D:\\Steam\\Palworld",
        config: "",
        hue: 190,
      },
    ],
    basicParams: [
      {
        key: "sg.ResolutionQuality",
        title: "Масштаб разрешения",
        value: "62",
        kind: "slider",
        min: 25,
        max: 200,
        demoTo: "100",
        demoAt: 0,
      },
      {
        key: "sg.TextureQuality",
        title: "Качество текстур",
        value: "Эпик",
        kind: "select",
        options: ["Низкое", "Среднее", "Высокое", "Эпик"],
        demoTo: "Высокое",
        demoAt: 0.45,
      },
      {
        key: "sg.ShadowQuality",
        title: "Качество теней",
        value: "Среднее",
        kind: "select",
        options: ["Низкое", "Среднее", "Высокое", "Эпик"],
      },
      {
        key: "FullscreenMode",
        title: "Режим окна",
        value: "Без рамки",
        kind: "select",
        options: ["Полный экран", "Без рамки", "Оконный"],
      },
      {
        key: "bUseVSync",
        title: "VSync",
        value: "True",
        kind: "toggle",
        demoTo: "False",
        demoAt: 0.75,
      },
    ],
    advancedParams: [
      {
        key: "r.DLSS.Enable",
        title: "DLSS",
        value: "1",
        kind: "toggle",
        tier: "Tier A",
      },
      {
        key: "r.RayTracing",
        title: "Ray Tracing",
        value: "1",
        kind: "toggle",
        tier: "Tier B",
        warning: "Влияет на FPS",
      },
      {
        key: "r.Lumen.Reflections.Allow",
        title: "Lumen Reflections",
        value: "1",
        kind: "toggle",
        tier: "Tier B",
      },
      {
        key: "r.Shadow.Virtual.Enable",
        title: "Virtual Shadow Maps",
        value: "1",
        kind: "toggle",
        tier: "Tier B",
      },
      {
        key: "r.Nanite.MaxPixelsPerEdge",
        title: "Nanite Max Pixels",
        value: "1.0",
        kind: "cvar",
        tier: "Tier A",
      },
      {
        key: "r.TSR.History.ScreenPercentage",
        title: "TSR Screen Percentage",
        value: "100",
        kind: "cvar",
        hint: "UE 5",
      },
    ],
    backups: [
      {
        id: "2026-06-26_14-32-01",
        label: "26.06.2026, 14:32",
        files: ["GameUserSettings.ini", "Engine.ini", "Scalability.ini"],
      },
      {
        id: "2026-06-25_09-18-44",
        label: "25.06.2026, 09:18",
        files: ["GameUserSettings.ini", "Engine.ini"],
      },
      {
        id: "2026-06-24_21-05-12",
        label: "24.06.2026, 21:05",
        files: ["GameUserSettings.ini"],
      },
    ],
  },
  faq: [
    {
      question: "Почему SmartScreen предупреждает?",
      paragraphs: [
        "Сборка без коммерческой подписи кода; для indie это обычно. Нажмите «Подробнее», затем «Выполнить в любом случае». Исходники открыты на GitHub.",
      ],
    },
    {
      question: "Basic безопасен?",
      paragraphs: [
        "Basic правит GameUserSettings.ini, тот же файл, что и меню игры. Engine.ini не трогается. Перед применением создаётся бэкап.",
      ],
    },
    {
      question: "Можно откатить?",
      paragraphs: [
        "Да. Каждый apply сохраняет снимок. В разделе бэкапов выберите точку и восстановите конфиги.",
      ],
    },
  ],
  download: {
    button: "Скачать для Windows",
    githubButton: "Исходники на GitHub",
    smartScreen: {
      title: "Первый запуск в Windows",
      intro:
        "Приложение без коммерческой подписи. SmartScreen может показать предупреждение. Для indie-софта это нормально.",
      step1: "Нажмите «Подробнее»",
      step2: "Затем «Выполнить в любом случае»",
      note: "После первого запуска Windows обычно больше не спрашивает.",
      confirm: "Понятно, скачать",
      cancel: "Отмена",
    },
  },
  donate: {
    title: "Поддержать разработку",
    text: "На подпись кода для Windows и дальнейшие обновления.",
    button: "Поддержать проект",
  },
  footer: {
    version: (v: string) => `Game Settings Master v${v}`,
    donateLink: "Поддержать проект",
    telegramLink: "Telegram",
  },
};
