export interface FaqStrings {
  question: string;
  paragraphs: string[];
}

export interface MockGame {
  id: string;
  name: string;
  source: "steam" | "epic" | "manual";
  engine: string;
  version: string;
  ok: boolean;
  fav?: boolean;
  path: string;
  config: string;
  hue: number;
}

export interface MockParam {
  key: string;
  title: string;
  value: string;
  kind: "slider" | "toggle" | "select" | "cvar";
  min?: number;
  max?: number;
  options?: string[];
  tier?: string;
  warning?: string;
  hint?: string;
  /** Цель и порог прогресса сцены modes: контрол приводится к demoTo после demoAt. */
  demoTo?: string;
  demoAt?: number;
}

export interface MockBackup {
  id: string;
  label: string;
  files: string[];
}

export interface StorySceneText {
  kicker: string;
  title: string;
  text: string;
}

export interface LocaleStrings {
  lang: "ru" | "en";
  htmlLang: string;
  siteName: string;
  meta: {
    title: string;
    description: string;
    keywords: string;
    ogLocale: string;
  };
  chrome: {
    aria: string;
    langLabel: string;
    ru: string;
    en: string;
    download: string;
  };
  story: {
    scrollHint: string;
    hero: StorySceneText;
    scan: StorySceneText & {
      scanning: string;
      counter: (found: number) => string;
    };
    backup: StorySceneText & {
      badge: string;
      now: string;
    };
    modes: StorySceneText & {
      fps: string;
      tooltip: string;
    };
    final: StorySceneText & {
      tagline: string;
      badges: string[];
    };
    flow: {
      faqTitle: string;
    };
  };
  mock: {
    windowTitle: string;
    navLibrary: string;
    navEditor: string;
    navSettings: string;
    search: string;
    scan: string;
    add: string;
    libTitle: string;
    libSubtitle: string;
    badgeTotal: (n: number) => string;
    badgeConfig: (n: number) => string;
    badgeUe: (n: number) => string;
    badgeCover: (n: number) => string;
    configOk: string;
    configMissing: string;
    select: string;
    cover: string;
    sourceSteam: string;
    sourceEpic: string;
    sourceManual: string;
    ctxPlay: string;
    ctxConfig: string;
    tabs: {
      basic: string;
      advanced: string;
      presets: string;
      backups: string;
    };
    basicHint: string;
    advancedHint: string;
    backupsHint: string;
    basicSafe: string;
    advancedWarn: string;
    paramsForEngine: string;
    known: string;
    sgLimits: string;
    engineShort: string;
    changes: (n: number) => string;
    discard: string;
    applyBasic: string;
    applyAdvanced: string;
    inIni: string;
    removeFromIni: string;
    detail: {
      title: string;
      key: string;
      current: [string, string];
      type: [string, string];
      range: [string, string];
      tier: string;
      desc: string;
      compat: [string, string];
    };
    backupsCount: (n: number) => string;
    howTitle: string;
    howBody: string;
    listTitle: string;
    listDesc: string;
    restore: string;
    gpu: string;
    newBackupId: string;
    games: MockGame[];
    basicParams: MockParam[];
    advancedParams: MockParam[];
    backups: MockBackup[];
  };
  faq: FaqStrings[];
  download: {
    button: string;
    githubButton: string;
    smartScreen: {
      title: string;
      intro: string;
      step1: string;
      step2: string;
      note: string;
      confirm: string;
      cancel: string;
    };
  };
  donate: {
    title: string;
    text: string;
    button: string;
  };
  footer: {
    version: (v: string) => string;
    donateLink: string;
    telegramLink: string;
  };
}
