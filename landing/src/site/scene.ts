import { assetPath } from "../lib/site";

type HeroTier = "high" | "ultra" | "low";

/** Подбирает файл с достаточным разрешением без лишнего upscale. */
function pickHeroTier(widthPx: number): HeroTier {
  if (widthPx <= 520) return "high";
  if (widthPx <= 1024) return "ultra";
  return "low";
}

function paintNoise(tile: HTMLCanvasElement): void {
  const ctx = tile.getContext("2d");
  if (!ctx) return;
  const { width, height } = tile;
  const data = ctx.createImageData(width, height);
  const buf = data.data;
  for (let i = 0; i < buf.length; i += 4) {
    const v = (Math.random() * 255) | 0;
    buf[i] = v;
    buf[i + 1] = v;
    buf[i + 2] = v;
    buf[i + 3] = 255;
  }
  ctx.putImageData(data, 0, 0);
}

export function mountScene(host: HTMLElement): { root: HTMLElement; cleanup: () => void } {
  const root = document.createElement("div");
  root.className = "scene";
  root.setAttribute("aria-hidden", "true");

  const bg = document.createElement("img");
  bg.className = "scene__bg";
  bg.alt = "";
  bg.decoding = "async";
  bg.setAttribute("fetchpriority", "high");

  const grain = document.createElement("div");
  grain.className = "scene__grain";
  const noise = document.createElement("canvas");
  noise.width = 128;
  noise.height = 128;
  paintNoise(noise);
  grain.style.backgroundImage = `url(${noise.toDataURL("image/png")})`;

  const fade = document.createElement("div");
  fade.className = "scene__fade";

  root.append(bg, grain, fade);
  host.prepend(root);

  let tier: HeroTier | null = null;
  let raf = 0;

  const syncSrc = () => {
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    const next = pickHeroTier(Math.ceil(window.innerWidth * dpr));
    if (next === tier) return;
    tier = next;
    bg.src = assetPath(`hero/quality-${next}.webp`);
  };

  const onResize = () => {
    cancelAnimationFrame(raf);
    raf = requestAnimationFrame(syncSrc);
  };

  window.addEventListener("resize", onResize, { passive: true });
  syncSrc();

  const cleanup = () => {
    cancelAnimationFrame(raf);
    window.removeEventListener("resize", onResize);
    root.remove();
  };

  return { root, cleanup };
}
