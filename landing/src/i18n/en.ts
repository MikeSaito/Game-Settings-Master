import type { LocaleStrings } from "./types";

export const en: LocaleStrings = {
  lang: "en",
  htmlLang: "en",
  siteName: "Game Settings Master",
  meta: {
    title: "Game Settings Master – Unreal Engine config editor",
    description:
      "Editor for GameUserSettings.ini and Engine.ini in Unreal Engine games. Backups, UE version hints, DLSS/FSR filters for your GPU.",
    keywords:
      "game settings, Unreal Engine, UE4, UE5, DLSS, FSR, config editor, ini",
    ogLocale: "en_US",
  },
  nav: {
    features: "Features",
    modes: "Modes",
    faq: "FAQ",
    download: "Download",
    aria: "Navigation",
  },
  hero: {
    kicker: "Ini editor for Unreal Engine",
    title: "Game settings",
    titleAccent: "under control",
    subtitle:
      "GameUserSettings and Engine.ini in one window. Backup before apply, UE version hints, DLSS and FSR filters matched to your GPU.",
    shotAlt: "Game Settings Master editor – Basic mode",
    sceneCredit:
      "Background: a concept render from a visual reference series — not in-engine footage. © project author.",
  },
  engineTags: [
    "UE 4.27",
    "UE 5.8",
    "GameUserSettings.ini",
    "Engine.ini",
    "DLSS",
    "FSR",
    "Nanite",
    "Lumen",
    "Steam",
    "Epic",
  ],
  basicAdvanced: {
    title: "Basic and Advanced",
    text: "Switch between in-game settings and engine CVars – the UI adapts to the task.",
    basic: {
      label: "Basic",
      title: "GameUserSettings",
      text: "What you usually find in a game's menu: quality, resolution, window mode, VSync.",
      bullets: ["sg.*", "resolution and window mode", "VSync, FPS limit"],
    },
    advanced: {
      label: "Advanced",
      title: "Engine.ini / CVars",
      text: "Low-level engine parameters with warnings and a config snapshot before writing.",
      bullets: ["r.* and other CVars", "UE version hints", "rollback via backup"],
    },
  },
  features: [
    {
      id: "library",
      step: "01",
      title: "Game library",
      text: "Steam and Epic scan, manual add. Finds config folders and shows game context – engine, paths, cover art.",
      shot: "screenshots/en/library.png",
    },
    {
      id: "editor",
      step: "02",
      title: "Basic / Advanced editor",
      text: "Clear sliders and toggles for GameUserSettings. Advanced mode for CVars with tier hints, warnings, and backup.",
      shot: "screenshots/en/editor-basic.png",
      reverse: true,
    },
    {
      id: "backup",
      step: "03",
      title: "Backups",
      text: "Config snapshot before every apply. Experiment with ini and roll back to a previous version in one click.",
      shot: "screenshots/en/backups.png",
      reverse: true,
    },
  ],
  faq: {
    title: "Common questions",
    items: [
      {
        question: "Why does SmartScreen warn on launch?",
        paragraphs: [
          "Windows Defender SmartScreen checks whether a file is signed by a known publisher. Game Settings Master is distributed as a free build without a commercial code-signing certificate – common for indie software, not a sign of malware.",
          'On first launch, Windows may show a blue screen saying it protected your PC. Click "More info", then "Run anyway". The app source is open on GitHub – you can verify what goes into each release.',
          "After a successful first run, Windows often remembers the file and stops warning. Part of project support goes toward a publisher certificate – that will remove the warning for new users too.",
        ],
      },
      {
        question: "Is Basic mode safe?",
        paragraphs: [
          "Basic edits GameUserSettings.ini – the same file in-game settings menus use. sg.* parameters (Scalability Groups) control texture, shadow, and effects quality – the presets players usually change with in-menu sliders.",
          "Basic does not touch Engine.ini or low-level CVars. The risk of breaking a game is comparable to launcher or in-game menu tweaks – you change the same values, with clearer labels and context.",
          "A config backup is created before every apply. If the result is not what you wanted, roll back from the backups tab without hunting through the Saved folder manually.",
        ],
      },
      {
        question: "What is Engine.ini?",
        paragraphs: [
          "Engine.ini is an Unreal Engine config file under Saved/Config. It stores CVars (console variables): r.*, render limits, feature toggles, and other low-level settings not exposed in the game menu.",
          "Games may also use Scalability.ini and Game.ini – Advanced shows parameters from all of these files. Some values overlap sg.* from GameUserSettings at the engine level – the app warns about those conflicts.",
          "Advanced edits are more powerful but need care: wrong values can hurt stability, FPS, or visuals. Use parameter hints and always back up before experimenting.",
        ],
      },
      {
        question: "Can I roll back changes?",
        paragraphs: [
          "Yes. Before writing to ini, the app saves a timestamped snapshot of your current configs – GameUserSettings.ini, Engine.ini, and related files.",
          "Open the backups tab in the editor, pick a restore point, and restore. Configs return to the state at snapshot time without manual copying from Saved.",
          "Apply settings, launch the game, and roll back if FPS or stability got worse. Each apply creates a new backup – older restore points are not overwritten automatically.",
        ],
      },
      {
        question: "Do UE4 and UE5 hints differ?",
        paragraphs: [
          "Unreal Engine 4 and 5 differ in CVars, defaults, and how some parameters behave. The app detects the game's engine version and shows hints relevant to that version.",
          "Options like Nanite, Lumen, or settings introduced in UE 5.4+ will not be suggested for a UE 4.27 game. Tier hints (priority and risk) also depend on version context – what is safe in one UE branch may be risky in another.",
          "If the version cannot be detected, the editor still lists parameters found in ini and lets you edit them – without version-specific guidance.",
        ],
      },
    ],
  },
  download: {
    title: "Download Game Settings Master",
    subtitle: "Windows · free · unsigned build",
    button: "Download for Windows",
    githubButton: "Source on GitHub",
    smartScreen: {
      title: "First launch on Windows",
      intro:
        "The app is unsigned – SmartScreen may show a warning. Normal for indie software.",
      step1: 'Click "More info"',
      step2: 'Then "Run anyway"',
      note: "After the first run, Windows usually stops asking.",
      confirm: "Got it, download",
      cancel: "Cancel",
    },
  },
  donate: {
    title: "Support development",
    text: "Toward a Windows code signing certificate and future updates.",
    button: "Donate",
  },
  footer: {
    version: (v: string) => `Game Settings Master v${v}`,
    donateLink: "Support the project",
    telegramLink: "Telegram",
  },
  localeSwitch: {
    label: "Language",
    ru: "RU",
    en: "EN",
  },
};
