import { Save, Trash2, Zap } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "../ui/Button";
import { Card } from "../ui/Card";
import { Input } from "../ui/Input";
import { SectionHeader } from "../ui/SectionHeader";
import type { AdvancedEditorState } from "@/hooks/editor/useAdvancedEditorState";

interface Props {
  state: AdvancedEditorState;
}

export function AdvancedEditorFooter({ state }: Props) {
  const { t } = useTranslation("advanced");

  return (
    <>
      <Card padding="md">
        <SectionHeader title={t("applyAndSave")} />
        <div className="flex flex-wrap items-end gap-3">
          <div className="min-w-[200px] flex-1">
            <Input
              label={t("presetNameLabel")}
              value={state.overrideName}
              onChange={(e) => state.setOverrideName(e.target.value)}
            />
          </div>
          <Button
            variant="primary"
            icon={<Zap size={16} />}
            onClick={() => state.applyCustomMutation.mutate()}
            loading={state.applyCustomMutation.isPending}
            disabled={state.gameRunning}
          >
            {t("apply")}
          </Button>
          <Button
            variant="secondary"
            icon={<Save size={16} />}
            onClick={() => state.saveOverrideMutation.mutate()}
            loading={state.saveOverrideMutation.isPending}
          >
            {t("savePreset")}
          </Button>
        </div>
      </Card>

      {state.overrides.length > 0 && (
        <section>
          <SectionHeader title={t("savedPresets")} />
          <div className="space-y-2">
            {state.overrides.map((override) => (
              <Card key={`${override.game_id}-${override.name}`} padding="sm" className="!p-0">
                <div className="flex items-center justify-between gap-4 px-4 py-3">
                  <span className="font-medium text-[var(--color-text-secondary)]">
                    {override.name}
                  </span>
                  <div className="flex gap-2">
                    <Button
                      variant="primary"
                      className="!py-1.5 !px-3 text-xs"
                      onClick={() => state.applyOverrideMutation.mutate(override)}
                      disabled={state.gameRunning}
                    >
                      {t("apply")}
                    </Button>
                    <button
                      type="button"
                      onClick={() =>
                        state.deleteOverrideMutation.mutate({
                          gameId: override.game_id,
                          name: override.name,
                        })
                      }
                      className="rounded-lg p-2 text-muted transition hover:bg-[#2e1a1a] hover:text-[#f08080]"
                    >
                      <Trash2 size={16} />
                    </button>
                  </div>
                </div>
              </Card>
            ))}
          </div>
        </section>
      )}
    </>
  );
}
