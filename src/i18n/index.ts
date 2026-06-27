import i18n, { type Resource, type ResourceLanguage } from "i18next";
import { initReactI18next } from "react-i18next";
import LanguageDetector from "i18next-browser-languagedetector";
import { loadAppSettings } from "@/lib/settings/appSettings";
import {
  detectSystemLanguage,
  languageFromTag,
  type AppLanguage,
} from "./detectLanguage";

export const LANGUAGE_STORAGE_KEY = "uesm:language";
export type { AppLanguage } from "./detectLanguage";
export const SUPPORTED_LANGUAGES: AppLanguage[] = ["ru", "en"];
export { detectSystemLanguage, languageFromTag } from "./detectLanguage";

// Each namespace lives in its own JSON file under locales/<lang>/<namespace>.json.
// Loading via glob lets us add namespaces without editing this file.
const ruModules = import.meta.glob<{ default: ResourceLanguage }>(
  "./locales/ru/*.json",
  { eager: true },
);
const enModules = import.meta.glob<{ default: ResourceLanguage }>(
  "./locales/en/*.json",
  { eager: true },
);

function buildLanguageResource(
  modules: Record<string, { default: ResourceLanguage }>,
): ResourceLanguage {
  const out: ResourceLanguage = {};
  for (const [path, mod] of Object.entries(modules)) {
    const ns = path.split("/").pop()?.replace(/\.json$/, "");
    if (ns) out[ns] = mod.default;
  }
  return out;
}

const resources: Resource = {
  ru: buildLanguageResource(ruModules),
  en: buildLanguageResource(enModules),
};

function initialLanguage(): AppLanguage {
  try {
    if (typeof localStorage !== "undefined") {
      return loadAppSettings().language;
    }
  } catch {
    /* ignore */
  }
  return detectSystemLanguage();
}

void i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources,
    lng: initialLanguage(),
    fallbackLng: "en",
    supportedLngs: SUPPORTED_LANGUAGES,
    nonExplicitSupportedLngs: true,
    load: "languageOnly",
    defaultNS: "common",
    interpolation: { escapeValue: false },
    detection: {
      order: ["localStorage", "navigator"],
      lookupLocalStorage: LANGUAGE_STORAGE_KEY,
      caches: ["localStorage"],
      convertDetectedLanguage: (lng) => languageFromTag(lng),
    },
  });

export function currentLanguage(): AppLanguage {
  const lng = (i18n.resolvedLanguage ?? i18n.language)?.slice(0, 2);
  if (lng === "en" || lng === "ru") return lng;
  return detectSystemLanguage();
}

export default i18n;
