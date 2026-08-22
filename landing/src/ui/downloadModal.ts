import { downloadUrl } from "../lib/site";
import type { LocaleStrings } from "../i18n/types";

let open: HTMLDialogElement | null = null;

export function showDownloadModal(t: LocaleStrings): void {
  // Диалог либо уже открыт, либо удален из DOM обработчиком close.
  if (open) return;

  const s = t.download.smartScreen;
  const dlg = document.createElement("dialog");
  dlg.className = "modal";

  const form = document.createElement("form");
  form.method = "dialog";
  form.className = "modal__sheet";

  const title = document.createElement("h2");
  title.className = "modal__title";
  title.textContent = s.title;

  const intro = document.createElement("p");
  intro.className = "modal__intro";
  intro.textContent = s.intro;

  const steps = document.createElement("ol");
  steps.className = "modal__steps";
  for (const text of [s.step1, s.step2]) {
    const li = document.createElement("li");
    li.textContent = text;
    steps.append(li);
  }

  const note = document.createElement("p");
  note.className = "modal__note";
  note.textContent = s.note;

  const actions = document.createElement("div");
  actions.className = "modal__actions";

  const cancel = document.createElement("button");
  cancel.type = "submit";
  cancel.className = "btn btn--line";
  cancel.value = "cancel";
  cancel.textContent = s.cancel;

  const confirm = document.createElement("a");
  confirm.className = "btn btn--fill";
  confirm.href = downloadUrl;
  confirm.rel = "noopener";
  confirm.textContent = s.confirm;
  confirm.addEventListener("click", () => dlg.close());

  actions.append(cancel, confirm);
  form.append(title, intro, steps, note, actions);
  dlg.append(form);
  document.body.append(dlg);
  open = dlg;
  dlg.showModal();
  dlg.addEventListener("close", () => {
    dlg.remove();
    open = null;
  });
}
