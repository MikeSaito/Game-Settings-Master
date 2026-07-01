import ReactDOM from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import App from "../App";
import "../i18n";
import "@/lib/settings";
import "../index.css";
import { resetE2eMockState } from "./mockInvoke";
import { APP_SETTINGS_STORAGE_KEY, DEFAULT_APP_SETTINGS } from "@/lib/settings";

resetE2eMockState();

try {
  localStorage.setItem(
    APP_SETTINGS_STORAGE_KEY,
    JSON.stringify({
      ...DEFAULT_APP_SETTINGS,
      language: "en",
      theme: "dark",
    }),
  );
  localStorage.setItem("uesm:language", "en");
} catch {
  /* ignore */
}

document.documentElement.dataset.theme = "dark";
document.documentElement.lang = "en";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <BrowserRouter>
    <App />
  </BrowserRouter>,
);
