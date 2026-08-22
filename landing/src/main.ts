import "./styles/index.css";
import { getLocale, maybeRedirectToEnglishHome } from "./i18n";
import { initSite } from "./site/buildPage";

function start(): void {
  maybeRedirectToEnglishHome();

  const t = getLocale();
  document.documentElement.lang = t.htmlLang;
  document.title = t.meta.title;

  const cleanup = initSite(t);

  if (import.meta.hot) {
    import.meta.hot.accept();
    import.meta.hot.dispose(() => cleanup());
  }
}

start();
