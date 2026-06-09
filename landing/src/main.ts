import "./styles/landing.css";
import sceneMaster from "./svg/scene-master.svg?raw";
import { getLocale } from "./i18n";
import { createSiteHeader } from "./components/SiteHeader";
import { createHeroSection } from "./components/HeroSection";
import { createFeatureSection } from "./components/FeatureSection";
import { createDownloadSection } from "./components/DownloadSection";
import { createSiteFooter } from "./components/SiteFooter";
import { applyReveal, mountScene } from "./scroll/revealController";
import { useScrollProgress } from "./scroll/useScrollProgress";

function init(): void {
  const t = getLocale();
  mountScene(sceneMaster);

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
    featuresSection,
    createDownloadSection(t),
    createSiteFooter(t),
  );

  app.append(createSiteHeader(t), main);

  useScrollProgress(applyReveal);
}

init();
