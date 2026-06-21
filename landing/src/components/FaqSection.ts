import type { LocaleStrings } from "../i18n/types";

export function createFaqSection(t: LocaleStrings): HTMLElement {
  const section = document.createElement("section");
  section.className = "faq section page-wrap reveal-stagger";
  section.id = "faq";
  section.innerHTML = `
    <div class="section-heading section-heading--center">
      <span class="section-heading__eyebrow">${t.faq.eyebrow}</span>
      <h2>${t.faq.title}</h2>
    </div>
    <div class="faq__list">
      ${t.faq.items
        .map(
          (item) => `
            <details class="faq__item">
              <summary>${item.question}</summary>
              <p>${item.answer}</p>
            </details>
          `,
        )
        .join("")}
    </div>
  `;
  return section;
}
