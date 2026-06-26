import { assetPath } from "../../lib/site";

export interface AppShotOptions {
  src: string;
  alt: string;
  variant?: "hero" | "wide" | "card";
  loading?: "eager" | "lazy";
}

export function createAppShot({ src, alt, variant = "card", loading = "lazy" }: AppShotOptions): HTMLElement {
  const figure = document.createElement("figure");
  figure.className = `app-shot app-shot--${variant}`;
  const img = document.createElement("img");
  img.src = assetPath(src);
  img.alt = alt;
  img.width = 1280;
  img.height = 720;
  img.loading = loading;
  img.decoding = "async";
  figure.appendChild(img);
  return figure;
}
