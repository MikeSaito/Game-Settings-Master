import type { TFunction } from "i18next";

const TOKEN_KEYS = [
  "max",
  "min",
  "quality",
  "scale",
  "distance",
  "shadow",
  "shadows",
  "streaming",
  "resolution",
  "texture",
  "textures",
  "detail",
  "lod",
  "fog",
  "motion",
  "blur",
  "reflection",
  "reflections",
  "ambient",
  "occlusion",
  "postprocess",
  "post",
  "process",
  "light",
  "lighting",
  "lights",
  "view",
  "field",
  "depth",
  "anisotropy",
  "filter",
  "filtering",
  "anti",
  "aliasing",
  "temporal",
  "upscale",
  "upscaling",
  "generation",
  "frame",
  "rate",
  "limit",
  "fullscreen",
  "window",
  "mode",
  "audio",
  "volume",
  "render",
  "rendering",
  "world",
  "foliage",
  "grass",
  "hair",
  "skin",
  "water",
  "sky",
  "cloud",
  "clouds",
  "volumetric",
  "global",
  "illumination",
  "exposure",
  "brightness",
  "contrast",
  "gamma",
  "sharpen",
  "sharpness",
  "distancefield",
  "nanite",
  "lumen",
  "virtual",
  "shadow",
  "cache",
  "pool",
  "size",
  "count",
  "enable",
  "enabled",
  "disable",
  "disabled",
  "use",
  "allow",
] as const;

function splitIdentifierPart(part: string): string[] {
  const normalized = part.replace(/_/g, " ").trim();
  if (!normalized) return [];
  return normalized
    .split(/(?=[A-Z][a-z])|[.\s]+/)
    .map((piece) => piece.trim())
    .filter(Boolean);
}

function humanizeToken(token: string, t: TFunction<"advanced">): string {
  const lower = token.toLowerCase();
  const key = `humanize.${lower}` as const;
  const translated = t(key, { defaultValue: "" });
  if (translated) return translated;
  if (token.length <= 4 && token === token.toUpperCase()) return token;
  return token.charAt(0).toUpperCase() + token.slice(1).toLowerCase();
}

export function humanizeCvarKey(key: string, t: TFunction<"advanced">): string {
  const stripped = key.replace(/^(r\.|sg\.|fx\.|t\.|p\.)/i, "");
  return stripped
    .split(".")
    .flatMap((part) => splitIdentifierPart(part))
    .map((part) => humanizeToken(part, t))
    .join(" · ");
}

export const HUMANIZE_TOKEN_KEYS = TOKEN_KEYS;
