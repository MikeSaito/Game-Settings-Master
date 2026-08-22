import type { LocaleStrings } from "../i18n/types";
import type { MockScreen } from "./appWindow";
import { buildContextBar, buildModeBar } from "./chrome";
import { badge, icon, mockButton } from "./ds";

export function buildBackupsScreen(
  t: LocaleStrings,
  onPanel: (screen: MockScreen) => void,
): { el: HTMLElement; cleanup: () => void } {
  const el = document.createElement("div");
  el.className = "mock__editor";

  const game = t.mock.games[0];
  if (!game) return { el, cleanup: () => {} };

  const content = document.createElement("div");
  content.className = "mock__backups";

  const countRow = document.createElement("div");
  countRow.className = "mock__stats";
  countRow.append(badge("neutral", t.mock.backupsCount(t.mock.backups.length)));

  const alert = document.createElement("div");
  alert.className = "mock__alert";
  alert.append(icon("info", 13));
  const alertText = document.createElement("div");
  const alertTitle = document.createElement("div");
  alertTitle.className = "mock__alert-title";
  alertTitle.textContent = t.mock.howTitle;
  const alertBody = document.createElement("div");
  alertBody.className = "mock__alert-body";
  alertBody.textContent = t.mock.howBody;
  alertText.append(alertTitle, alertBody);
  alert.append(alertText);

  const sectionHead = document.createElement("div");
  sectionHead.className = "mock__section-head";
  const sectionTitle = document.createElement("div");
  sectionTitle.className = "mock__section-title";
  sectionTitle.textContent = t.mock.listTitle;
  const sectionDesc = document.createElement("div");
  sectionDesc.className = "mock__section-desc";
  sectionDesc.textContent = t.mock.listDesc;
  sectionHead.append(sectionTitle, sectionDesc);

  const list = document.createElement("ul");
  list.className = "mock__list";

  for (const b of t.mock.backups) {
    const li = document.createElement("li");
    const row = document.createElement("div");
    row.className = "mock__backup";

    const info = document.createElement("div");
    info.className = "mock__backup-info";
    const topLine = document.createElement("div");
    topLine.className = "mock__backup-top";
    const label = document.createElement("span");
    label.className = "mock__backup-label";
    label.textContent = b.label;
    const id = document.createElement("span");
    id.className = "mock__backup-id";
    id.textContent = b.id;
    topLine.append(label, id);
    const files = document.createElement("div");
    files.className = "mock__backup-files";
    for (const file of b.files) {
      files.append(badge("neutral", file));
    }
    info.append(topLine, files);

    row.append(info, mockButton("secondary", t.mock.restore, "restore"));
    li.append(row);
    list.append(li);
  }

  content.append(countRow, alert, sectionHead, list);

  el.append(buildContextBar(t, game), buildModeBar(t, "backups", onPanel), content);
  return { el, cleanup: () => {} };
}
