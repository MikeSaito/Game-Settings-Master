import type { LocaleStrings } from "../../i18n/types";

export function createHowItWorksSection(t: LocaleStrings): HTMLElement {
  const section = document.createElement("section");
  section.className = "how-it-works section page-wrap reveal-stagger";
  section.id = "how-it-works";
  section.innerHTML = `
    <div class="section-heading section-heading--center">
      <span class="section-heading__eyebrow">${t.howItWorks.eyebrow}</span>
      <h2>${t.howItWorks.title}</h2>
    </div>
    <div class="steps-grid">
      ${t.howItWorks.steps
        .map(
          (step) => `
            <article class="step-card">
              <span>${step.step}</span>
              <h3>${step.title}</h3>
              <p>${step.text}</p>
            </article>
          `,
        )
        .join("")}
    </div>
  `;
  return section;
}
