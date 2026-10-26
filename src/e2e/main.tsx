import ReactDOM from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import App from "../App";
import "../i18n";
import "@/lib/settings";
import "../index.css";
import { resetE2eMockState } from "./mockInvoke";
import { APP_SETTINGS_STORAGE_KEY, DEFAULT_APP_SETTINGS } from "@/lib/settings";

const previewParams = new URLSearchParams(window.location.search);
const previewLanguage = previewParams.get("lang");
if (previewLanguage === "ru" || previewLanguage === "en") {
  sessionStorage.setItem("gsm-e2e-language", previewLanguage);
}
const fixtureLanguage =
  sessionStorage.getItem("gsm-e2e-language") === "ru" ? "ru" : "en";
const fixtureTheme = previewParams.get("theme") === "light" ? "light" : "dark";
resetE2eMockState();

try {
  localStorage.setItem(
    APP_SETTINGS_STORAGE_KEY,
    JSON.stringify({
      ...DEFAULT_APP_SETTINGS,
      language: fixtureLanguage,
      theme: fixtureTheme,
    }),
  );
  localStorage.setItem("uesm:language", fixtureLanguage);
} catch {
  /* ignore */
}

document.documentElement.dataset.theme = fixtureTheme;
document.documentElement.lang = fixtureLanguage;

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <BrowserRouter>
    <App />
  </BrowserRouter>,
);
