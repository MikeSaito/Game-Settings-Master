import { useQuery } from "@tanstack/react-query";
import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { useTranslation } from "react-i18next";
import i18n from "../i18n";
import { useBackgroundSafeEnabled } from "@/hooks/app/useBackgroundSafeEnabled";
import { getGameConfig } from "@/lib/api";
import { formatPresetLabel, getLastPreset } from "@/lib/editor";
import type { AppTab, GameProfile } from "@/lib/core";

const OVERRIDE_INI = ["Engine.ini", "Game.ini", "Scalability.ini", "Input.ini"];

export type PresetMode = "user" | "selected" | "applied";

export interface WorkspacePreset {
  label: string;
  mode: PresetMode;
}

interface GameWorkspaceContextValue {
  preset: WorkspacePreset;
  setWorkspacePreset: (preset: WorkspacePreset | null) => void;
}

const GameWorkspaceContext = createContext<GameWorkspaceContextValue | null>(null);

function detectUserOnly(files: Record<string, unknown> | undefined): boolean {
  if (!files) return false;
  return !OVERRIDE_INI.some((f) => f in files);
}

interface ProviderProps {
  game: GameProfile;
  activeTab: AppTab;
  children: ReactNode;
}

export function GameWorkspaceProvider({ game, activeTab, children }: ProviderProps) {
  const { t } = useTranslation("common");
  const [override, setOverride] = useState<WorkspacePreset | null>(null);
  const configDir = game.config_dir ?? "";
  const isGameTab = activeTab !== "library";
  const queriesEnabled = useBackgroundSafeEnabled(!!configDir && isGameTab);

  const { data: gameConfig } = useQuery({
    queryKey: ["game-config", configDir, game.id, game.engine_family],
    queryFn: () => getGameConfig(configDir, game.id, game.engine_family),
    enabled: queriesEnabled,
    staleTime: 5 * 60_000,
    refetchOnMount: false,
  });

  const userOnly = detectUserOnly(gameConfig?.files);

  const fallbackPreset = useCallback(
    (userOnlyMode: boolean): WorkspacePreset => {
      if (userOnlyMode) {
        return { label: t("workspaceUser"), mode: "user" };
      }
      const last = getLastPreset(game.id);
      if (last) {
        return { label: formatPresetLabel(last.presetId), mode: "applied" };
      }
      return { label: t("workspaceApplied"), mode: "applied" };
    },
    [game.id, t],
  );

  useEffect(() => {
    setOverride(null);
  }, [game.id, activeTab]);

  const preset = useMemo(
    () => override ?? fallbackPreset(userOnly),
    [override, userOnly, fallbackPreset],
  );

  const setWorkspacePreset = useCallback((next: WorkspacePreset | null) => {
    setOverride(next);
  }, []);

  const value = useMemo(
    () => ({ preset, setWorkspacePreset }),
    [preset, setWorkspacePreset],
  );

  return (
    <GameWorkspaceContext.Provider value={value}>{children}</GameWorkspaceContext.Provider>
  );
}

export function presetBadgeText(preset: WorkspacePreset): string {
  if (preset.mode === "user") return preset.label;
  if (preset.mode === "selected") return preset.label;
  return i18n.t("common:workspaceAppliedPrefix", { label: preset.label });
}

export function useGameWorkspace() {
  const ctx = useContext(GameWorkspaceContext);
  if (!ctx) {
    throw new Error("useGameWorkspace outside GameWorkspaceProvider");
  }
  return ctx;
}

export function useWorkspacePreset(label: string, mode: PresetMode, enabled = true) {
  const setWorkspacePreset = useContext(GameWorkspaceContext)?.setWorkspacePreset;

  useEffect(() => {
    if (!enabled || !setWorkspacePreset) return;
    setWorkspacePreset({ label, mode });
    return () => setWorkspacePreset(null);
  }, [label, mode, enabled, setWorkspacePreset]);
}
