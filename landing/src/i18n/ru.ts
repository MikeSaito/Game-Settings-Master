import illLibrary from "../svg/ill-library.svg?raw";
import illGpu from "../svg/ill-gpu.svg?raw";
import illEditor from "../svg/ill-editor.svg?raw";
import illBackup from "../svg/ill-backup.svg?raw";
import illCatalog from "../svg/ill-cloud.svg?raw";
import type { LocaleStrings } from "./types";

export const ru: LocaleStrings = {
  lang: "ru",
  htmlLang: "ru",
  siteName: "Game Settings Master",
  meta: {
    title: "Game Settings Master — настройки игр в фокусе",
    description:
      "Тёплый и точный редактор настроек Unreal Engine: базовые GameUserSettings, расширенные Engine.ini CVars, GPU-фильтры и бэкапы.",
    keywords:
      "game settings, Unreal Engine, UE4, UE5, настройки игр, DLSS, FSR, config editor, ini",
    ogLocale: "ru_RU",
  },
  nav: {
    features: "Возможности",
    howItWorks: "Как работает",
    catalog: "Каталог",
    faq: "FAQ",
    download: "Скачать",
    aria: "Навигация",
  },
  hero: {
    badge: "Game Settings Master",
    title: "Настройки игр",
    titleAccent: "в фокусе",
    subtitle:
      "Базовые GameUserSettings для быстрых правок и расширенный Engine.ini для точной настройки. С подсказками, GPU-aware фильтрами и бэкапом перед каждым применением.",
  },
  engineTags: ["Basic", "Advanced", "UE 4", "UE 5", "Backups"],
  stats: [
    { value: "725", label: "CVars в каталоге" },
    { value: "10", label: "версий UE 4.27–5.8" },
    { value: "UE 4/5", label: "version-aware подсказки" },
    { value: "Free", label: "Windows build" },
  ],
  basicAdvanced: {
    eyebrow: "Два режима",
    title: "Безопасный старт и экспертная точность",
    text: "Лендинг повторяет логику desktop-приложения: сначала понятные настройки GameUserSettings, затем осознанное редактирование Engine.ini.",
    basic: {
      label: "Базовое",
      title: "GameUserSettings без риска",
      text: "Для быстрых изменений, которые обычно уже есть в меню игры.",
      bullets: ["sg.* качество", "разрешение и режим окна", "VSync, FPS limit, display поля"],
    },
    advanced: {
      label: "Расширенное",
      title: "Engine.ini под контролем",
      text: "Для опытных пользователей: CVars, версии UE и бэкап перед apply.",
      bullets: ["r.* и другие engine CVars", "подсказки по версиям UE", "откат через snapshot"],
    },
  },
  features: [
    {
      id: "library",
      step: "01",
      title: "Библиотека игр",
      text: "Steam, Epic и ручное добавление: приложение находит конфиги и показывает контекст игры перед редактированием.",
      illustration: illLibrary,
    },
    {
      id: "editor",
      step: "02",
      title: "Базовое / Расширенное",
      text: "Сначала понятные GameUserSettings и sg.*, затем экспертные CVars из Engine.ini с предупреждениями и бэкапами.",
      illustration: illEditor,
      reverse: true,
    },
    {
      id: "smart",
      step: "03",
      title: "GPU-aware фильтры",
      text: "DLSS, FSR, ray tracing и Frame Generation показываются с учётом возможностей GPU, чтобы не предлагать бессмысленные переключатели.",
      illustration: illGpu,
    },
    {
      id: "backup",
      step: "04",
      title: "Бэкапы",
      text: "Snapshot перед каждым apply. Можно экспериментировать с ini и быстро вернуть предыдущее состояние.",
      illustration: illBackup,
      reverse: true,
    },
    {
      id: "catalog-feature",
      step: "05",
      title: "Каталог описаний",
      text: "725 объединённых ключей, official sg.*, GameUserSettings и редактируемые неизвестные параметры из ini игры.",
      illustration: illCatalog,
    },
  ],
  howItWorks: {
    eyebrow: "Рабочий поток",
    title: "От сканирования до apply за три шага",
    steps: [
      {
        step: "01",
        title: "Сканирование",
        text: "Найдите игру автоматически или добавьте путь вручную. GSM определит конфиги и движок.",
      },
      {
        step: "02",
        title: "Настройка",
        text: "Выберите Basic для быстрых правок или Advanced для Engine.ini и CVars.",
      },
      {
        step: "03",
        title: "Apply с backup",
        text: "Перед записью создаётся snapshot, чтобы откат был понятным и быстрым.",
      },
    ],
  },
  catalogHighlight: {
    eyebrow: "Каталог",
    title: "725 merged keys с привязкой к версиям UE",
    text: "Каталог объединяет UE 4.27–5.8, показывает tier-подсказки и помогает фильтровать рекомендуемые параметры вместо слепого перебора.",
    bullets: [
      "full fetch для разработчиков через локальный UE reference",
      "tier tooltips для приоритета и риска",
      "unknown keys остаются редактируемыми",
    ],
  },
  gpu: {
    eyebrow: "GPU",
    title: "DLSS / FSR без лишнего шума",
    text: "GPU-aware clamp помогает скрывать неподходящие опции и держит фокус на параметрах, которые реально применимы к железу пользователя.",
    bullets: ["DLSS и Frame Generation", "FSR и апскейлеры", "ray tracing capability checks"],
  },
  faq: {
    eyebrow: "FAQ",
    title: "Частые вопросы",
    items: [
      {
        question: "Почему Windows SmartScreen предупреждает при запуске?",
        answer: "Сборка пока без коммерческой подписи издателя. Код и релизы доступны на GitHub, а предупреждение обычно исчезает после первого запуска.",
      },
      {
        question: "Базовый режим безопасен?",
        answer: "Он работает с GameUserSettings и sg.* параметрами, которые ближе всего к обычным игровым настройкам. Перед применением всё равно создаётся бэкап.",
      },
      {
        question: "Что такое Engine.ini?",
        answer: "Это конфигурационный файл Unreal Engine, где хранятся низкоуровневые CVars. Расширенный режим предназначен для пользователей, которые понимают эффект таких правок.",
      },
      {
        question: "Можно ли откатить изменения?",
        answer: "Да. GSM создаёт snapshot перед apply, а раздел бэкапов помогает восстановить предыдущую версию конфигов.",
      },
      {
        question: "Зачем разработчикам Epic fetch?",
        answer: "Он нужен для локального обновления UE reference и пересборки каталога параметров из исходников/ini Unreal Engine.",
      },
      {
        question: "UE4 и UE5 отличаются в подсказках?",
        answer: "Да. Каталог хранит данные по версиям UE 4.27–5.8, чтобы подсказки и рекомендации учитывали поколение движка.",
      },
    ],
  },
  download: {
    title: "Скачать приложение",
    subtitle: "Windows · бесплатно · без подписи издателя",
    button: "Скачать",
    githubButton: "GitHub",
    smartScreen: {
      title: "Первый запуск в Windows",
      intro:
        "Приложение пока без коммерческой подписи — SmartScreen может показать синее предупреждение. Для indie-софта это нормально.",
      step1: "Нажмите «Подробнее»",
      step2: "Затем «Выполнить в любом случае»",
      note: "После первого запуска Windows обычно больше не спрашивает.",
      confirm: "Понятно, скачать",
      cancel: "Отмена",
    },
  },
  donate: {
    title: "Поддержать разработку",
    text: "Помогите оплатить подпись кода для Windows и будущие обновления.",
    button: "Поддержать",
  },
  footer: {
    version: (v: string) => `Game Settings Master v${v}`,
    donateLink: "Поддержать проект",
  },
  localeSwitch: {
    label: "Язык",
    ru: "RU",
    en: "EN",
  },
};
