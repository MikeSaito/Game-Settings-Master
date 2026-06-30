import { assetPath, localeHome } from "../lib/site";
import type { LocaleStrings } from "../i18n/types";

export function buildHeader(t: LocaleStrings): HTMLElement {
  const el = document.createElement("header");
  el.className = "topbar";
  const home = localeHome(t.lang);
  const other = localeHome(t.lang === "en" ? "ru" : "en");
  const lang = t.lang === "en" ? t.localeSwitch.ru : t.localeSwitch.en;

  el.innerHTML = `
    <a href="${home}" class="topbar__brand">
      <img src="${assetPath("logo.svg")}" width="28" height="28" alt="" />
      <span>${t.siteName}</span>
    </a>
    <nav class="topbar__nav" aria-label="${t.nav.aria}">
      <a href="${home}#features">${t.nav.features}</a>
      <a href="${home}#modes">${t.nav.modes}</a>
      <a href="${home}#faq">${t.nav.faq}</a>
      <a href="${home}#download" class="topbar__dl">${t.nav.download}</a>
      <a href="${other}" class="topbar__lang" hreflang="${t.lang === "en" ? "ru" : "en"}">${lang}</a>
    </nav>
  `;
  return el;
}
