import "./styles/landing.css";
import { getLocale } from "./i18n";
import { createSiteHeader } from "./components/layout/SiteHeader";
import { createSiteFooter } from "./components/layout/SiteFooter";
import { createHeroSection } from "./components/sections/HeroSection";
import { createFeatureSection } from "./components/sections/FeatureSection";
import { createDownloadSection } from "./components/sections/DownloadSection";
import { createStatsBar } from "./components/sections/StatsBar";
import { createBasicAdvancedSection } from "./components/sections/BasicAdvancedSection";
import { createHowItWorksSection } from "./components/sections/HowItWorksSection";
import { createCatalogHighlightSection } from "./components/sections/CatalogHighlightSection";
import { createGpuSection } from "./components/sections/GpuSection";
import { createFaqSection } from "./components/sections/FaqSection";
import { initStaggeredReveal } from "./scroll/revealController";

function init(): void {
  const t = getLocale();

  const app = document.getElementById("app");
  if (!app) return;

  const main = document.createElement("main");
  main.className = "site-main";

  const featuresSection = document.createElement("section");
  featuresSection.className = "features page-wrap";
  featuresSection.id = "features";
  for (const feature of t.features) {
    featuresSection.appendChild(createFeatureSection(feature));
  }

  main.append(
    createHeroSection(t),
    createStatsBar(t),
    createBasicAdvancedSection(t),
    featuresSection,
    createHowItWorksSection(t),
    createCatalogHighlightSection(t.catalogHighlight),
    createGpuSection(t.gpu),
    createDownloadSection(t),
    createFaqSection(t),
    createSiteFooter(t),
  );

  app.append(createSiteHeader(t), main);

  initStaggeredReveal();
}

init();
