import "./styles/index.css";
import { getLocale, maybeRedirectToEnglishHome } from "./i18n";
import { initSite } from "./site/buildPage";

function start(): void {
  maybeRedirectToEnglishHome();

  const t = getLocale();
  document.documentElement.lang = t.lang;

  initSite(t);
}

start();
