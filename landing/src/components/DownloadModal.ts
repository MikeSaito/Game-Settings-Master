import { downloadUrl } from "../lib/site";
import type { LocaleStrings } from "../i18n/types";

let activeDialog: HTMLDialogElement | null = null;

export function openDownloadModal(t: LocaleStrings): void {
  if (activeDialog) {
    activeDialog.showModal();
    return;
  }

  const dialog = document.createElement("dialog");
  dialog.className = "download-modal";
  dialog.setAttribute("aria-labelledby", "download-modal-title");

  const s = t.download.smartScreen;

  dialog.innerHTML = `
    <form method="dialog" class="download-modal__panel">
      <h2 id="download-modal-title" class="download-modal__title">${s.title}</h2>
      <p class="download-modal__intro">${s.intro}</p>
      <ol class="download-modal__steps">
        <li>${s.step1}</li>
        <li>${s.step2}</li>
      </ol>
      <p class="download-modal__note">${s.note}</p>
      <div class="download-modal__actions">
        <button type="submit" class="btn btn--ghost" value="cancel">${s.cancel}</button>
        <a class="btn btn--primary download-modal__confirm" href="${downloadUrl}" rel="noopener">
          ${s.confirm}
        </a>
      </div>
    </form>
  `;

  const confirm = dialog.querySelector<HTMLAnchorElement>(".download-modal__confirm");
  confirm?.addEventListener("click", () => {
    dialog.close();
  });

  document.body.appendChild(dialog);
  activeDialog = dialog;
  dialog.showModal();

  dialog.addEventListener("close", () => {
    dialog.remove();
    activeDialog = null;
  });
}
