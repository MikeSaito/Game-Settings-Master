import { AlertTriangle } from "lucide-react";
import { Button } from "./ui/Button";

export type ReShadeDisclaimerKind = "enable" | "install" | "launch";

const copy: Record<
  ReShadeDisclaimerKind,
  { title: string; body: string; confirm: string }
> = {
  enable: {
    title: "Включить ReShade?",
    body:
      "ReShade — сторонний пост-обработчик (DLL-прокси в папке игры). " +
      "В онлайн-играх с античитом это может привести к блокировке аккаунта. " +
      "GSM не проверяет античит — вы принимаете риск самостоятельно.",
    confirm: "Понимаю риски, включить",
  },
  install: {
    title: "Установить ReShade в папку игры?",
    body:
      "GSM скопирует ReShade DLL, ReShade.ini и пресет в каталог exe. " +
      "Существующий proxy-DLL (dxgi.dll / d3d11.dll) будет сохранён в бэкап. " +
      "Закройте игру перед установкой.",
    confirm: "Установить",
  },
  launch: {
    title: "Запуск с ReShade",
    body:
      "Перед запуском GSM подготовит ReShade в папке игры. " +
      "Напоминание: использование ReShade в мультиплеере — на ваш страх и риск.",
    confirm: "Запустить",
  },
};

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
  if (!open) return null;

  const { title, body, confirm } = copy[kind];

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
            Отмена
          </Button>
          <Button variant="primary" onClick={onConfirm} loading={loading}>
            {confirm}
          </Button>
        </div>
      </div>
    </div>
  );
}
