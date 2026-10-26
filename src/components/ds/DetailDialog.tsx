import { useEffect, useId, useRef, type ReactNode } from "react";
import { X } from "lucide-react";
import { useTranslation } from "react-i18next";

/** Native modal keeps background controls inert and restores the opener's focus. */
export function DetailDialog({
  open,
  title,
  onClose,
  children,
  footer,
}: {
  open: boolean;
  title: string;
  onClose: () => void;
  children: ReactNode;
  footer?: ReactNode;
}) {
  const ref = useRef<HTMLDialogElement>(null);
  const heading = useId();
  const { t } = useTranslation("common");
  useEffect(() => {
    if (!open) return;
    const dialog = ref.current;
    const previous = document.activeElement as HTMLElement | null;
    dialog?.showModal();
    return () => {
      dialog?.close();
      previous?.focus();
    };
  }, [open]);
  return (
    <dialog
      ref={ref}
      aria-labelledby={heading}
      className="detail-dialog"
      onCancel={(event) => {
        event.preventDefault();
        onClose();
      }}
    >
      <div className="flex shrink-0 items-start justify-between gap-4 p-6 pb-4">
        <h2 id={heading} className="min-w-0 break-words text-xl font-medium">
          {title}
        </h2>
        <button
          type="button"
          onClick={onClose}
          aria-label={t("close")}
          className="material-icon-button"
        >
          <X size={20} />
        </button>
      </div>
      <div className="min-h-0 overflow-y-auto px-6 pb-6">
        {open && children}
      </div>
      {open && footer && (
        <div className="flex shrink-0 justify-end gap-2 border-t border-[var(--color-border)] px-6 py-4">
          {footer}
        </div>
      )}
    </dialog>
  );
}
