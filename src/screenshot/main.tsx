import ReactDOM from "react-dom/client";
import i18n, { type AppLanguage } from "../i18n";
import "../index.css";
import "./screenshot.css";
import { ScreenshotFrames } from "./frames";
import { FeatureFrames } from "./featureFrames";

function screenshotLang(): AppLanguage {
  const value = new URLSearchParams(window.location.search).get("lang");
  return value === "en" ? "en" : "ru";
}

const lang = screenshotLang();
document.documentElement.setAttribute("data-theme", "dark");
document.documentElement.lang = lang;

void i18n.changeLanguage(lang).then(() => {
  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    new URLSearchParams(window.location.search).has("feature") ? (
      <FeatureFrames
        lang={lang}
        feature={new URLSearchParams(window.location.search).get("feature")!}
      />
    ) : (
      <ScreenshotFrames lang={lang} />
    ),
  );
});
