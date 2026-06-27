import { en } from "./en";
import { ru } from "./ru";
import type { LocaleStrings } from "./types";
import { joinBase } from "../lib/site";

function isEnglishPath(pathname: string): boolean {
  return /\/en(?:\/|\.html)?$/.test(pathname);
}

function detectSystemLanguage(): "ru" | "en" {
  if (typeof navigator === "undefined") return "en";

  const tags = navigator.languages?.length ? navigator.languages : [navigator.language];
  for (const tag of tags) {
    if (!tag) continue;
    const code = tag.toLowerCase().split("-")[0];
    if (code === "ru") return "ru";
    if (code === "en") return "en";
  }

  return "en";
}

function isLikelySearchBot(): boolean {
  if (typeof navigator === "undefined") return false;
  const ua = navigator.userAgent.toLowerCase();
  return /googlebot|bingbot|yandex|duckduckbot|baiduspider|slurp|facebookexternalhit|twitterbot|linkedinbot|applebot|petalbot|semrushbot|ahrefsbot/.test(
    ua,
  );
}

/** Locale follows the URL so crawlers get stable RU/EN pages for indexing. */
export function getLocale(): LocaleStrings {
  if (isEnglishPath(window.location.pathname)) {
    return en;
  }
  return ru;
}

/** Send non-Russian visitors from `/` to `/en/` without affecting search bots. */
export function maybeRedirectToEnglishHome(): void {
  if (typeof window === "undefined") return;
  if (isLikelySearchBot()) return;
  if (isEnglishPath(window.location.pathname)) return;
  if (detectSystemLanguage() !== "en") return;

  const target = joinBase("en/");
  const targetPath = new URL(target, window.location.origin).pathname;
  if (window.location.pathname !== targetPath) {
    window.location.replace(target);
  }
}

export type { LocaleStrings, FeatureStrings } from "./types";
