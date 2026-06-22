import type { GameProfile } from "@/lib/core";

const baseGame: GameProfile = {
  id: "test-game",
  name: "Test UE Game",
  source: "steam",
  install_dir: "C:\\Games\\Test",
  config_dir: "C:\\Games\\Test\\Saved\\Config\\Windows",
  exe_name: "TestGame.exe",
  is_ue: true,
  possible_ue: true,
  cover_url: null,
  custom_cover: null,
  build_id: null,
  engine_family: "ue5",
  engine_version: "5.4",
};

/** Default UE game for library / advanced editor tests. */
export const testGame: GameProfile = { ...baseGame };
