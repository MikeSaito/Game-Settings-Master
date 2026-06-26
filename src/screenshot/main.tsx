import ReactDOM from "react-dom/client";
import i18n, { type AppLanguage } from "../i18n";
import "../index.css";
import "./screenshot.css";
import { ScreenshotFrames } from "./frames";

function screenshotLang(): AppLanguage {
  const value = new URLSearchParams(window.location.search).get("lang");
  return value === "en" ? "en" : "ru";
}

const lang = screenshotLang();
document.documentElement.setAttribute("data-theme", "light");
document.documentElement.lang = lang;

void i18n.changeLanguage(lang).then(() => {
  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <ScreenshotFrames lang={lang} />,
  );
});
