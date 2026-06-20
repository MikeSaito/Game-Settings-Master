import illLibrary from "../svg/ill-library.svg?raw";
import illGpu from "../svg/ill-gpu.svg?raw";
import illEditor from "../svg/ill-editor.svg?raw";
import illBackup from "../svg/ill-backup.svg?raw";
import illCatalog from "../svg/ill-cloud.svg?raw";
import type { LocaleStrings } from "./types";

export const en: LocaleStrings = {
  lang: "en",
  htmlLang: "en",
  siteName: "Game Settings Master",
  meta: {
    title: "Game Settings Master — UE & Unity config editor",
    description:
      "Read and tune Unreal Engine and Unity game configs — with parameter descriptions, GPU-aware options, and backups.",
    keywords:
      "game settings, Unreal Engine, Unity, DLSS, FSR, config editor, ini",
    ogLocale: "en_US",
  },
  nav: {
    features: "Features",
    download: "Download",
    aria: "Navigation",
  },
  hero: {
    badge: "Game Settings Master",
    title: "Game settings",
    titleAccent: "in focus",
    subtitle:
      "Read and tune Unreal Engine and Unity game configs — with parameter descriptions, GPU-aware options, and backups.",
  },
  engineTags: ["UE 4", "UE 5", "Unity"],
  features: [
    {
      id: "library",
      step: "01",
      title: "Game library",
      text: "Steam and Epic scan, manual add. The app finds your config folder automatically.",
      illustration: illLibrary,
    },
    {
      id: "editor",
      step: "02",
      title: "Parameter editor",
      text: "Interactive sliders, toggles and dropdowns for key UE4/UE5 and Unity parameters — with descriptions, categories and dependencies.",
      illustration: illEditor,
      reverse: true,
    },
    {
      id: "smart",
      step: "03",
      title: "GPU-aware filters",
      text: "DLSS, FSR, ray tracing and Frame Generation — safe clamp for your GPU. No pointless options on weak hardware.",
      illustration: illGpu,
    },
    {
      id: "backup",
      step: "04",
      title: "Backups",
      text: "Snapshot before every apply. Roll back to the previous state in one click — no fear of breaking your config.",
      illustration: illBackup,
      reverse: true,
    },
    {
      id: "catalog",
      step: "05",
      title: "Parameter catalog",
      text: "Bundled metadata for the editor — a reference of keys, sections and hints. Not ready-made presets, but parameter descriptions.",
      illustration: illCatalog,
    },
  ],
  download: {
    title: "Download the app",
    subtitle: "Windows · free · unsigned build",
    button: "Download",
    githubButton: "GitHub",
    smartScreen: {
      title: "First launch on Windows",
      intro:
        "The app is not commercially signed yet — SmartScreen may show a blue warning. That's normal for indie software.",
      step1: 'Click "More info"',
      step2: 'Then "Run anyway"',
      note: "After the first run, Windows usually stops asking.",
      confirm: "Got it, download",
      cancel: "Cancel",
    },
  },
  donate: {
    title: "Support development",
    text: "Help fund a Windows code signing certificate and future updates.",
    button: "Donate",
  },
  footer: {
    version: (v: string) => `Game Settings Master v${v}`,
    donateLink: "Support the project",
  },
  localeSwitch: {
    label: "Language",
    ru: "RU",
    en: "EN",
  },
};
