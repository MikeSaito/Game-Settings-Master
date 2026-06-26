import type { NavigateFunction, Location } from "react-router-dom";
import type { EditorPanel } from "./editorPanels";
import { writeStoredPanel } from "./editorPanels";
import { gameTabPath, libraryPath } from "./routes";

export function goToLibrary(navigate: NavigateFunction, location: Location): void {
  const target = libraryPath();
  if (location.pathname === target && !location.hash && !location.search) {
    window.scrollTo({ top: 0, behavior: "smooth" });
    return;
  }
  navigate({ pathname: target, search: "", hash: "" });
}

/** Editor URL is always `/advanced`; panel lives in sessionStorage. */
export function openGameEditor(
  navigate: NavigateFunction,
  gameId: string,
  panel: EditorPanel = "basic",
): void {
  writeStoredPanel(gameId, panel);
  navigate(gameTabPath(gameId, "advanced"));
}
