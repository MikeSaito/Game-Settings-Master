import type { LocaleStrings } from "../i18n/types";
import { createDeck } from "../deck/createDeck";

export function initSite(t: LocaleStrings): () => void {
  const app = document.getElementById("app");
  if (!app) return () => {};

  const deck = createDeck(t);
  app.replaceChildren(deck.el);
  return deck.cleanup;
}
