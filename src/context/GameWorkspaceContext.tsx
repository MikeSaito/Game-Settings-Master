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
import { getGameConfig } from "../lib/api";
import { formatPresetLabel, getLastPreset } from "../lib/lastPreset";
import type { AppTab, GameProfile } from "../lib/types";

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

function fallbackPreset(game: GameProfile, userOnly: boolean): WorkspacePreset {
  if (userOnly) {
    return { label: "Пользовательский", mode: "user" };
  }
  const last = getLastPreset(game.id);
  if (last) {
    return { label: formatPresetLabel(last.presetId), mode: "applied" };
  }
  return { label: "Настроен", mode: "applied" };
}

export function presetBadgeText(preset: WorkspacePreset): string {
  if (preset.mode === "user") return preset.label;
  if (preset.mode === "selected") return preset.label;
  return `Применён: ${preset.label}`;
}

interface ProviderProps {
  game: GameProfile;
  activeTab: AppTab;
  children: ReactNode;
}

export function GameWorkspaceProvider({ game, activeTab, children }: ProviderProps) {
  const [override, setOverride] = useState<WorkspacePreset | null>(null);
  const configDir = game.config_dir ?? "";

  const { data: gameConfig } = useQuery({
    queryKey: ["game-config", configDir],
    queryFn: () => getGameConfig(configDir),
    enabled: !!configDir,
    staleTime: 20_000,
  });

  const userOnly = detectUserOnly(gameConfig?.files);

  useEffect(() => {
    setOverride(null);
  }, [game.id, activeTab]);

  const preset = useMemo(
    () => override ?? fallbackPreset(game, userOnly),
    [override, game, userOnly],
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

export function useGameWorkspace() {
  const ctx = useContext(GameWorkspaceContext);
  if (!ctx) {
    throw new Error("useGameWorkspace вне GameWorkspaceProvider");
  }
  return ctx;
}

export function useWorkspacePreset(label: string, mode: PresetMode, enabled = true) {
  const ctx = useContext(GameWorkspaceContext);

  useEffect(() => {
    if (!enabled || !ctx) return;
    ctx.setWorkspacePreset({ label, mode });
    return () => ctx.setWorkspacePreset(null);
  }, [label, mode, enabled, ctx]);
}
