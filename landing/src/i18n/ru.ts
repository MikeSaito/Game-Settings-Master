import illLibrary from "../svg/ill-library.svg?raw";
import illPresets from "../svg/ill-presets.svg?raw";
import illGpu from "../svg/ill-gpu.svg?raw";
import illEditor from "../svg/ill-editor.svg?raw";
import illBackup from "../svg/ill-backup.svg?raw";
import illCloud from "../svg/ill-cloud.svg?raw";
import type { LocaleStrings } from "./types";

export const ru: LocaleStrings = {
  lang: "ru",
  htmlLang: "ru",
  siteName: "Game Settings Master",
  meta: {
    title: "Game Settings Master — мастер графики для игр",
    description:
      "Пресеты, ручной редактор и облачная синхронизация настроек для Unreal Engine, Unity и авторских разборов.",
    keywords:
      "game settings, пресеты графики, Unreal Engine, Unity, настройки игр, DLSS, FSR",
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
  engineTags: ["UE 4", "UE 5", "Unity", "Авторские разборы"],
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
      title: "Пресеты в один клик",
      text: "От Ultra Low до Ultra High — с предпросмотром diff до применения. Видно каждую правку в конфигах.",
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
      text: "Более сотни параметров с описаниями, категориями и зависимостями.",
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
  footer: {
    version: (v: string) => `Game Settings Master v${v}`,
  },
  localeSwitch: {
    label: "Язык",
    ru: "RU",
    en: "EN",
  },
};
