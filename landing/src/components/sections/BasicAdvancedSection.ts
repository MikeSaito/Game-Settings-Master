import type { BasicAdvancedColumnStrings, LocaleStrings } from "../../i18n/types";

function renderColumn(column: BasicAdvancedColumnStrings, tone: "safe" | "expert"): string {
  return `
    <article class="mode-card mode-card--${tone}">
      <div class="mode-card__label">${column.label}</div>
      <h3>${column.title}</h3>
      <p>${column.text}</p>
      <ul>
        ${column.bullets.map((bullet) => `<li>${bullet}</li>`).join("")}
      </ul>
    </article>
  `;
}

export function createBasicAdvancedSection(t: LocaleStrings): HTMLElement {
  const section = document.createElement("section");
  section.className = "basic-advanced section page-wrap reveal-stagger";
  section.id = "basic-advanced";
  section.innerHTML = `
    <div class="section-heading">
      <span class="section-heading__eyebrow">${t.basicAdvanced.eyebrow}</span>
      <h2>${t.basicAdvanced.title}</h2>
      <p>${t.basicAdvanced.text}</p>
    </div>
    <div class="compare-grid">
      ${renderColumn(t.basicAdvanced.basic, "safe")}
      ${renderColumn(t.basicAdvanced.advanced, "expert")}
    </div>
  `;
  return section;
}
