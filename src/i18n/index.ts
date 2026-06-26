import i18n, { type Resource, type ResourceLanguage } from "i18next";
import { initReactI18next } from "react-i18next";
import LanguageDetector from "i18next-browser-languagedetector";

export const LANGUAGE_STORAGE_KEY = "uesm:language";
export type AppLanguage = "ru" | "en";
export const SUPPORTED_LANGUAGES: AppLanguage[] = ["ru", "en"];

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

void i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources,
    fallbackLng: "ru",
    supportedLngs: SUPPORTED_LANGUAGES,
    nonExplicitSupportedLngs: true,
    load: "languageOnly",
    defaultNS: "common",
    interpolation: { escapeValue: false },
    detection: {
      order: ["localStorage", "navigator"],
      lookupLocalStorage: LANGUAGE_STORAGE_KEY,
      caches: ["localStorage"],
    },
  });

export function currentLanguage(): AppLanguage {
  const lng = (i18n.resolvedLanguage ?? i18n.language ?? "ru").slice(0, 2);
  return lng === "en" ? "en" : "ru";
}

export default i18n;
