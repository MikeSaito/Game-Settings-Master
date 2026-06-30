export interface FeatureStrings {
  id: string;
  step: string;
  title: string;
  text: string;
  shot: string;
  reverse?: boolean;
}

export interface BasicAdvancedColumnStrings {
  label: string;
  title: string;
  text: string;
  bullets: string[];
}

export interface BasicAdvancedStrings {
  title: string;
  text: string;
  basic: BasicAdvancedColumnStrings;
  advanced: BasicAdvancedColumnStrings;
}

export interface FaqStrings {
  question: string;
  paragraphs: string[];
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
  nav: {
    features: string;
    modes: string;
    faq: string;
    download: string;
    aria: string;
  };
  hero: {
    kicker: string;
    title: string;
    titleAccent: string;
    subtitle: string;
    shotAlt: string;
    sceneCredit: string;
  };
  engineTags: string[];
  basicAdvanced: BasicAdvancedStrings;
  features: FeatureStrings[];
  faq: {
    title: string;
    items: FaqStrings[];
  };
  download: {
    title: string;
    subtitle: string;
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
  footer: {
    version: (v: string) => string;
    donateLink: string;
    telegramLink: string;
  };
  donate: {
    title: string;
    text: string;
    button: string;
  };
  localeSwitch: {
    label: string;
    ru: string;
    en: string;
  };
}
