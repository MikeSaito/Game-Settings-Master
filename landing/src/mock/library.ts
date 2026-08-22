import type { LocaleStrings } from "../i18n/types";
import { sourceLabel } from "./appWindow";
import { badge, icon, mockButton } from "./ds";

export function buildLibraryScreen(t: LocaleStrings): { el: HTMLElement; cleanup: () => void } {
  const el = document.createElement("div");
  el.className = "mock__library";

  const head = document.createElement("div");
  head.className = "mock__lib-head";
  const headText = document.createElement("div");
  const kicker = document.createElement("p");
  kicker.className = "mock__lib-kicker";
  kicker.textContent = t.mock.windowTitle;
  const title = document.createElement("h3");
  title.className = "mock__lib-title";
  title.textContent = t.mock.libTitle;
  const subtitle = document.createElement("p");
  subtitle.className = "mock__lib-subtitle";
  subtitle.textContent = t.mock.libSubtitle;
  headText.append(kicker, title, subtitle);
  const headBadges = document.createElement("div");
  headBadges.className = "mock__lib-badges";
  const total = t.mock.games.length;
  const withConfig = t.mock.games.filter((g) => g.ok).length;
  const ueCount = t.mock.games.filter((g) => g.engine.startsWith("UE")).length;
  headBadges.append(
    badge("info", t.mock.badgeTotal(total)),
    badge("success", t.mock.badgeConfig(withConfig)),
    badge("accent", t.mock.badgeUe(ueCount)),
    badge("neutral", t.mock.badgeCover(total)),
  );
  head.append(headText, headBadges);

  const toolbar = document.createElement("div");
  toolbar.className = "mock__toolbar";
  const searchWrap = document.createElement("div");
  searchWrap.className = "mock__search";
  searchWrap.append(icon("search", 12));
  const search = document.createElement("input");
  search.type = "search";
  search.readOnly = true;
  search.tabIndex = -1;
  search.placeholder = t.mock.search;
  search.setAttribute("aria-label", t.mock.search);
  searchWrap.append(search);

  const viewSegment = document.createElement("div");
  viewSegment.className = "mock__segment";
  const gridBtn = document.createElement("button");
  gridBtn.type = "button";
  gridBtn.className = "mock__segment-btn is-active";
  gridBtn.setAttribute("aria-label", "Grid");
  gridBtn.tabIndex = -1;
  gridBtn.append(icon("grid", 12));
  const listBtn = document.createElement("button");
  listBtn.type = "button";
  listBtn.className = "mock__segment-btn";
  listBtn.setAttribute("aria-label", "List");
  listBtn.tabIndex = -1;
  listBtn.append(icon("list", 12));
  viewSegment.append(gridBtn, listBtn);

  toolbar.append(
    searchWrap,
    viewSegment,
    mockButton("secondary", t.mock.scan, "refresh"),
    mockButton("primary", t.mock.add, "plus"),
  );

  const list = document.createElement("ul");
  list.className = "mock__grid";

  for (const game of t.mock.games) {
    const li = document.createElement("li");
    const card = document.createElement("article");
    card.className = "mock__card";
    card.dataset.id = game.id;
    if (game.id === t.mock.games[0]?.id) card.classList.add("is-selected");

    const cover = document.createElement("div");
    cover.className = "mock__cover";
    cover.style.setProperty("--hue", String(game.hue));
    cover.setAttribute("aria-hidden", "true");
    const letter = document.createElement("span");
    letter.className = "mock__cover-letter";
    letter.textContent = game.name.trim().charAt(0).toUpperCase();
    cover.append(letter);
    if (game.fav) {
      const fav = document.createElement("span");
      fav.className = "mock__cover-fav";
      fav.append(icon("heart", 11));
      cover.append(fav);
    }

    const body = document.createElement("div");
    body.className = "mock__card-body";
    const name = document.createElement("p");
    name.className = "mock__card-name";
    name.textContent = game.name;
    const chips = document.createElement("p");
    chips.className = "mock__card-meta";
    chips.append(
      badge("neutral", sourceLabel(t, game.source)),
      badge("accent", game.engine),
      badge(game.ok ? "success" : "warning", game.ok ? t.mock.configOk : t.mock.configMissing),
    );
    const path = document.createElement("p");
    path.className = "mock__card-path";
    path.textContent = game.path;
    body.append(name, chips, path);

    const footer = document.createElement("div");
    footer.className = "mock__card-actions";
    footer.append(
      mockButton("primary", t.mock.select),
      mockButton("secondary", t.mock.cover),
    );

    card.append(cover, body, footer);
    li.append(card);
    list.append(li);
  }

  el.append(head, toolbar, list);
  return { el, cleanup: () => {} };
}
