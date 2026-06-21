import "./styles/landing.css";
import { getLocale } from "./i18n";
import { createSiteHeader } from "./components/SiteHeader";
import { createHeroSection } from "./components/HeroSection";
import { createFeatureSection } from "./components/FeatureSection";
import { createDownloadSection } from "./components/DownloadSection";
import { createSiteFooter } from "./components/SiteFooter";
import { createStatsBar } from "./components/StatsBar";
import { createBasicAdvancedSection } from "./components/BasicAdvancedSection";
import { createHowItWorksSection } from "./components/HowItWorksSection";
import { createCatalogHighlightSection } from "./components/CatalogHighlightSection";
import { createGpuSection } from "./components/GpuSection";
import { createFaqSection } from "./components/FaqSection";
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
