import { AlertTriangle } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "./ui/Button";

export type ReShadeDisclaimerKind = "enable" | "install" | "launch";

interface Props {
  kind: ReShadeDisclaimerKind;
  open: boolean;
  loading?: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

export function ReShadeDisclaimerModal({
  kind,
  open,
  loading,
  onConfirm,
  onCancel,
}: Props) {
  const { t } = useTranslation("reshade");

  if (!open) return null;

  const title = t(`disclaimer.${kind}.title`);
  const body = t(`disclaimer.${kind}.body`);
  const confirm = t(`disclaimer.${kind}.confirm`);

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4 backdrop-blur-sm"
      role="dialog"
      aria-modal="true"
      aria-labelledby="reshade-disclaimer-title"
    >
      <div className="w-full max-w-lg rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-elevated)] p-6 shadow-2xl">
        <div className="flex items-start gap-3">
          <AlertTriangle className="mt-0.5 shrink-0 text-[#e8c468]" size={22} />
          <div>
            <h2
              id="reshade-disclaimer-title"
              className="text-lg font-semibold text-[var(--color-text)]"
            >
              {title}
            </h2>
            <p className="mt-3 text-sm leading-relaxed text-[var(--color-text-secondary)]">
              {body}
            </p>
          </div>
        </div>
        <div className="mt-6 flex justify-end gap-2">
          <Button variant="ghost" onClick={onCancel} disabled={loading}>
            {t("disclaimer.cancel")}
          </Button>
          <Button variant="primary" onClick={onConfirm} loading={loading}>
            {confirm}
          </Button>
        </div>
      </div>
    </div>
  );
}
