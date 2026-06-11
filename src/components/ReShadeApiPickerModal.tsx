import { Monitor } from "lucide-react";
import { useEffect, useState } from "react";
import type { GraphicsApiInfo } from "../lib/types";
import { Button } from "./ui/Button";

interface Props {
  open: boolean;
  apis: GraphicsApiInfo[];
  initialApi?: string | null;
  rememberDefault?: boolean;
  loading?: boolean;
  title?: string;
  onConfirm: (api: string, remember: boolean) => void;
  onCancel: () => void;
}

export function ReShadeApiPickerModal({
  open,
  apis,
  initialApi,
  rememberDefault = true,
  loading,
  title = "Какой графический API использует игра?",
  onConfirm,
  onCancel,
}: Props) {
  const [selected, setSelected] = useState(initialApi ?? apis[0]?.id ?? "dx12");
  const [remember, setRemember] = useState(rememberDefault);

  useEffect(() => {
    if (open) {
      setSelected(initialApi ?? apis[0]?.id ?? "dx12");
      setRemember(rememberDefault);
    }
  }, [open, initialApi, apis, rememberDefault]);

  if (!open) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4 backdrop-blur-sm"
      role="dialog"
      aria-modal="true"
      aria-labelledby="reshade-api-title"
      onClick={onCancel}
    >
      <div
        className="max-h-[90vh] w-full max-w-xl overflow-y-auto rounded-xl border border-[var(--color-border)] bg-[var(--color-bg-elevated)] p-6 shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-start gap-3">
          <Monitor className="mt-0.5 shrink-0 text-accent" size={22} />
          <div className="min-w-0 flex-1">
            <h2
              id="reshade-api-title"
              className="text-lg font-semibold text-[var(--color-text)]"
            >
              {title}
            </h2>
            <p className="mt-2 text-sm leading-relaxed text-[var(--color-text-secondary)]">
              GSM не определяет API автоматически — выберите вручную, как в установщике ReShade.
              Не знаете? Посмотрите настройки графики игры: обычно UE5 = DX12, старые UE4 = DX11.
            </p>
          </div>
        </div>

        <div className="mt-5 space-y-2">
          {apis.map((api) => {
            const active = selected === api.id;
            return (
              <button
                key={api.id}
                type="button"
                onClick={() => setSelected(api.id)}
                className={
                  active
                    ? "w-full rounded-lg border border-[var(--color-accent)] bg-[var(--color-accent-soft)] p-3 text-left"
                    : "w-full rounded-lg border border-[var(--color-border)] p-3 text-left hover:border-[var(--color-border-strong)]"
                }
              >
                <div className="text-sm font-medium text-[var(--color-text)]">{api.name}</div>
                <div className="mt-1 text-xs text-muted">{api.description}</div>
                <div className="mt-1 font-mono text-xs text-[var(--color-text-secondary)]">
                  {api.files.join(", ")}
                </div>
              </button>
            );
          })}
        </div>

        <label className="mt-4 flex cursor-pointer items-center gap-2 text-sm text-[var(--color-text-secondary)]">
          <input
            type="checkbox"
            checked={remember}
            onChange={(e) => setRemember(e.target.checked)}
            className="rounded border-[var(--color-border)]"
          />
          Запомнить для этой игры
        </label>

        <div className="mt-6 flex justify-end gap-2">
          <Button variant="ghost" onClick={onCancel} disabled={loading}>
            Отмена
          </Button>
          <Button
            variant="primary"
            onClick={() => onConfirm(selected, remember)}
            loading={loading}
            disabled={!selected}
          >
            Продолжить
          </Button>
        </div>
      </div>
    </div>
  );
}
