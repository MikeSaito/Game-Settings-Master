export interface FeatureStrings {
  id: string;
  step: string;
  title: string;
  text: string;
  illustration: string;
  reverse?: boolean;
}

export interface StatStrings {
  value: string;
  label: string;
}

export interface BasicAdvancedColumnStrings {
  label: string;
  title: string;
  text: string;
  bullets: string[];
}

export interface BasicAdvancedStrings {
  eyebrow: string;
  title: string;
  text: string;
  basic: BasicAdvancedColumnStrings;
  advanced: BasicAdvancedColumnStrings;
}

export interface StepStrings {
  step: string;
  title: string;
  text: string;
}

export interface HighlightStrings {
  eyebrow: string;
  title: string;
  text: string;
  bullets: string[];
}

export interface FaqStrings {
  question: string;
  answer: string;
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
    howItWorks: string;
    catalog: string;
    faq: string;
    download: string;
    aria: string;
  };
  hero: {
    badge: string;
    title: string;
    titleAccent: string;
    subtitle: string;
  };
  engineTags: string[];
  stats: StatStrings[];
  basicAdvanced: BasicAdvancedStrings;
  features: FeatureStrings[];
  howItWorks: {
    eyebrow: string;
    title: string;
    steps: StepStrings[];
  };
  catalogHighlight: HighlightStrings;
  gpu: HighlightStrings;
  faq: {
    eyebrow: string;
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
