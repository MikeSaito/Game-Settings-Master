import { FolderOpen, Trash2, Wand2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { ReShadePageState } from "./useReShadePage";
import { Button } from "../ui/Button";

interface Props {
  page: ReShadePageState;
}

export function ReShadeWizardFooter({ page }: Props) {
  const { t } = useTranslation("reshade");

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
          {page.primaryCtaLoading ? t("footer.applying") : page.primaryCtaLabel}
        </Button>
        <Button
          variant="ghost"
          icon={<Trash2 size={16} />}
          loading={page.removeMutation.isPending}
          disabled={!page.canRemoveReShade}
          onClick={page.removeCurrentGameReShade}
        >
          {t("footer.remove")}
        </Button>
        <Button
          variant="ghost"
          icon={<FolderOpen size={16} />}
          onClick={() => page.openGameFolder()}
        >
          {t("footer.openExeFolder")}
        </Button>
      </div>
      <p className="text-sm text-muted">
        {t("footer.launchHintBefore")}{" "}
        <strong>{t("footer.playButton")}</strong> {t("footer.launchHintMiddle")}{" "}
        <strong>Home</strong> {t("footer.launchHintAfter")}
      </p>
    </section>
  );
}
