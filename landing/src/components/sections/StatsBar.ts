import type { LocaleStrings } from "../../i18n/types";

export function createStatsBar(t: LocaleStrings): HTMLElement {
  const section = document.createElement("section");
  section.className = "stats-bar page-wrap reveal-stagger";
  section.setAttribute("aria-label", t.hero.badge);
  section.innerHTML = t.stats
    .map(
      (stat) => `
        <div class="stats-bar__item">
          <strong>${stat.value}</strong>
          <span>${stat.label}</span>
        </div>
      `,
    )
    .join("");
  return section;
}
