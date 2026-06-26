export function localeShot(lang: "ru" | "en", file: string): string {
  return `screenshots/${lang}/${file}`;
}
