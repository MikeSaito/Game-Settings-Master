export const GUS = "GameUserSettings.ini";
export const ENGINE = "Engine.ini";

export const DLSS_MODE_TO_NUM: Record<string, string> = {
  Off: "0",
  Performance: "1",
  Balanced: "2",
  Quality: "3",
  UltraQuality: "4",
  DLAA: "5",
};

export const DLSS_NUM_TO_MODE: Record<string, string> = {
  "0": "Off",
  "1": "Performance",
  "2": "Balanced",
  "3": "Quality",
  "4": "UltraQuality",
  "5": "DLAA",
};

export const DLSS_MODE_TO_SCALE: Record<string, string> = {
  Off: "1.0",
  Performance: "0.5",
  Balanced: "0.58",
  Quality: "0.66",
  UltraQuality: "0.77",
  DLAA: "1.0",
};
