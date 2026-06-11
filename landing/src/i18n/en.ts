import illLibrary from "../svg/ill-library.svg?raw";
import illPresets from "../svg/ill-presets.svg?raw";
import illGpu from "../svg/ill-gpu.svg?raw";
import illEditor from "../svg/ill-editor.svg?raw";
import illBackup from "../svg/ill-backup.svg?raw";
import illCloud from "../svg/ill-cloud.svg?raw";
import illReshade from "../svg/ill-reshade.svg?raw";
import type { LocaleStrings } from "./types";

export const en: LocaleStrings = {
  lang: "en",
  htmlLang: "en",
  siteName: "Game Settings Master",
  meta: {
    title: "Game Settings Master — graphics presets for games",
    description:
      "Presets, manual editor, ReShade and cloud sync for Unreal Engine, Unity and author-curated game breakdowns.",
    keywords:
      "game settings, graphics presets, Unreal Engine, Unity, DLSS, FSR, ReShade, config editor",
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
      "Graphics master for Unreal Engine, Unity and author-curated breakdowns — without digging through configs manually.",
  },
  engineTags: ["UE 4", "UE 5", "Unity", "ReShade", "Author-curated"],
  features: [
    {
      id: "library",
      step: "01",
      title: "Game library",
      text: "Steam and Epic scan, manual add. The app finds your config folder automatically.",
      illustration: illLibrary,
    },
    {
      id: "presets",
      step: "02",
      title: "One-click presets",
      text: "From Ultra Low to Ultra High — with a diff preview before apply. Every change visible in configs.",
      illustration: illPresets,
      reverse: true,
    },
    {
      id: "smart",
      step: "03",
      title: "Smart tuning",
      text: "DLSS, FSR, ray tracing and Frame Generation — safe clamp for your GPU. No pointless options on weak hardware.",
      illustration: illGpu,
    },
    {
      id: "editor",
      step: "04",
      title: "Manual editor",
      text: "Over a hundred parameters with descriptions, categories and dependencies.",
      illustration: illEditor,
      reverse: true,
    },
    {
      id: "backup",
      step: "05",
      title: "Backups",
      text: "Snapshot before every apply. Roll back to the previous state in one click — no fear of breaking your config.",
      illustration: illBackup,
    },
    {
      id: "cloud",
      step: "06",
      title: "Cloud presets",
      text: "Content syncs from the server without an app release. Offline — built-in cache fallback.",
      illustration: illCloud,
      reverse: true,
    },
    {
      id: "reshade",
      step: "07",
      title: "ReShade",
      text: "Install post-processing into the game folder: Performance, Clarity and Cinematic presets, plus author ini packs for specific games. Launch with or without ReShade — proxies are removed automatically when off.",
      illustration: illReshade,
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
