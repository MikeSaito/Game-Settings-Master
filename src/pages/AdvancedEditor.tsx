import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Save, Search, SlidersHorizontal, Trash2, Zap } from "lucide-react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { currentLanguage } from "../i18n";
import { useAppWindowFocused } from "../context/AppWindowFocusProvider";
import { BackupBanner } from "../components/BackupBanner";
import { ParameterCard } from "../components/ParameterCard";
import { Alert } from "../components/ui/Alert";
import { Badge } from "../components/ui/Badge";
import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { EmptyState } from "../components/ui/EmptyState";
import { Input } from "../components/ui/Input";
import { PageHeader } from "../components/ui/PageHeader";
import { SectionHeader } from "../components/ui/SectionHeader";
import { useWorkspacePreset } from "../context/GameWorkspaceContext";
import { useBackgroundSafeEnabled } from "../hooks/useBackgroundSafeEnabled";
import { GameRunningAlert, useGameRunning } from "../hooks/useGameRunning";
import { useRunningExeName } from "../hooks/useRunningExeName";
import {
  applyCustom,
  getGameOverrides,
  getGameParameters,
  getGpuInfo,
  getScalabilityLimits,
  saveGameOverride,
  deleteGameOverride,
  applyGameOverride,
} from "../lib/api";
import {
  filterSelectOptions,
  gpuFilterHint,
  isParamVisible,
} from "../lib/gpuCompat";
import {
  applyParamDependencies,
  getDependencyLabel,
} from "../lib/paramDependencies";
import { cn } from "../lib/cn";
import { buildCustomChanges } from "../lib/buildCustomChanges";
import {
  defaultValueFor,
  ENGINE_CATEGORIES,
  engineParamId,
  initialEngineEnabledKeys,
  isEngineEnabled,
  isEngineToggleable,
  resolveEditableCategories,
} from "../lib/engineParams";
import { formatInvokeError } from "../lib/errors";
import type { GameParameter, GameProfile } from "../lib/types";

interface Props {
  game: GameProfile | null;
}

const CATEGORY_ORDER = [
  "Scalability",
  "Graphics",
  "Display",
  "API",
  "Jobs",
  "Rendering",
  "Shadows",
  "Textures",
  "PostProcess",
  "Mirrors",
  "LOD",
  "World",
  "Media",
  "Startup",
  "AuthorCurated",
  "GameSpecific",
  "Audio",
  "System",
  "Debug",
  "Other",
] as const;

const UNITY_EDITABLE = new Set([
  "Graphics",
  "Display",
  "API",
  "Jobs",
  "Startup",
  "System",
]);

const EDITABLE_FOR_APPLY = new Set([
  "Scalability",
  "Rendering",
  "Shadows",
  "Textures",
  "PostProcess",
  "Display",
  "GameSpecific",
  "AuthorCurated",
  "Audio",
]);

export function AdvancedEditor({ game }: Props) {
  const { t } = useTranslation("advanced");
  const queryClient = useQueryClient();
  const configDir = game?.config_dir ?? "";
  const isUnity = game?.is_unity || game?.engine_family === "unity";
  const isForza = game?.engine_family === "forza";
  const runningExeName = useRunningExeName(game);
  const gameRunning = useGameRunning(runningExeName);
  const queriesEnabled = useBackgroundSafeEnabled(!!configDir && !!game?.id);
  const overridesEnabled = useBackgroundSafeEnabled(!!game?.id);
  const gpuEnabled = useBackgroundSafeEnabled();

  useWorkspacePreset(t("title"), "selected", !!configDir);
  const [params, setParams] = useState<GameParameter[]>([]);
  const paramsDirtyRef = useRef(false);
  const activeGameIdRef = useRef(game?.id);
  activeGameIdRef.current = game?.id;
  const [overrideName, setOverrideName] = useState(t("defaultPresetName"));
  const [message, setMessage] = useState<string>();
  const [applyError, setApplyError] = useState<string>();
  const [activeCategory, setActiveCategory] = useState<string>(
    isForza ? "Graphics" : isUnity ? "Graphics" : "Scalability",
  );
  const [search, setSearch] = useState("");
  const [engineEnabled, setEngineEnabled] = useState<Set<string>>(new Set());

  useEffect(() => {
    setMessage(undefined);
    setApplyError(undefined);
    paramsDirtyRef.current = false;
  }, [game?.id]);

  const { data: parameters = [], isLoading, isFetching } = useQuery({
    queryKey: ["parameters", configDir, game?.id, game?.engine_family, currentLanguage()],
    queryFn: () =>
      getGameParameters(
        configDir,
        game?.id,
        game?.install_dir,
        game?.engine_family,
      ),
    enabled: queriesEnabled,
    staleTime: 5 * 60_000,
    refetchOnMount: false,
    placeholderData: (previousData, previousQuery) =>
      previousQuery?.queryKey?.[2] === game?.id ? previousData : undefined,
  });

  const parametersLoading = (isLoading || isFetching) && parameters.length === 0;

  const { data: limits } = useQuery({
    queryKey: ["limits", configDir, game?.install_dir, game?.id],
    queryFn: () => getScalabilityLimits(configDir, game!.install_dir, game!.id),
    enabled: queriesEnabled && !!game,
  });

  const { data: overrides = [] } = useQuery({
    queryKey: ["overrides", game?.id],
    queryFn: () => getGameOverrides(game!.id),
    enabled: overridesEnabled,
  });

  const { data: gpu } = useQuery({
    queryKey: ["gpu"],
    queryFn: getGpuInfo,
    enabled: gpuEnabled,
    staleTime: 300_000,
  });

  // Re-read values from disk when the game may have changed them:
  // on game exit and when the app window regains focus.
  const windowFocused = useAppWindowFocused();
  const refreshFromDisk = useCallback(() => {
    if (!game?.id || !configDir) return;
    // Do not overwrite unsaved user edits.
    if (paramsDirtyRef.current) return;
    void queryClient.invalidateQueries({
      queryKey: ["parameters", configDir, game.id],
    });
    void queryClient.invalidateQueries({
      queryKey: ["limits", configDir, game.install_dir, game.id],
    });
    void queryClient.invalidateQueries({ queryKey: ["game-config"] });
  }, [queryClient, configDir, game?.id, game?.install_dir]);

  const prevRunningRef = useRef(gameRunning);
  useEffect(() => {
    if (prevRunningRef.current && !gameRunning) {
      refreshFromDisk();
    }
    prevRunningRef.current = gameRunning;
  }, [gameRunning, refreshFromDisk]);

  const prevFocusedRef = useRef(windowFocused);
  useEffect(() => {
    if (!prevFocusedRef.current && windowFocused) {
      refreshFromDisk();
    }
    prevFocusedRef.current = windowFocused;
  }, [windowFocused, refreshFromDisk]);

  const visibleParams = useMemo(
    () => params.filter((p) => isParamVisible(p, gpu)),
    [params, gpu],
  );

  useEffect(() => {
    if (paramsDirtyRef.current) return;
    setParams(parameters);
    setEngineEnabled(initialEngineEnabledKeys(parameters));
    if (isForza) {
      setActiveCategory("Graphics");
    } else if (isUnity) {
      setActiveCategory("Graphics");
    }
  }, [parameters, isUnity, isForza]);

  const categories = useMemo(() => {
    const counts = new Map<string, number>();
    for (const p of visibleParams) {
      counts.set(p.category, (counts.get(p.category) ?? 0) + 1);
    }
    const ordered = CATEGORY_ORDER.filter((c) => counts.has(c));
    for (const c of counts.keys()) {
      if (!ordered.includes(c as (typeof CATEGORY_ORDER)[number])) {
        ordered.push(c as (typeof CATEGORY_ORDER)[number]);
      }
    }
    return ordered.map((cat) => ({ cat, count: counts.get(cat) ?? 0 }));
  }, [visibleParams, game?.id]);

  useEffect(() => {
    if (categories.length && !categories.some((c) => c.cat === activeCategory)) {
      setActiveCategory(categories[0].cat);
    }
  }, [categories, activeCategory]);

  const filteredParams = useMemo(() => {
    const q = search.trim().toLowerCase();
    const list = visibleParams.filter((p) => {
      if (p.category !== activeCategory) return false;
      if (!q) return true;
      return (
        p.key.toLowerCase().includes(q) ||
        p.title.toLowerCase().includes(q) ||
        p.description.toLowerCase().includes(q) ||
        (p.value_hint?.toLowerCase().includes(q) ?? false)
      );
    });

    if (!ENGINE_CATEGORIES.has(activeCategory)) {
      return list;
    }

    return [...list].sort((a, b) => {
      const aOn = isEngineEnabled(a, engineEnabled) ? 0 : 1;
      const bOn = isEngineEnabled(b, engineEnabled) ? 0 : 1;
      if (aOn !== bOn) return aOn - bOn;
      return a.title.localeCompare(b.title, "ru");
    });
  }, [visibleParams, activeCategory, search, game?.id, engineEnabled]);

  const engineStats = useMemo(() => {
    const engine = visibleParams.filter(
      (p) => p.file === "Engine.ini" && isEngineToggleable(p),
    );
    const on = engine.filter((p) => isEngineEnabled(p, engineEnabled)).length;
    return { total: engine.length, on, off: engine.length - on };
  }, [visibleParams, engineEnabled]);

  const catalogStats = useMemo(() => {
    const known = visibleParams.filter((p) => p.known).length;
    return {
      known,
      unknown: visibleParams.length - known,
      total: visibleParams.length,
    };
  }, [visibleParams]);

  const gpuHint = gpu ? gpuFilterHint(gpu) : null;

  const updateParam = (key: string, section: string, file: string, value: string) => {
    paramsDirtyRef.current = true;
    setParams((prev) =>
      applyParamDependencies(
        prev,
        { key, section, file, value },
        gpu,
      ),
    );
  };

  const toggleEngineParam = (p: GameParameter, enabled: boolean) => {
    paramsDirtyRef.current = true;
    const id = engineParamId(p);
    setEngineEnabled((prev) => {
      const next = new Set(prev);
      if (enabled) next.add(id);
      else next.delete(id);
      return next;
    });
    if (enabled && !p.value.trim()) {
      updateParam(p.key, p.section, p.file, defaultValueFor(p));
    }
  };

  const forzaCategories = useMemo(
    () =>
      new Set([
        "Graphics",
        "Display",
        "Rendering",
        "Shadows",
        "Textures",
        "PostProcess",
        "Audio",
        "System",
        "AuthorCurated",
      ]),
    [],
  );

  const editableCategories = useMemo(() => {
    if (isForza) {
      return resolveEditableCategories(parameters, forzaCategories, forzaCategories);
    }
    const base = isUnity ? UNITY_EDITABLE : EDITABLE_FOR_APPLY;
    return resolveEditableCategories(parameters, base);
  }, [parameters, isForza, isUnity, forzaCategories]);

  const buildChanges = () =>
    buildCustomChanges(params, parameters, gpu, engineEnabled, editableCategories);

  const applyCustomMutation = useMutation({
    mutationFn: async () => {
      const snapshot = { gameId: game!.id, configDir };
      const { files, removals } = buildChanges();
      if (
        Object.keys(files).length === 0 &&
        Object.keys(removals).length === 0
      ) {
        throw new Error(
          isUnity ? t("errors.noChangesUnity") : t("errors.noChanges"),
        );
      }
      const result = await applyCustom(
        snapshot.configDir,
        files,
        runningExeName ?? undefined,
        removals,
        snapshot.gameId,
        game?.engine_family,
      );
      return { result, snapshot };
    },
    onMutate: () => setApplyError(undefined),
    onSuccess: ({ result, snapshot }) => {
      if (activeGameIdRef.current !== snapshot.gameId) return;
      paramsDirtyRef.current = false;
      setMessage(
        t("applied", {
          count: result.diff.length,
          backupId: result.backup_id,
        }),
      );
      queryClient.invalidateQueries({
        queryKey: ["backups", snapshot.configDir, snapshot.gameId],
      });
      queryClient.invalidateQueries({
        queryKey: ["parameters", snapshot.configDir, snapshot.gameId],
      });
      queryClient.invalidateQueries({ queryKey: ["game-config"] });
    },
    onError: (err) => setApplyError(formatInvokeError(err)),
  });

  const saveOverrideMutation = useMutation({
    mutationFn: async () => {
      const snapshot = { gameId: game!.id, name: overrideName };
      const { files, removals } = buildChanges();
      await saveGameOverride({
        game_id: snapshot.gameId,
        name: snapshot.name,
        files,
        removals,
      });
      return snapshot;
    },
    onSuccess: (snapshot) => {
      if (activeGameIdRef.current !== snapshot.gameId) return;
      queryClient.invalidateQueries({ queryKey: ["overrides", snapshot.gameId] });
      setMessage(t("presetSaved", { name: snapshot.name }));
    },
    onError: (err) => setApplyError(formatInvokeError(err)),
  });

  const applyOverrideMutation = useMutation({
    mutationFn: async (override: (typeof overrides)[0]) => {
      const snapshot = { gameId: game!.id, configDir };
      const result = await applyGameOverride(
        snapshot.configDir,
        override,
        runningExeName ?? undefined,
      );
      return { result, snapshot };
    },
    onSuccess: ({ result, snapshot }) => {
      if (activeGameIdRef.current !== snapshot.gameId) return;
      setMessage(t("presetApplied", { backupId: result.backup_id }));
      queryClient.invalidateQueries({
        queryKey: ["backups", snapshot.configDir, snapshot.gameId],
      });
      queryClient.invalidateQueries({
        queryKey: ["parameters", snapshot.configDir, snapshot.gameId],
      });
      queryClient.invalidateQueries({ queryKey: ["game-config"] });
    },
    onError: (err) => setApplyError(formatInvokeError(err)),
  });

  const deleteOverrideMutation = useMutation({
    mutationFn: ({ gameId, name }: { gameId: string; name: string }) =>
      deleteGameOverride(gameId, name),
    onSuccess: (_result, variables) => {
      if (activeGameIdRef.current !== variables.gameId) return;
      queryClient.invalidateQueries({ queryKey: ["overrides", variables.gameId] });
    },
    onError: (err) => setApplyError(formatInvokeError(err)),
  });

  if (!game) {
    return (
      <EmptyState
        icon={SlidersHorizontal}
        title={t("noGame.title")}
        description={t("noGame.desc")}
      />
    );
  }

  if (!configDir) {
    return (
      <Alert tone="warning" title={t("noConfig.title")}>
        {isUnity ? t("noConfig.unity") : t("noConfig.default")}
      </Alert>
    );
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title={t("title")}
        meta={
          <>
            <Badge tone="default">
              {t("paramsCount", { count: catalogStats.total })}
            </Badge>
            <Badge tone="success">
              {t("knownCount", { count: catalogStats.known })}
            </Badge>
            {catalogStats.unknown > 0 && (
              <Badge tone="warning">
                {t("unknownCount", { count: catalogStats.unknown })}
              </Badge>
            )}
            {limits && (
              <Badge tone="info">sg max {limits.global_max} · scale 25–200%</Badge>
            )}
          </>
        }
      />

      {gpuHint && (
        <Alert tone="info" title={t("gpuHintTitle")}>
          {gpuHint}
        </Alert>
      )}

      <GameRunningAlert exeName={runningExeName} gameName={game?.name} />

      {message && <BackupBanner message={message} />}
      {applyError && (
        <Alert tone="error" title={t("errorTitle")}>
          {applyError}
        </Alert>
      )}

      <Input
        icon={<Search size={16} />}
        placeholder={t("searchPlaceholder")}
        value={search}
        onChange={(e) => setSearch(e.target.value)}
      />

      <div className="flex flex-wrap gap-1.5">
        {categories.map(({ cat, count }) => (
          <button
            key={cat}
            type="button"
            onClick={() => setActiveCategory(cat)}
            className={cn(
              "rounded-xl px-3 py-2 text-sm font-medium transition",
              activeCategory === cat
                ? "bg-[var(--color-bg-active)] text-[var(--color-text)] ring-1 ring-[var(--color-accent)]/40"
                : "text-muted hover:bg-[var(--color-bg-hover)] hover:text-[var(--color-text-secondary)]",
            )}
          >
            {t(`category.${cat}`, { defaultValue: cat })}
            <span className="ml-1.5 text-xs opacity-60">{count}</span>
          </button>
        ))}
      </div>

      {activeCategory === "AuthorCurated" && (
        <Alert tone="info" title={t("authorCurated.title")}>
          {t("authorCurated.body")}
        </Alert>
      )}

      {isForza && (
        <Alert tone="info" title="Forza Horizon 6 — UserConfigSelections">
          {t("forza.before")}
          <code className="text-xs">media/</code>
          {t("forza.after")}
        </Alert>
      )}

      {!isForza && ENGINE_CATEGORIES.has(activeCategory) && (
        <Alert tone="info" title={t("engineIni.title")}>
          {t("engineIni.before")}
          <strong>{t("engineIni.onOff")}</strong>
          {t("engineIni.after", {
            on: engineStats.on,
            total: engineStats.total,
          })}
        </Alert>
      )}

      {parametersLoading ? (
        <div className="flex flex-col items-center gap-3 py-16">
          <span className="h-8 w-8 animate-spin rounded-full border-2 border-[var(--color-border)] border-t-[var(--color-accent)]" />
          <p className="text-sm text-body">{t("loadingParams")}</p>
        </div>
      ) : filteredParams.length === 0 ? (
        <EmptyState
          icon={Search}
          title={search ? t("emptyFiltered.titleSearch") : t("emptyFiltered.titleEmpty")}
          description={
            search ? t("emptyFiltered.descSearch") : t("emptyFiltered.descEmpty")
          }
          className="py-12"
        />
      ) : (
        <div className="flex flex-col gap-3">
          {filteredParams.map((param) => {
            const toggleable = isEngineToggleable(param);
            const enabled = isEngineEnabled(param, engineEnabled);
            return (
              <ParameterCard
                key={`${param.file}-${param.section}-${param.key}`}
                param={param}
                editable={param.editable && enabled}
                engineToggleable={toggleable}
                engineEnabled={enabled}
                selectOptions={filterSelectOptions(param, gpu) ?? undefined}
                dependencyLabel={getDependencyLabel(param.key) ?? undefined}
                onEngineToggle={(on) => toggleEngineParam(param, on)}
                onChange={
                  param.editable && enabled
                    ? (value) =>
                        updateParam(param.key, param.section, param.file, value)
                    : undefined
                }
              />
            );
          })}
        </div>
      )}

      <Card padding="md">
        <SectionHeader title={t("applyAndSave")} />
        <div className="flex flex-wrap items-end gap-3">
          <div className="min-w-[200px] flex-1">
            <Input
              label={t("presetNameLabel")}
              value={overrideName}
              onChange={(e) => setOverrideName(e.target.value)}
            />
          </div>
          <Button
            variant="primary"
            icon={<Zap size={16} />}
            onClick={() => applyCustomMutation.mutate()}
            loading={applyCustomMutation.isPending}
            disabled={gameRunning}
          >
            {t("apply")}
          </Button>
          <Button
            variant="secondary"
            icon={<Save size={16} />}
            onClick={() => saveOverrideMutation.mutate()}
            loading={saveOverrideMutation.isPending}
          >
            {t("savePreset")}
          </Button>
        </div>
      </Card>

      {overrides.length > 0 && (
        <section>
          <SectionHeader title={t("savedPresets")} />
          <div className="space-y-2">
            {overrides.map((override) => (
              <Card key={`${override.game_id}-${override.name}`} padding="sm" className="!p-0">
                <div className="flex items-center justify-between gap-4 px-4 py-3">
                  <span className="font-medium text-[var(--color-text-secondary)]">{override.name}</span>
                  <div className="flex gap-2">
                    <Button
                      variant="primary"
                      className="!py-1.5 !px-3 text-xs"
                      onClick={() => applyOverrideMutation.mutate(override)}
                      disabled={gameRunning}
                    >
                      {t("apply")}
                    </Button>
                    <button
                      type="button"
                      onClick={() =>
                        deleteOverrideMutation.mutate({
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
    </div>
  );
}
