import type { LocaleStrings } from "../i18n/types";
import { makeActions } from "../ui/cta";

export function buildHero(t: LocaleStrings): HTMLElement {
  const el = document.createElement("section");
  el.className = "gate";
  el.innerHTML = `
    <div class="gate__body wrap">
      <p class="gate__label">${t.hero.kicker}</p>
      <h1 class="gate__title">
        <span>${t.hero.title}</span>
        <em>${t.hero.titleAccent}</em>
      </h1>
      <p class="gate__lead">${t.hero.subtitle}</p>
    </div>
    <div class="gate__scroll">${t.lang === "ru" ? "листайте" : "scroll"}</div>
    <p class="gate__credit">${t.hero.sceneCredit}</p>
  `;
  el.querySelector(".gate__body")?.append(makeActions(t, "hero"));
  return el;
}
