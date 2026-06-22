import type { HighlightStrings } from "../../i18n/types";

export function createCatalogHighlightSection(catalog: HighlightStrings): HTMLElement {
  const section = document.createElement("section");
  section.className = "catalog-highlight section page-wrap reveal-stagger";
  section.id = "catalog";
  section.innerHTML = `
    <div class="highlight-card">
      <div class="highlight-card__content">
        <span class="section-heading__eyebrow">${catalog.eyebrow}</span>
        <h2>${catalog.title}</h2>
        <p>${catalog.text}</p>
        <ul class="check-list">
          ${catalog.bullets.map((bullet) => `<li>${bullet}</li>`).join("")}
        </ul>
      </div>
      <div class="catalog-mini" aria-hidden="true">
        <div class="catalog-mini__toolbar">
          <span>UE 5.1</span>
          <span>Recommended</span>
        </div>
        <div class="catalog-mini__row is-strong">
          <span>r.Nanite.MaxPixelsPerEdge</span>
          <b>Tier A</b>
        </div>
        <div class="catalog-mini__row">
          <span>sg.TextureQuality</span>
          <b>sg.*</b>
        </div>
        <div class="catalog-mini__row">
          <span>r.TSR.History.ScreenPercentage</span>
          <b>UE 5</b>
        </div>
      </div>
    </div>
  `;
  return section;
}
