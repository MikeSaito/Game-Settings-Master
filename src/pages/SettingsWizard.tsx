import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { AlertTriangle, Check, Sparkles, Zap } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { BackupBanner } from "../components/BackupBanner";
import { IniDiffView } from "../components/IniDiffView";
import { PresetCard } from "../components/PresetCard";
import { Alert } from "../components/ui/Alert";
import { Badge } from "../components/ui/Badge";
import { Button } from "../components/ui/Button";
import { EmptyState } from "../components/ui/EmptyState";
import { PageHeader } from "../components/ui/PageHeader";
import { SectionHeader } from "../components/ui/SectionHeader";
import { useDebouncedValue } from "../hooks/useDebouncedValue";
import { useBackgroundSafeEnabled } from "../hooks/useBackgroundSafeEnabled";
import { applyPreset, listPresets, previewPreset } from "../lib/api";
import { useWorkspacePreset } from "../context/GameWorkspaceContext";
import { GameRunningAlert, useGameRunning } from "../hooks/useGameRunning";
import { useRunningExeName } from "../hooks/useRunningExeName";
import { usePresetSyncBanner } from "../hooks/usePresetSyncBanner";
import { formatInvokeError } from "../lib/errors";
import { formatPresetLabel, saveLastPreset } from "../lib/lastPreset";
import type { GameProfile } from "../lib/types";

interface Props {
  game: GameProfile | null;
}

function presetStepHint(
  presets: { id: string; name: string; description: string }[],
): string {
  if (presets.length === 0) {
    return "Пресеты появятся после синхронизации с сервером.";
  }
  const described = presets.filter((p) => p.description.trim());
  if (described.length >= 2) {
    const first = described[0];
    const last = described[described.length - 1];
    return `${first.name} — ${first.description.split(".")[0]}. · ${last.name} — ${last.description.split(".")[0]}.`;
  }
  if (described.length === 1) {
    return described[0].description;
  }
  return presets.map((p) => p.id).join(" · ");
}

export function SettingsWizard({ game }: Props) {
  const queryClient = useQueryClient();
  const activeGameIdRef = useRef(game?.id);
  activeGameIdRef.current = game?.id;
  const [selectedPresetId, setSelectedPresetId] = useState<string>("");
  const [lastBackupId, setLastBackupId] = useState<string>();
  const [successMessage, setSuccessMessage] = useState<string>();
  const [applyError, setApplyError] = useState<string>();
  const syncBanner = usePresetSyncBanner();

  const configDir = game?.config_dir ?? "";
  const engineFamily = game?.engine_family;
  const presetsEnabled = !!engineFamily && engineFamily !== "unknown";

  const {
    data: presets = [],
    isLoading: presetsLoading,
    isFetching: presetsFetching,
    error: presetsError,
  } = useQuery({
    queryKey: ["presets", engineFamily, game?.id],
    queryFn: () => listPresets(engineFamily, game?.id),
    enabled: presetsEnabled,
    staleTime: 10 * 60_000,
    refetchOnMount: false,
    retry: 1,
    placeholderData: (previousData, previousQuery) =>
      previousQuery?.queryKey?.[2] === game?.id ? previousData : undefined,
  });

  const activePresetId = selectedPresetId || presets[0]?.id || "";
  const previewPresetId = useDebouncedValue(activePresetId, 450);
  const previewEnabled = useBackgroundSafeEnabled(
    !!configDir && !!previewPresetId && !!game?.id,
  );

  const {
    data: diff = [],
    isFetching: diffLoading,
    error: previewError,
  } = useQuery({
    queryKey: ["preview", configDir, previewPresetId, game?.id, engineFamily],
    queryFn: () =>
      previewPreset(
        configDir,
        previewPresetId,
        game?.id,
        game?.install_dir,
        engineFamily,
      ),
    enabled: previewEnabled,
    staleTime: Infinity,
    refetchOnMount: false,
  });

  const runningExeName = useRunningExeName(game);
  const gameRunning = useGameRunning(runningExeName);

  useEffect(() => {
    setSelectedPresetId("");
    setLastBackupId(undefined);
    setSuccessMessage(undefined);
    setApplyError(undefined);
  }, [game?.id]);

  useWorkspacePreset(
    activePresetId ? formatPresetLabel(activePresetId) : "—",
    "selected",
    !!game?.config_dir && !!activePresetId,
  );

  const apply = useMutation({
    mutationFn: async () => {
      const snapshot = {
        gameId: game!.id,
        configDir,
        presetId: activePresetId,
      };
      const result = await applyPreset(
        snapshot.configDir,
        snapshot.presetId,
        snapshot.gameId,
        game!.install_dir,
        runningExeName ?? undefined,
        game!.engine_family,
      );
      return { result, snapshot };
    },
    onMutate: () => setApplyError(undefined),
    onSuccess: async ({ result, snapshot }) => {
      if (activeGameIdRef.current !== snapshot.gameId) return;
      setLastBackupId(result.backup_id);
      saveLastPreset(snapshot.gameId, snapshot.presetId);
      const filesLabel =
        result.changed_files.length > 0
          ? result.changed_files.join(", ")
          : "ini";
      const engineInPreview = diff.some((d) => d.file === "Engine.ini");
      const engineWritten = result.changed_files.includes("Engine.ini");
      const engineNote =
        engineInPreview && !engineWritten
          ? " Внимание: Engine.ini в предпросмотре был, но файл не записан — закройте игру и повторите."
          : "";
      if (result.diff.length === 0) {
        setSuccessMessage(
          "Запись выполнена, но изменений не обнаружено — значения уже совпадают с пресетом.",
        );
      } else {
        setSuccessMessage(
          `Пресет применён: ${filesLabel} (${result.diff.length} изменений). Перезапустите игру.${engineNote}`,
        );
      }
      queryClient.invalidateQueries({ queryKey: ["games"] });
      queryClient.invalidateQueries({ queryKey: ["game-config"] });
      queryClient.invalidateQueries({
        queryKey: ["backups", snapshot.configDir, snapshot.gameId],
      });
      queryClient.invalidateQueries({
        queryKey: ["parameters", snapshot.configDir, snapshot.gameId],
      });
      if (
        result.effective_config_dir &&
        result.effective_config_dir !== snapshot.configDir
      ) {
        queryClient.invalidateQueries({
          queryKey: ["preview", result.effective_config_dir],
        });
        queryClient.invalidateQueries({
          queryKey: ["parameters", result.effective_config_dir, snapshot.gameId],
        });
      }
      await queryClient.refetchQueries({
        queryKey: ["preview", snapshot.configDir],
      });
    },
    onError: (err) => setApplyError(formatInvokeError(err)),
  });

  if (!game) {
    return (
      <EmptyState
        icon={Sparkles}
        title="Выберите игру"
        description="Авторские пресеты доступны только для игр, разобранных автором (например Forza). Выберите такую игру в библиотеке."
      />
    );
  }

  if (!game.config_dir) {
    return (
      <Alert tone="warning" icon={AlertTriangle} title={`Config не найден — ${game.name}`}>
        {engineFamily === "forza"
          ? "Запустите Forza Horizon 6 один раз — нужен UserConfigSelections в AppData. Затем укажите папку ForzaUserConfigSelections в библиотеке."
          : "Укажите папку Saved/Config/Windows в библиотеке или запустите игру один раз для генерации ini."}
      </Alert>
    );
  }

  const previewMatchesSelection = previewPresetId === activePresetId;
  const canApply =
    presets.length > 0 &&
    !!activePresetId &&
    !diffLoading &&
    !previewError &&
    previewMatchesSelection &&
    !gameRunning;

  return (
    <div className="space-y-8">
      <PageHeader
        title="Авторские пресеты"
        meta={
          <>
            <Badge tone="accent">От автора</Badge>
            {engineFamily === "forza" && (
              <Badge tone="default">Forza Horizon 6</Badge>
            )}
          </>
        }
      />

      {syncBanner && (
        <Alert tone={syncBanner.tone} title={syncBanner.title}>
          {syncBanner.message}
        </Alert>
      )}

      {engineFamily === "forza" && (
        <Alert tone="info" title="Пресеты от автора приложения">
          Профили из мода FH6 Graphics Presets: UserConfigSelections + media-override.
          Разрешение, HDR и громкость не перезаписываются.
        </Alert>
      )}

      <GameRunningAlert exeName={runningExeName} gameName={game?.name} />

      {applyError && (
        <Alert tone="error" title="Ошибка применения">
          {applyError}
        </Alert>
      )}

      {successMessage && <BackupBanner backupId={lastBackupId} message={successMessage} />}

      {presetsError && (
        <Alert tone="error" title="Пресеты недоступны">
          {formatInvokeError(presetsError)}
        </Alert>
      )}

      {!presetsLoading && presets.length === 0 && !presetsError && (
        <Alert tone="warning" title="Нет пресетов">
          Пресеты не загружены. Проверьте интернет и перезапустите приложение.
        </Alert>
      )}

      <section>
        <SectionHeader
          step={1}
          title="Выберите пресет"
          hint={presetStepHint(presets)}
        />
        <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
          {presetsLoading || (presetsFetching && presets.length === 0) ? (
            <div className="col-span-full flex flex-col items-center gap-3 py-12">
              <span className="h-8 w-8 animate-spin rounded-full border-2 border-[var(--color-border)] border-t-[var(--color-accent)]" />
              <p className="text-sm text-muted">Загрузка пресетов…</p>
            </div>
          ) : (
            presets.map((preset) => (
              <PresetCard
                key={preset.id}
                preset={preset}
                selected={activePresetId === preset.id}
                onSelect={() => setSelectedPresetId(preset.id)}
              />
            ))
          )}
        </div>
      </section>

      <section>
        <SectionHeader
          step={2}
          title="Предпросмотр изменений"
          hint={
            activePresetId ? (
              <>{diff.length} правок</>
            ) : (
              "Выберите пресет"
            )
          }
        />
        {previewError && (
          <Alert tone="error" className="mb-4" title="Ошибка предпросмотра">
            {formatInvokeError(previewError)}
          </Alert>
        )}
        {engineFamily === "forza" && diff.some((d) => d.file.startsWith("media/")) && (
          <Alert tone="info" className="mb-4" title="Media-override">
            Пресет также перезапишет файлы в папке игры media/ — DefaultTrackSettings,
            routebudget, деревья вдали и GlobalCarAttributes.
          </Alert>
        )}
        <IniDiffView diff={diff} loading={diffLoading} />
      </section>

      <section className="flex flex-wrap items-center gap-4">
        <Button
          variant="primary"
          icon={<Zap size={18} />}
          onClick={() => apply.mutate()}
          loading={apply.isPending}
          disabled={!canApply || apply.isPending}
          className="!px-6 !py-3 text-base"
        >
          {apply.isPending ? "Применение…" : "Применить пресет"}
        </Button>
        {apply.isSuccess && (
          <span className="flex items-center gap-2 text-sm text-emerald-400">
            <Check size={18} /> Готово — перезапустите игру
          </span>
        )}
      </section>

    </div>
  );
}
