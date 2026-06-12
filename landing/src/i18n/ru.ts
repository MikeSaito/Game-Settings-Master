import illLibrary from "../svg/ill-library.svg?raw";
import illPresets from "../svg/ill-presets.svg?raw";
import illGpu from "../svg/ill-gpu.svg?raw";
import illEditor from "../svg/ill-editor.svg?raw";
import illBackup from "../svg/ill-backup.svg?raw";
import illCloud from "../svg/ill-cloud.svg?raw";
import illReshade from "../svg/ill-reshade.svg?raw";
import type { LocaleStrings } from "./types";

export const ru: LocaleStrings = {
  lang: "ru",
  htmlLang: "ru",
  siteName: "Game Settings Master",
  meta: {
    title: "Game Settings Master — мастер графики для игр",
    description:
      "Пресеты, ручной редактор, ReShade и облачная синхронизация настроек для Unreal Engine, Unity и авторских разборов.",
    keywords:
      "game settings, пресеты графики, Unreal Engine, Unity, настройки игр, DLSS, FSR, ReShade",
    ogLocale: "ru_RU",
  },
  nav: {
    features: "Возможности",
    download: "Скачать",
    aria: "Навигация",
  },
  hero: {
    badge: "Game Settings Master",
    title: "Настройки игр",
    titleAccent: "в фокусе",
    subtitle:
      "Мастер графики для Unreal Engine, Unity и авторских разборов других игр — без ручного ковыряния в конфигах.",
  },
  engineTags: ["UE 4", "UE 5", "Unity", "ReShade", "Авторские разборы"],
  features: [
    {
      id: "library",
      step: "01",
      title: "Библиотека игр",
      text: "Сканирование Steam и Epic, ручное добавление. Приложение само находит папку конфигурации.",
      illustration: illLibrary,
    },
    {
      id: "presets",
      step: "02",
      title: "Авторские пресеты",
      text: "Готовые наборы настроек для разобранных автором игр (например Forza Horizon 6) — применение в один клик с предпросмотром diff. Видно каждую правку в конфигах.",
      illustration: illPresets,
      reverse: true,
    },
    {
      id: "smart",
      step: "03",
      title: "Умная настройка",
      text: "DLSS, FSR, ray tracing и Frame Generation — безопасный clamp под ваш GPU. Без бессмысленных опций на слабом железе.",
      illustration: illGpu,
    },
    {
      id: "editor",
      step: "04",
      title: "Ручной редактор",
      text: "Интерактивные ползунки, переключатели и списки для ключевых параметров UE4/UE5 — с описаниями, категориями и зависимостями.",
      illustration: illEditor,
      reverse: true,
    },
    {
      id: "backup",
      step: "05",
      title: "Бэкапы",
      text: "Snapshot перед каждым apply. Откат к предыдущему состоянию одним кликом — без страха сломать конфиг.",
      illustration: illBackup,
    },
    {
      id: "cloud",
      step: "06",
      title: "Облачные пресеты",
      text: "Контент с сервера синхронизируется без релиза приложения. Offline — встроенный fallback из кэша.",
      illustration: illCloud,
      reverse: true,
    },
    {
      id: "reshade",
      step: "07",
      title: "ReShade",
      text: "Установка post-processing в папку игры: пресеты Performance, Clarity и Cinematic, авторские ini для отдельных игр. Запуск с ReShade или без — proxy снимается автоматически.",
      illustration: illReshade,
    },
  ],
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
    text: "Сбор средств на сертификат подписи Windows и дальнейшее развитие приложения.",
    button: "Поддержать проект",
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
