import { en } from "./en";
import { ru } from "./ru";
import type { LocaleStrings } from "./types";

const locales: Record<string, LocaleStrings> = { ru, en };

export function getLocale(): LocaleStrings {
  const lang = document.documentElement.lang;
  if (lang && locales[lang]) {
    return locales[lang];
  }

  if (/\/en(?:\/|\.html)?$/.test(window.location.pathname)) {
    return en;
  }

  return ru;
}

export type { LocaleStrings, FeatureStrings } from "./types";
