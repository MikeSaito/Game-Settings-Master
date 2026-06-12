/** Базовый путь GitHub Pages (`/repo-name/`). В dev — `/`. */
export const basePath = import.meta.env.BASE_URL;

export const siteUrl = (
  import.meta.env.VITE_SITE_URL ?? "https://example.github.io/game-settings-master"
).replace(/\/$/, "");

export const githubRepo =
  import.meta.env.VITE_GITHUB_REPO ?? "Mike/game-settings-master";

export const githubUrl = `https://github.com/${githubRepo}`;

/** Прямая ссылка на установщик в GitHub Releases (latest). */
export const APP_VERSION = import.meta.env.VITE_APP_VERSION ?? "0.3.1";

export const downloadUrl =
  import.meta.env.VITE_DOWNLOAD_URL ??
  `https://github.com/${githubRepo}/releases/latest/download/Game-Settings-Master_${APP_VERSION}_x64-setup.exe`;

export const donateUrl = "https://dalink.to/mike_saito";

export function joinBase(...segments: string[]): string {
  const base = basePath.endsWith("/") ? basePath : `${basePath}/`;
  const tail = segments
    .filter(Boolean)
    .join("/")
    .replace(/^\/+/, "");
  return `${base}${tail}`;
}

/** Путь к статике (logo, favicon). */
export function assetPath(path: string): string {
  const clean = path.startsWith("/") ? path.slice(1) : path;
  return joinBase(clean);
}

/** Домашняя страница локали: `` → RU, `en/` → EN. */
export function localeHome(lang: "ru" | "en"): string {
  return lang === "en" ? joinBase("en/") : joinBase("");
}

export function absoluteUrl(path: string): string {
  const normalized = path.startsWith("/") ? path : `/${path}`;
  if (normalized.startsWith("http")) return normalized;
  return `${siteUrl}${normalized}`;
}
