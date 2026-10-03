import "./styles/landing.css";
import { en, ru } from "./content";
import { initDemo } from "./demo";

const t = document.documentElement.lang === "en" ? en : ru;
const demo = document.getElementById("demo-content");
const cleanup = demo ? initDemo(demo, t) : () => {};
const openHelp = () => {
  if (location.hash !== "#smartscreen") return;
  const details = document.getElementById(
    "smartscreen",
  ) as HTMLDetailsElement | null;
  if (details) details.open = true;
};
openHelp();
window.addEventListener("hashchange", openHelp);
if (import.meta.hot)
  import.meta.hot.dispose(() => {
    cleanup();
    window.removeEventListener("hashchange", openHelp);
  });
