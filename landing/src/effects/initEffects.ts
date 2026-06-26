import { initAbstractCanvas } from "./abstractCanvas";
import { initMouseState } from "./mouseState";

export function initLandingEffects(): void {
  const cleanups: Array<() => void> = [];

  const mouseCleanup = initMouseState();
  cleanups.push(mouseCleanup);

  const canvasCleanup = initAbstractCanvas();
  if (canvasCleanup) cleanups.push(canvasCleanup);

  window.addEventListener(
    "pagehide",
    () => {
      for (const cleanup of cleanups) cleanup();
    },
    { once: true },
  );
}
