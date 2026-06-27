import "./styles/landing.css";
import { getLocale, maybeRedirectToEnglishHome } from "./i18n";
import { createSiteHeader } from "./components/layout/SiteHeader";
import { createSiteFooter } from "./components/layout/SiteFooter";
import { createHeroSection } from "./components/sections/HeroSection";
import { createMarqueeStrip } from "./components/sections/MarqueeStrip";
import { createShowcaseSection } from "./components/sections/ShowcaseSection";
import { createFeatureSection } from "./components/sections/FeatureSection";
import { createFaqSection } from "./components/sections/FaqSection";
import { createDownloadSection } from "./components/sections/DownloadSection";
import { initStaggeredReveal } from "./scroll/revealController";
import { initLandingEffects } from "./effects/initEffects";

function init(): void {
  maybeRedirectToEnglishHome();

  const t = getLocale();
  document.documentElement.lang = t.lang;

  const app = document.getElementById("app");
  if (!app) return;

  const main = document.createElement("main");
  main.className = "site-main";

  const featuresSection = document.createElement("section");
  featuresSection.className = "features page-wrap";
  featuresSection.id = "features";
  const featuresIntro = document.createElement("div");
  featuresIntro.className = "section-heading section-heading--center reveal-stagger";
  featuresIntro.innerHTML = `<h2>${t.lang === "ru" ? "Возможности" : "Features"}</h2>`;
  featuresSection.appendChild(featuresIntro);
  for (const feature of t.features) {
    featuresSection.appendChild(createFeatureSection(feature));
  }

  main.append(
    createHeroSection(t),
    createMarqueeStrip(t),
    createShowcaseSection(t),
    featuresSection,
    createFaqSection(t),
    createDownloadSection(t),
    createSiteFooter(t),
  );

  app.append(createSiteHeader(t), main);
  initLandingEffects();
  initStaggeredReveal();
}

init();
