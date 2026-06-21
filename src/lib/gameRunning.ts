/** Exe name for is_game_running: profile or basename from backend (resolve_game_exe_path). */
export function exeNameForRunningCheck(
  exeName: string | null | undefined,
  exePath: string | null | undefined,
): string | null | undefined {
  if (exeName) return exeName;
  if (!exePath) return undefined;
  const parts = exePath.split(/[/\\]/);
  return parts[parts.length - 1] || undefined;
}
