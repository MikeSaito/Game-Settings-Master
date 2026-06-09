import type { LocaleStrings } from "../i18n/types";
import { createCtaButtons } from "./CtaButtons";

export function createHeroSection(t: LocaleStrings): HTMLElement {
  const section = document.createElement("section");
  section.className = "hero page-wrap";
  section.innerHTML = `
    <div class="hero__badge">${t.hero.badge}</div>
    <h1 class="hero__title">${t.hero.title} <span>${t.hero.titleAccent}</span></h1>
    <p class="hero__subtitle">${t.hero.subtitle}</p>
    <div class="hero__engines">
      ${t.engineTags.map((tag) => `<span class="engine-tag">${tag}</span>`).join("")}
    </div>
  `;
  section.appendChild(createCtaButtons(t, "hero"));
  return section;
}
