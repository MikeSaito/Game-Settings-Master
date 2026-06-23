export function countPendingChanges(
  files: Record<string, Record<string, Record<string, string>>>,
  removals: Record<string, Record<string, string[]>>,
) {
  const breakdown = { sg: 0, display: 0, engine: 0 };
  const pendingKeys = new Set<string>();
  let total = 0;

  for (const [file, sections] of Object.entries(files)) {
    for (const entries of Object.values(sections)) {
      for (const key of Object.keys(entries)) {
        pendingKeys.add(key);
        total += 1;
        if (key.startsWith("sg.")) breakdown.sg += 1;
        else if (file === "GameUserSettings.ini") breakdown.display += 1;
        else breakdown.engine += 1;
      }
    }
  }

  for (const [file, sections] of Object.entries(removals)) {
    for (const keys of Object.values(sections)) {
      for (const key of keys) {
        pendingKeys.add(key);
        total += 1;
        if (key.startsWith("sg.")) breakdown.sg += 1;
        else if (file === "GameUserSettings.ini") breakdown.display += 1;
        else breakdown.engine += 1;
      }
    }
  }

  return { total, breakdown, pendingKeys: [...pendingKeys].map((key) => key.toLowerCase()) };
}
