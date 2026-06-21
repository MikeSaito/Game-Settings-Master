/** Базовый путь GitHub Pages (`/repo-name/`). В dev — `/`. */
export const basePath = import.meta.env.BASE_URL;

export const siteUrl = (
  import.meta.env.VITE_SITE_URL ?? "https://mikesaito.github.io/Game-Settings-Master"
).replace(/\/$/, "");

export const githubRepo =
  import.meta.env.VITE_GITHUB_REPO ?? "MikeSaito/Game-Settings-Master";

export const githubUrl = `https://github.com/${githubRepo}`;

/** Прямая ссылка на установщик в GitHub Releases (latest). */
const DEFAULT_APP_VERSION = "1.0.2-a";

function resolveAppVersion(): string {
  const raw = import.meta.env.VITE_APP_VERSION?.trim();
  return raw || DEFAULT_APP_VERSION;
}

export const APP_VERSION = resolveAppVersion();

export const downloadUrl = `https://github.com/${githubRepo}/releases/latest/download/Game-Settings-Master_${APP_VERSION}_x64-setup.exe`;

export const donateUrl = "https://www.donationalerts.com/r/mike_saito";

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
