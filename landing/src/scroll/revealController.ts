function lerp(a: number, b: number, t: number): number {
  return a + (b - a) * t;
}

let lastProgress = -1;

export function applyReveal(progress: number): void {
  // Пропускаем микро-изменения — меньше лишних перерисовок
  if (Math.abs(progress - lastProgress) < 0.004) return;
  lastProgress = progress;

  const root = document.documentElement;
  const scale = lerp(1.07, 1, progress);

  root.style.setProperty("--reveal", String(progress));
  root.style.setProperty("--scene-scale", String(scale));

  const coarse = document.getElementById("layer-coarse");
  const medium = document.getElementById("layer-medium");
  const fine = document.getElementById("layer-fine");

  if (coarse) coarse.setAttribute("opacity", String(lerp(1, 0.15, progress)));
  if (medium) medium.setAttribute("opacity", String(lerp(0.12, 0.75, progress)));
  if (fine) fine.setAttribute("opacity", String(lerp(0, 1, progress)));
}

export function mountScene(svgMarkup: string): void {
  const root = document.getElementById("scene-root");
  if (!root) return;
  root.innerHTML = svgMarkup;
}
