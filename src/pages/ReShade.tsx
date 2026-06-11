import { Palette } from "lucide-react";
import { ReShadeWizard } from "../components/reshade/ReShadeWizard";
import { EmptyState } from "../components/ui/EmptyState";
import type { GameProfile } from "../lib/types";

interface Props {
  game: GameProfile | null;
}

export function ReShade({ game }: Props) {
  if (!game) {
    return (
      <EmptyState
        icon={Palette}
        title="Выберите игру"
        description="ReShade настраивается для конкретной игры из библиотеки."
      />
    );
  }

  return <ReShadeWizard game={game} />;
}
