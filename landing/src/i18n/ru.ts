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
    title: "Game Settings Master — редактор конфигов UE и Unity",
    description:
      "Читайте и настраивайте ini/boot.config игр на UE и Unity — с описаниями параметров, фильтрами под GPU и бэкапами.",
    keywords:
      "game settings, Unreal Engine, Unity, настройки игр, DLSS, FSR, config editor, ini",
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
      "Читайте и настраивайте ini/boot.config игр на UE и Unity — с описаниями параметров, фильтрами под GPU и бэкапами.",
  },
  engineTags: ["UE 4", "UE 5", "Unity"],
  features: [
    {
      id: "library",
      step: "01",
      title: "Библиотека игр",
      text: "Сканирование Steam и Epic, ручное добавление. Приложение само находит папку конфигурации.",
      illustration: illLibrary,
    },
    {
      id: "editor",
      step: "02",
      title: "Редактор параметров",
      text: "Интерактивные ползунки, переключатели и списки для ключевых параметров UE4/UE5 и Unity — с описаниями, категориями и зависимостями.",
      illustration: illEditor,
      reverse: true,
    },
    {
      id: "smart",
      step: "03",
      title: "GPU-aware фильтры",
      text: "DLSS, FSR, ray tracing и Frame Generation — безопасный clamp под ваш GPU. Без бессмысленных опций на слабом железе.",
      illustration: illGpu,
    },
    {
      id: "backup",
      step: "04",
      title: "Бэкапы",
      text: "Snapshot перед каждым apply. Откат к предыдущему состоянию одним кликом — без страха сломать конфиг.",
      illustration: illBackup,
      reverse: true,
    },
    {
      id: "catalog",
      step: "05",
      title: "Каталог описаний",
      text: "Встроенные metadata для редактора — справочник ключей, секций и подсказок. Не готовые пресеты, а описания параметров.",
      illustration: illCatalog,
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
