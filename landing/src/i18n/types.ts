export interface FeatureStrings {
  id: string;
  step: string;
  title: string;
  text: string;
  illustration: string;
  reverse?: boolean;
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
  features: FeatureStrings[];
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
  };
  localeSwitch: {
    label: string;
    ru: string;
    en: string;
  };
}
