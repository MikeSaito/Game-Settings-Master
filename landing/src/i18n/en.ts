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
    title: "Game Settings Master — game settings in focus",
    description:
      "A warm, precise Unreal Engine settings editor: Basic GameUserSettings, Advanced Engine.ini CVars, GPU-aware filters, and backups.",
    keywords:
      "game settings, Unreal Engine, UE4, UE5, DLSS, FSR, config editor, ini",
    ogLocale: "en_US",
  },
  nav: {
    features: "Features",
    howItWorks: "How it works",
    catalog: "Catalog",
    faq: "FAQ",
    download: "Download",
    aria: "Navigation",
  },
  hero: {
    badge: "Game Settings Master",
    title: "Game settings",
    titleAccent: "in focus",
    subtitle:
      "Basic GameUserSettings for quick edits and Advanced Engine.ini control for precise tuning. Includes hints, GPU-aware filters, and a backup before every apply.",
  },
  engineTags: ["Basic", "Advanced", "UE 4", "UE 5", "Backups"],
  stats: [
    { value: "725", label: "catalog CVars" },
    { value: "10", label: "UE versions 4.27–5.8" },
    { value: "UE 4/5", label: "version-aware hints" },
    { value: "Free", label: "Windows build" },
  ],
  basicAdvanced: {
    eyebrow: "Two modes",
    title: "Safe starts and expert precision",
    text: "The landing mirrors the desktop app: approachable GameUserSettings first, intentional Engine.ini editing when you need deeper control.",
    basic: {
      label: "Basic",
      title: "GameUserSettings without guesswork",
      text: "For quick changes that usually match in-game settings.",
      bullets: ["sg.* quality", "resolution and window mode", "VSync, FPS limit, display fields"],
    },
    advanced: {
      label: "Advanced",
      title: "Engine.ini under control",
      text: "For experienced users: CVars, UE version hints, and a backup before apply.",
      bullets: ["r.* and other engine CVars", "UE version-aware hints", "rollback through snapshots"],
    },
  },
  features: [
    {
      id: "library",
      step: "01",
      title: "Game library",
      text: "Steam, Epic, and manual paths: the app finds config folders and shows game context before you edit.",
      illustration: illLibrary,
    },
    {
      id: "editor",
      step: "02",
      title: "Basic / Advanced",
      text: "Start with clear GameUserSettings and sg.* controls, then move into Engine.ini CVars with warnings and backups.",
      illustration: illEditor,
      reverse: true,
    },
    {
      id: "smart",
      step: "03",
      title: "GPU-aware filters",
      text: "DLSS, FSR, ray tracing, and Frame Generation are filtered by GPU capability, so the UI avoids pointless switches.",
      illustration: illGpu,
    },
    {
      id: "backup",
      step: "04",
      title: "Backups",
      text: "Snapshot before every apply. Experiment with ini changes and return to the previous state quickly.",
      illustration: illBackup,
      reverse: true,
    },
    {
      id: "catalog",
      step: "05",
      title: "Parameter catalog",
      text: "725 merged keys, official sg.*, GameUserSettings, and editable unknown parameters from game ini files.",
      illustration: illCatalog,
    },
  ],
  howItWorks: {
    eyebrow: "Workflow",
    title: "From scan to apply in three steps",
    steps: [
      {
        step: "01",
        title: "Scan",
        text: "Find a game automatically or add a path manually. GSM detects configs and engine context.",
      },
      {
        step: "02",
        title: "Tune",
        text: "Choose Basic for quick edits or Advanced for Engine.ini and CVars.",
      },
      {
        step: "03",
        title: "Apply with backup",
        text: "A snapshot is created before writing, so rollback is clear and fast.",
      },
    ],
  },
  catalogHighlight: {
    eyebrow: "Catalog",
    title: "725 merged keys with UE version context",
    text: "The catalog merges UE 4.27–5.8, adds tier hints, and helps filter recommended parameters instead of forcing blind search.",
    bullets: [
      "full fetch for developers through local UE reference",
      "tier tooltips for priority and risk",
      "unknown keys remain editable",
    ],
  },
  gpu: {
    eyebrow: "GPU",
    title: "DLSS / FSR without the noise",
    text: "GPU-aware clamp hides options that do not apply and keeps attention on settings that can actually work on the user's hardware.",
    bullets: ["DLSS and Frame Generation", "FSR and upscalers", "ray tracing capability checks"],
  },
  faq: {
    eyebrow: "FAQ",
    title: "Common questions",
    items: [
      {
        question: "Why does Windows SmartScreen warn on launch?",
        answer: "The build is not commercially signed yet. Source and releases are available on GitHub, and the warning usually stops after the first launch.",
      },
      {
        question: "Is Basic mode safe?",
        answer: "It works with GameUserSettings and sg.* parameters, which are closest to normal in-game settings. GSM still creates a backup before applying.",
      },
      {
        question: "What is Engine.ini?",
        answer: "It is an Unreal Engine config file for lower-level CVars. Advanced mode is intended for users who understand the effect of those edits.",
      },
      {
        question: "Can I roll changes back?",
        answer: "Yes. GSM creates a snapshot before apply, and the backups area can restore a previous config version.",
      },
      {
        question: "Why would developers use Epic fetch?",
        answer: "It updates local UE reference data and rebuilds the parameter catalog from Unreal Engine source/config files.",
      },
      {
        question: "Do UE4 and UE5 hints differ?",
        answer: "Yes. The catalog stores UE 4.27–5.8 data, so hints and recommendations can account for engine generation.",
      },
    ],
  },
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
