import { Palette } from "lucide-react";
import { useTranslation } from "react-i18next";
import { ReShadeWizard } from "../components/reshade/ReShadeWizard";
import { EmptyState } from "../components/ui/EmptyState";
import type { GameProfile } from "../lib/types";

interface Props {
  game: GameProfile | null;
}

export function ReShade({ game }: Props) {
  const { t } = useTranslation("reshade");
  if (!game) {
    return (
      <EmptyState
        icon={Palette}
        title={t("emptyState.title")}
        description={t("emptyState.description")}
      />
    );
  }

  return <ReShadeWizard game={game} />;
}
