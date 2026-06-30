import type { LocaleStrings } from "../i18n/types";

function measure(tags: string[]): number {
  const probe = document.createElement("div");
  probe.className = "ticker__set";
  probe.style.cssText = "position:absolute;visibility:hidden;left:-9999px";
  for (const tag of tags) {
    const s = document.createElement("span");
    s.textContent = tag;
    probe.append(s);
  }
  document.body.append(probe);
  const w = probe.offsetWidth;
  probe.remove();
  return w;
}

export function buildTicker(t: LocaleStrings): HTMLElement {
  const el = document.createElement("div");
  el.className = "ticker";
  el.setAttribute("aria-hidden", "true");

  const track = document.createElement("div");
  track.className = "ticker__track";

  const sync = () => {
    const w = measure(t.engineTags);
    const reps = Math.max(2, Math.ceil((window.innerWidth * 1.3) / Math.max(w, 1)));
    const tags = Array.from({ length: reps }, () => t.engineTags).flat();
    const mk = () => {
      const set = document.createElement("div");
      set.className = "ticker__set";
      for (const tag of tags) {
        const s = document.createElement("span");
        s.textContent = tag;
        set.append(s);
      }
      return set;
    };
    const a = mk();
    const b = mk();
    b.setAttribute("aria-hidden", "true");
    track.replaceChildren(a, b);
    track.style.setProperty("--dur", `${Math.max(26, Math.min(55, tags.length * 2.5))}s`);
  };

  sync();
  window.addEventListener("resize", sync, { passive: true });
  el.append(track);
  return el;
}
