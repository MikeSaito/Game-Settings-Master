import { assetPath, localeHome } from "../lib/site";
import type { LocaleStrings } from "../i18n/types";

export function createSiteHeader(t: LocaleStrings): HTMLElement {
  const header = document.createElement("header");
  header.className = "site-header";
  const homeHref = localeHome(t.lang);
  const otherHref = localeHome(t.lang === "en" ? "ru" : "en");
  const otherLabel = t.lang === "en" ? t.localeSwitch.ru : t.localeSwitch.en;
  const logoSrc = assetPath("logo.png");

  header.innerHTML = `
    <a href="${homeHref}" class="site-header__brand">
      <img src="${logoSrc}" width="28" height="28" alt="" class="site-header__logo" />
      <span>${t.siteName}</span>
    </a>
    <nav class="site-header__nav" aria-label="${t.nav.aria}">
      <a href="${homeHref}#features">${t.nav.features}</a>
      <a href="${homeHref}#download">${t.nav.download}</a>
      <a href="${otherHref}" class="site-header__locale" hreflang="${t.lang === "en" ? "ru" : "en"}">${otherLabel}</a>
    </nav>
  `;

  const onScroll = () => {
    header.classList.toggle("is-scrolled", window.scrollY > 50);
  };
  window.addEventListener("scroll", onScroll, { passive: true });
  onScroll();

  return header;
}
