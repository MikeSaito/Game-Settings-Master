import { FolderOpen, Trash2, Wand2 } from "lucide-react";
import type { ReShadePageState } from "./useReShadePage";
import { Button } from "../ui/Button";

interface Props {
  page: ReShadePageState;
}

export function ReShadeWizardFooter({ page }: Props) {
  return (
    <section className="space-y-4">
      <div className="flex flex-wrap items-center gap-4">
        <Button
          variant="primary"
          icon={<Wand2 size={18} />}
          onClick={page.runPrimaryCta}
          loading={page.primaryCtaLoading}
          disabled={!page.canPrimaryCta}
          className="!px-6 !py-3 text-base"
        >
          {page.primaryCtaLoading ? "Применение…" : page.primaryCtaLabel}
        </Button>
        <Button
          variant="ghost"
          icon={<Trash2 size={16} />}
          loading={page.removeMutation.isPending}
          disabled={!page.canRemoveReShade}
          onClick={page.removeCurrentGameReShade}
        >
          Удалить
        </Button>
        <Button
          variant="ghost"
          icon={<FolderOpen size={16} />}
          onClick={() => page.openGameFolder()}
        >
          Открыть папку exe
        </Button>
      </div>
      <p className="text-sm text-muted">
        Запуск игры — кнопка <strong>Играть</strong> в шапке. В игре — <strong>Home</strong> для
        меню ReShade.
      </p>
      <Button
        variant="ghost"
        className="!px-0 text-sm"
        loading={page.launchSkipMutation.isPending}
        disabled={!page.canLaunchWithoutReShade}
        onClick={() => page.launchWithoutReShade()}
      >
        Играть без ReShade
      </Button>
    </section>
  );
}
