import { Download, Trash2, Upload, Zap } from "lucide-react";
import { useRef } from "react";
import { useTranslation } from "react-i18next";
import type { AdvancedEditorState } from "@/hooks/editor/useAdvancedEditorState";
import type { GameOverride } from "@/lib/core";
import { Button } from "@/components/ds/Button";

interface Props {
  state: AdvancedEditorState;
}

function downloadPresetJson(override: GameOverride) {
  const payload = {
    ...override,
    files: override.files,
    removals: override.removals ?? {},
  };
  const blob = new Blob([JSON.stringify(payload, null, 2)], {
    type: "application/json",
  });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = `${override.name.replace(/[^\w.-]+/g, "_") || "preset"}.uesm-preset.json`;
  anchor.click();
  URL.revokeObjectURL(url);
}

function parseImportedPreset(raw: string, gameId: string): GameOverride | null {
  const data = JSON.parse(raw) as Partial<GameOverride>;
  if (!data.name || typeof data.name !== "string" || !data.files) return null;
  return {
    game_id: gameId,
    name: data.name.trim(),
    files: data.files,
    removals: data.removals,
  };
}

export function SavedPresetsPanel({ state }: Props) {
  const { t } = useTranslation("advanced");
  const importRef = useRef<HTMLInputElement>(null);

  const onImportFile = async (file: File) => {
    if (!state.game?.id) return;
    try {
      const text = await file.text();
      const preset = parseImportedPreset(text, state.game.id);
      if (!preset) {
        state.setApplyError(t("presets.importInvalid"));
        return;
      }
      await state.importOverrideMutation.mutateAsync(preset);
    } catch {
      state.setApplyError(t("presets.importInvalid"));
    }
  };

  return (
    <section className="mt-3 rounded-[var(--radius-panel)] border border-[var(--color-border)] bg-[var(--color-surface)] p-3">
      <div className="mb-2 flex flex-wrap items-center justify-between gap-2">
        <h3 className="text-sm font-semibold text-[var(--color-text)]">{t("savedPresets")}</h3>
        <div className="flex gap-2">
          <Button
            variant="ghost"
            className="!py-1 !px-2 text-xs"
            icon={<Upload size={14} />}
            onClick={() => importRef.current?.click()}
            loading={state.importOverrideMutation.isPending}
          >
            {t("presets.import")}
          </Button>
          <input
            ref={importRef}
            type="file"
            accept="application/json,.json"
            className="hidden"
            onChange={(event) => {
              const file = event.target.files?.[0];
              event.target.value = "";
              if (file) void onImportFile(file);
            }}
          />
        </div>
      </div>
      <ul className="space-y-1.5">
        {state.overrides.length === 0 && (
          <li className="px-1 py-2 text-xs text-[var(--color-text-muted)]">{t("presets.empty")}</li>
        )}
        {state.overrides.map((override) => (
          <li
            key={`${override.game_id}-${override.name}`}
            className="flex items-center justify-between gap-3 rounded-lg border border-[var(--color-border)] px-3 py-2"
          >
            <span className="min-w-0 truncate text-sm text-[var(--color-text-secondary)]">
              {override.name}
            </span>
            <div className="flex shrink-0 gap-1">
              <Button
                variant="ghost"
                className="!py-1 !px-2 text-xs"
                icon={<Download size={14} />}
                onClick={() => downloadPresetJson(override)}
                title={t("presets.export")}
              >
                {t("presets.export")}
              </Button>
              <Button
                variant="secondary"
                className="!py-1 !px-2 text-xs"
                icon={<Zap size={14} />}
                onClick={() => state.applyOverrideMutation.mutate(override)}
                disabled={state.gameRunning}
                loading={state.applyOverrideMutation.isPending}
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
                className="rounded-lg p-1.5 text-[var(--color-text-muted)] transition hover:bg-[#2e1a1a] hover:text-[#f08080]"
                title={t("presets.delete")}
              >
                <Trash2 size={15} />
              </button>
            </div>
          </li>
        ))}
      </ul>
    </section>
  );
}
