import { Check } from "lucide-react";

interface Props {
  message: string;
}

export function ReShadeSuccessBar({ message }: Props) {
  return (
    <div className="flex items-center gap-2 rounded-xl border border-emerald-500/30 bg-emerald-500/10 px-4 py-3 text-sm text-emerald-400">
      <Check size={18} className="shrink-0" />
      <span>{message}</span>
    </div>
  );
}
