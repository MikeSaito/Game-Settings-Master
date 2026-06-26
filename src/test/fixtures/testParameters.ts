import type { GameParameter } from "@/lib/core";

export const testParameters: GameParameter[] = Array.from({ length: 50 }, (_, i) => ({
  key: `r.TestCvar${i}`,
  section: "/Script/Engine.RendererSettings",
  file: "Scalability.ini",
  value: String(i % 4),
  title: `Parameter ${i}`,
  description: `Description for parameter ${i}`,
  impact: "Medium",
  category: "Scalability",
  min: "0",
  max: "4",
  in_game_label: null,
  value_hint: null,
  value_type: "int",
  known: true,
  editable: true,
  present_in_ini: true,
}));
