import { assetPath } from "../lib/site";

export function makeShot(src: string, alt: string): HTMLElement {
  const fig = document.createElement("figure");
  fig.className = "shot";
  const img = document.createElement("img");
  img.src = assetPath(src);
  img.alt = alt;
  img.loading = "lazy";
  img.decoding = "async";
  fig.append(img);
  return fig;
}
