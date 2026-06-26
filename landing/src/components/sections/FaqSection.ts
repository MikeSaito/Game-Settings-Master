import type { LocaleStrings } from "../../i18n/types";

export function createFaqSection(t: LocaleStrings): HTMLElement {
  const section = document.createElement("section");
  section.className = "faq section page-wrap reveal-stagger";
  section.id = "faq";
  section.innerHTML = `
    <div class="faq__panel">
      <div class="section-heading">
        <h2>${t.faq.title}</h2>
      </div>
      <div class="faq__list">
        ${t.faq.items
          .map(
            (item) => `
              <details class="faq__item">
                <summary>${item.question}</summary>
                <div class="faq__answer">
                  ${item.paragraphs.map((p) => `<p>${p}</p>`).join("")}
                </div>
              </details>
            `,
          )
          .join("")}
      </div>
    </div>
  `;
  return section;
}
