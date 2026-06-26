import type { BasicAdvancedColumnStrings, LocaleStrings } from "../../i18n/types";
import { localeShot } from "../../lib/shots";
import { createAppShot } from "../shared/AppShot";

function renderColumn(column: BasicAdvancedColumnStrings, tone: "safe" | "expert"): string {
  return `
    <article class="mode-card mode-card--${tone}">
      <div class="mode-card__label">${column.label}</div>
      <h3>${column.title}</h3>
      <p>${column.text}</p>
      <ul>${column.bullets.map((b) => `<li>${b}</li>`).join("")}</ul>
    </article>
  `;
}

export function createShowcaseSection(t: LocaleStrings): HTMLElement {
  const section = document.createElement("section");
  section.className = "showcase section page-wrap reveal-stagger";
  section.id = "modes";
  section.innerHTML = `
    <div class="section-heading">
      <h2>${t.basicAdvanced.title}</h2>
      <p>${t.basicAdvanced.text}</p>
    </div>
    <div class="showcase__demo" data-showcase>
      <div class="showcase__tabs" role="tablist">
        <button type="button" class="showcase__tab is-active" role="tab" aria-selected="true" data-mode="basic">${t.basicAdvanced.basic.label}</button>
        <button type="button" class="showcase__tab" role="tab" aria-selected="false" data-mode="advanced">${t.basicAdvanced.advanced.label}</button>
      </div>
      <div class="showcase__shot" data-showcase-shot></div>
    </div>
    <div class="compare-grid">
      ${renderColumn(t.basicAdvanced.basic, "safe")}
      ${renderColumn(t.basicAdvanced.advanced, "expert")}
    </div>
  `;

  const shotWrap = section.querySelector<HTMLElement>("[data-showcase-shot]");
  const tabs = section.querySelectorAll<HTMLButtonElement>(".showcase__tab");

  const shots = {
    basic: createAppShot({
      src: localeShot(t.lang, "editor-basic.png"),
      alt: t.basicAdvanced.basic.title,
      variant: "wide",
    }),
    advanced: createAppShot({
      src: localeShot(t.lang, "editor-advanced.png"),
      alt: t.basicAdvanced.advanced.title,
      variant: "wide",
    }),
  };

  const setMode = (mode: "basic" | "advanced") => {
    if (!shotWrap) return;
    shotWrap.replaceChildren(shots[mode]);
    tabs.forEach((tab) => {
      const active = tab.dataset.mode === mode;
      tab.classList.toggle("is-active", active);
      tab.setAttribute("aria-selected", String(active));
    });
  };

  setMode("basic");
  tabs.forEach((tab) => {
    tab.addEventListener("click", () => {
      const mode = tab.dataset.mode as "basic" | "advanced";
      if (mode) setMode(mode);
    });
  });

  return section;
}
