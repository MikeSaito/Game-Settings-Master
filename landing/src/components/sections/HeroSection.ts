import type { LocaleStrings } from "../../i18n/types";
import { localeShot } from "../../lib/shots";
import { createAppShot } from "../shared/AppShot";
import { createCtaButtons } from "../shared/CtaButtons";

export function createHeroSection(t: LocaleStrings): HTMLElement {
  const section = document.createElement("section");
  section.className = "hero";
  section.innerHTML = `
    <div class="hero__inner page-wrap">
      <div class="hero__content reveal-stagger">
        <p class="hero__kicker">${t.hero.kicker}</p>
        <h1 class="hero__title">
          ${t.hero.title}
          <span class="hero__title-accent">${t.hero.titleAccent}</span>
        </h1>
        <p class="hero__subtitle">${t.hero.subtitle}</p>
      </div>
      <div class="hero__visual reveal-stagger">
        <div class="hero__aura" aria-hidden="true"></div>
      </div>
    </div>
  `;
  section.querySelector(".hero__content")?.appendChild(createCtaButtons(t, "hero"));
  section.querySelector(".hero__visual")?.appendChild(
    createAppShot({
      src: localeShot(t.lang, "editor-basic.png"),
      alt: t.hero.shotAlt,
      variant: "hero",
      loading: "eager",
    }),
  );
  return section;
}
