import { downloadUrl } from "../lib/site";
import type { LocaleStrings } from "../i18n/types";

let open: HTMLDialogElement | null = null;

export function showDownloadModal(t: LocaleStrings): void {
  if (open) {
    open.showModal();
    return;
  }

  const s = t.download.smartScreen;
  const dlg = document.createElement("dialog");
  dlg.className = "modal";
  dlg.innerHTML = `
    <form method="dialog" class="modal__sheet">
      <h2 class="modal__title">${s.title}</h2>
      <p class="modal__intro">${s.intro}</p>
      <ol class="modal__steps"><li>${s.step1}</li><li>${s.step2}</li></ol>
      <p class="modal__note">${s.note}</p>
      <div class="modal__actions">
        <button type="submit" class="btn btn--line" value="cancel">${s.cancel}</button>
        <a class="btn btn--fill" href="${downloadUrl}" rel="noopener">${s.confirm}</a>
      </div>
    </form>
  `;
  dlg.querySelector("a")?.addEventListener("click", () => dlg.close());
  document.body.append(dlg);
  open = dlg;
  dlg.showModal();
  dlg.addEventListener("close", () => {
    dlg.remove();
    open = null;
  });
}
