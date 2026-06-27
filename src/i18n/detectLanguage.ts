export type AppLanguage = "ru" | "en";

/** Map a BCP-47 tag (e.g. `de-DE`, `en-US`) to a supported app language. */
export function languageFromTag(tag: string): AppLanguage {
  const code = tag.trim().toLowerCase().split("-")[0];
  return code === "ru" ? "ru" : "en";
}

/** Prefer Russian only when the OS/browser lists a Russian locale; otherwise English. */
export function detectSystemLanguage(): AppLanguage {
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
