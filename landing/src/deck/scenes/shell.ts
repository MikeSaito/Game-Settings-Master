import type { StorySceneText } from "../../i18n/types";

export interface SceneShell {
  section: HTMLElement;
  pin: HTMLElement;
  copy: HTMLElement;
  stage: HTMLElement;
}

export function sceneShell(
  modifier: string,
  text: StorySceneText,
  opts?: { titleAs?: "h1" | "h2" },
): SceneShell {
  const section = document.createElement("section");
  section.className = `scene scene--${modifier}`;
  section.dataset.scene = modifier;

  const pin = document.createElement("div");
  pin.className = "scene__pin";

  const copy = document.createElement("div");
  copy.className = "scene__copy";
  copy.dataset.reveal = "";

  const kicker = document.createElement("p");
  kicker.className = "scene__kicker";
  kicker.textContent = text.kicker;

  const title = document.createElement(opts?.titleAs ?? "h2");
  title.className = "scene__title";
  title.textContent = text.title;

  const lead = document.createElement("p");
  lead.className = "scene__text";
  lead.textContent = text.text;

  copy.append(kicker, title, lead);

  const stage = document.createElement("div");
  stage.className = "scene__stage";
  stage.dataset.reveal = "";

  pin.append(copy, stage);
  section.append(pin);

  return { section, pin, copy, stage };
}
