import type { NavigateFunction, Location } from "react-router-dom";
import { libraryPath } from "./routes";

export function goToLibrary(navigate: NavigateFunction, location: Location): void {
  const target = libraryPath();
  if (location.pathname === target && !location.hash && !location.search) {
    window.scrollTo({ top: 0, behavior: "smooth" });
    return;
  }
  navigate({ pathname: target, search: "", hash: "" });
}
