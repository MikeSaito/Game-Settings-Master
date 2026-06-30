import { assetPath } from "../lib/site";

type Cover = { sx: number; sy: number; sw: number; sh: number };

function cover(img: HTMLImageElement, w: number, h: number): Cover {
  const ir = img.naturalWidth / img.naturalHeight;
  const vr = w / h;
  if (ir > vr) {
    const sh = img.naturalHeight;
    const sw = sh * vr;
    return { sx: (img.naturalWidth - sw) * 0.5, sy: 0, sw, sh };
  }
  const sw = img.naturalWidth;
  const sh = sw / vr;
  return { sx: 0, sy: (img.naturalHeight - sh) * 0.38, sw, sh };
}

function paint(ctx: CanvasRenderingContext2D, img: HTMLImageElement, w: number, h: number) {
  const c = cover(img, w, h);
  ctx.clearRect(0, 0, w, h);
  ctx.imageSmoothingEnabled = true;
  ctx.imageSmoothingQuality = "high";
  ctx.drawImage(img, c.sx, c.sy, c.sw, c.sh, 0, 0, w, h);

  const v = ctx.createRadialGradient(w * 0.5, h * 0.35, h * 0.1, w * 0.5, h * 0.5, h * 0.95);
  v.addColorStop(0, "rgba(8,7,6,0)");
  v.addColorStop(1, "rgba(8,7,6,0.28)");
  ctx.fillStyle = v;
  ctx.fillRect(0, 0, w, h);
}

export function mountScene(): () => void {
  const root = document.createElement("div");
  root.className = "scene";
  root.setAttribute("aria-hidden", "true");
  root.innerHTML = `<canvas></canvas><div class="scene__grain"></div><div class="scene__fade"></div>`;
  document.body.prepend(root);

  const canvas = root.querySelector("canvas")!;
  const ctx = canvas.getContext("2d");
  if (!ctx) return () => root.remove();

  const img = new Image();
  img.src = assetPath("hero/quality-ultra.webp");
  img.decoding = "async";

  let raf = 0;
  const resize = () => {
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    const w = window.innerWidth;
    const h = window.innerHeight;
    canvas.width = Math.floor(w * dpr);
    canvas.height = Math.floor(h * dpr);
    canvas.style.width = `${w}px`;
    canvas.style.height = `${h}px`;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    if (img.complete && img.naturalWidth) paint(ctx, img, w, h);
  };

  const onLoad = () => {
    cancelAnimationFrame(raf);
    raf = requestAnimationFrame(resize);
  };

  img.addEventListener("load", onLoad);
  window.addEventListener("resize", onLoad, { passive: true });
  if (img.complete) onLoad();

  return () => {
    cancelAnimationFrame(raf);
    window.removeEventListener("resize", onLoad);
    root.remove();
  };
}
