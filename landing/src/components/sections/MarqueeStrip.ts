import type { LocaleStrings } from "../../i18n/types";

function buildGroup(tags: string[], ariaHidden = false): HTMLElement {
  const group = document.createElement("div");
  group.className = "marquee__group";
  if (ariaHidden) group.setAttribute("aria-hidden", "true");

  for (const tag of tags) {
    const span = document.createElement("span");
    span.textContent = tag;
    group.appendChild(span);
  }

  return group;
}

function measureTagSetWidth(tags: string[]): number {
  const probe = buildGroup(tags);
  probe.className = "marquee__group marquee__group--probe";
  document.body.appendChild(probe);
  const width = probe.offsetWidth;
  probe.remove();
  return width;
}

function syncMarqueeTrack(track: HTMLElement, tags: string[]): void {
  const singleWidth = measureTagSetWidth(tags);
  const viewport = window.innerWidth;
  const repeats = Math.max(2, Math.ceil((viewport * 1.2) / Math.max(singleWidth, 1)));
  const expanded = Array.from({ length: repeats }, () => tags).flat();

  track.replaceChildren(buildGroup(expanded), buildGroup(expanded, true));

  const duration = Math.max(28, Math.min(60, expanded.length * 2.8));
  track.style.setProperty("--marquee-duration", `${duration}s`);
}

export function createMarqueeStrip(t: LocaleStrings): HTMLElement {
  const strip = document.createElement("div");
  strip.className = "marquee-strip";
  strip.setAttribute("aria-hidden", "true");

  const marquee = document.createElement("div");
  marquee.className = "marquee";

  const track = document.createElement("div");
  track.className = "marquee__track";

  marquee.appendChild(track);
  strip.appendChild(marquee);

  const sync = () => syncMarqueeTrack(track, t.engineTags);
  sync();
  window.addEventListener("resize", sync, { passive: true });

  return strip;
}
