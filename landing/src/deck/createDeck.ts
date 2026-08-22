import type { LocaleStrings } from "../i18n/types";
import {
  APP_VERSION,
  assetPath,
  donateUrl,
  githubUrl,
  localeHome,
  telegramUrl,
} from "../lib/site";
import { showDownloadModal } from "../ui/downloadModal";
import {
  ScrollTrigger,
  createMatchMedia,
  observeReveals,
  animateWhenVisible,
  FALLBACK_QUERY,
  type Scene,
} from "./scroll";
import { buildHeroScene } from "./scenes/hero";
import { buildScanScene } from "./scenes/scan";
import { buildBackupScene } from "./scenes/backup";
import { buildModesScene } from "./scenes/modes";
import { buildFinalScene } from "./scenes/final";

function buildHeader(t: LocaleStrings): { el: HTMLElement; cleanup: () => void } {
  const header = document.createElement("header");
  header.className = "deck__chrome";

  const brand = document.createElement("a");
  brand.className = "deck__brand";
  brand.href = localeHome(t.lang);
  const logo = document.createElement("img");
  logo.src = assetPath("logo.svg");
  logo.alt = "";
  logo.width = 28;
  logo.height = 28;
  const name = document.createElement("span");
  name.textContent = t.siteName;
  brand.append(logo, name);

  const tools = document.createElement("div");
  tools.className = "deck__tools";

  const other = t.lang === "ru" ? "en" : "ru";
  const lang = document.createElement("a");
  lang.className = "deck__lang";
  lang.href = localeHome(other);
  lang.textContent = other === "en" ? t.chrome.en : t.chrome.ru;
  lang.setAttribute("aria-label", t.chrome.langLabel);
  lang.hreflang = other;

  const download = document.createElement("button");
  download.type = "button";
  download.className = "btn btn--fill deck__dl";
  download.textContent = t.chrome.download;
  const onDownload = () => showDownloadModal(t);
  download.addEventListener("click", onDownload);

  tools.append(lang, download);
  header.append(brand, tools);

  return {
    el: header,
    cleanup: () => download.removeEventListener("click", onDownload),
  };
}

function buildFlow(t: LocaleStrings): HTMLElement {
  const flow = document.createElement("div");
  flow.className = "flow";

  const faq = document.createElement("section");
  faq.className = "faq";
  faq.dataset.reveal = "";
  const faqTitle = document.createElement("h2");
  faqTitle.className = "faq__title";
  faqTitle.textContent = t.story.flow.faqTitle;
  faq.append(faqTitle);
  for (const item of t.faq) {
    const details = document.createElement("details");
    const summary = document.createElement("summary");
    summary.textContent = item.question;
    details.append(summary);
    for (const text of item.paragraphs) {
      const p = document.createElement("p");
      p.textContent = text;
      details.append(p);
    }
    faq.append(details);
  }

  const donate = document.createElement("section");
  donate.className = "donate";
  donate.dataset.reveal = "";
  const donateText = document.createElement("p");
  const donateStrong = document.createElement("strong");
  donateStrong.textContent = t.donate.title;
  donateText.append(donateStrong, document.createTextNode(` ${t.donate.text}`));
  const donateLink = document.createElement("a");
  donateLink.className = "btn btn--line";
  donateLink.href = donateUrl;
  donateLink.target = "_blank";
  donateLink.rel = "noopener noreferrer";
  donateLink.textContent = t.donate.button;
  donate.append(donateText, donateLink);

  flow.append(faq, donate);
  return flow;
}

function buildFooter(t: LocaleStrings): HTMLElement {
  const footer = document.createElement("footer");
  footer.className = "deck__footer";

  const links = document.createElement("nav");
  links.className = "deck__links";
  const entries: Array<[string, string]> = [
    ["GitHub", githubUrl],
    [t.footer.telegramLink, telegramUrl],
    [t.footer.donateLink, donateUrl],
  ];
  for (const [text, href] of entries) {
    const a = document.createElement("a");
    a.href = href;
    a.target = "_blank";
    a.rel = "noopener noreferrer";
    a.textContent = text;
    links.append(a);
  }

  const ver = document.createElement("span");
  ver.className = "deck__ver";
  ver.textContent = t.footer.version(APP_VERSION);

  footer.append(links, ver);
  return footer;
}

export function createDeck(t: LocaleStrings): { el: HTMLElement; cleanup: () => void } {
  const root = document.createElement("div");
  root.className = "deck story";

  const header = buildHeader(t);

  const main = document.createElement("main");
  main.className = "story__scenes";
  main.setAttribute("aria-label", t.chrome.aria);

  const scenes: Scene[] = [
    buildHeroScene(t),
    buildScanScene(t),
    buildBackupScene(t),
    buildModesScene(t),
    buildFinalScene(t),
  ];
  for (const scene of scenes) main.append(scene.el);

  root.append(header.el, main, buildFlow(t), buildFooter(t));

  const mm = createMatchMedia();
  for (const scene of scenes) scene.init(mm);
  mm.add(FALLBACK_QUERY, () => {
    root.classList.add("story--fallback");
    const stop = observeReveals(root);
    return () => {
      root.classList.remove("story--fallback");
      stop();
    };
  });

  const stopHint = animateWhenVisible(root, ".scene__hint", "is-animated");
  const stopCta = animateWhenVisible(root, ".final__cta .btn--fill", "is-animated");

  // Корень монтируется в DOM после возврата: refresh после вставки и после загрузки шрифтов.
  const raf = requestAnimationFrame(() => ScrollTrigger.refresh());
  let fontsAlive = true;
  document.fonts?.ready.then(() => {
    if (fontsAlive) ScrollTrigger.refresh();
  });

  return {
    el: root,
    cleanup: () => {
      fontsAlive = false;
      cancelAnimationFrame(raf);
      stopHint();
      stopCta();
      mm.revert();
      for (const scene of scenes) scene.cleanup();
      header.cleanup();
    },
  };
}
