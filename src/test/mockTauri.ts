import { vi } from "vitest";
import { testGame } from "./fixtures/gameProfile";
import { testParameters } from "./fixtures/testParameters";

export type InvokeHandler = (args?: Record<string, unknown>) => unknown;

/** Per-command overrides; tests can mutate before render. */
export const mockInvokeHandlers: Record<string, InvokeHandler> = {
  scan_games: () => [testGame],
  get_game_parameters_cmd: () => testParameters,
  get_scalability_limits_cmd: () => ({
    groups: {},
    global_max: 4,
    sources: [],
  }),
  get_game_overrides: () => [],
  get_gpu_info_cmd: () => ({
    vendor: "nvidia",
    name: "Test GPU",
    vram_mb: 8192,
    dlss: true,
    dlss_fg: false,
    fsr: true,
    xe_ss: false,
  }),
  is_game_running_cmd: () => false,
  set_language_cmd: () => null,
};

export function createMockInvoke() {
  return vi.fn(async (cmd: string, args?: Record<string, unknown>) => {
    const handler = mockInvokeHandlers[cmd];
    if (handler) {
      return handler(args);
    }
    return null;
  });
}

export const mockInvoke = createMockInvoke();
