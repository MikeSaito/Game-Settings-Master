import type { LocaleStrings } from "../i18n/types";

export function buildFaq(t: LocaleStrings): HTMLElement {
  const el = document.createElement("section");
  el.className = "block";
  el.id = "faq";

  el.innerHTML = `
    <div class="wrap">
      <header class="block__head" data-reveal>
        <p class="block__eyebrow">${t.lang === "ru" ? "Вопросы" : "FAQ"}</p>
        <h2 class="block__title">${t.faq.title}</h2>
      </header>
      <div class="ask">
        ${t.faq.items
          .map(
            (item) => `
          <details>
            <summary>${item.question}</summary>
            <div class="ask__body">
              ${item.paragraphs.map((p) => `<p>${p}</p>`).join("")}
            </div>
          </details>`,
          )
          .join("")}
      </div>
    </div>
  `;
  return el;
}
