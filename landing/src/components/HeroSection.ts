import type { LocaleStrings } from "../i18n/types";
import { createCtaButtons } from "./CtaButtons";

export function createHeroSection(t: LocaleStrings): HTMLElement {
  const section = document.createElement("section");
  section.className = "hero page-wrap reveal-stagger";
  section.innerHTML = `
    <div class="hero__content">
      <div class="hero__badge">${t.hero.badge}</div>
      <h1 class="hero__title">${t.hero.title} <span>${t.hero.titleAccent}</span></h1>
      <p class="hero__subtitle">${t.hero.subtitle}</p>
      <div class="hero__engines">
        ${t.engineTags.map((tag) => `<span class="engine-tag">${tag}</span>`).join("")}
      </div>
    </div>
    <div class="mock-window hero__mock" aria-hidden="true">
      <div class="mock-window__chrome">
        <b>Advanced Editor</b>
      </div>
      <div class="mock-window__body">
        <aside class="mock-window__rail">
          <span class="is-active"></span>
          <span></span>
          <span></span>
        </aside>
        <div class="mock-window__panel">
          <div class="mock-window__mode">
            <span class="is-active">Basic</span>
            <span>Advanced</span>
          </div>
          <div class="mock-row">
            <div><b>sg.TextureQuality</b><small>GameUserSettings.ini</small></div>
            <span class="mock-pill">High</span>
          </div>
          <div class="mock-row">
            <div><b>ResolutionSizeX</b><small>Display</small></div>
            <span class="mock-pill mock-pill--safe">2560</span>
          </div>
          <div class="mock-row mock-row--expert">
            <div><b>r.Nanite.MaxPixelsPerEdge</b><small>Engine.ini · backup ready</small></div>
            <span class="mock-pill mock-pill--warn">CVar</span>
          </div>
        </div>
      </div>
    </div>
  `;
  section.querySelector(".hero__content")?.appendChild(createCtaButtons(t, "hero"));

  const indicator = document.createElement("div");
  indicator.className = "hero__scroll";
  indicator.setAttribute("aria-hidden", "true");
  indicator.innerHTML = "<span></span>";
  section.appendChild(indicator);
  return section;
}
