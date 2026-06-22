import fs from "node:fs";
import path from "node:path";

const root = path.resolve("src");
const map = {
  "lib/tauriRuntime": "@/lib/api",
  "lib/tauriDialog": "@/lib/api",
  "lib/api": "@/lib/api",
  "lib/types": "@/lib/core",
  "lib/cn": "@/lib/core",
  "lib/errors": "@/lib/core",
  "lib/queryClient": "@/lib/core",
  "lib/appSettings": "@/lib/settings",
  "lib/routes": "@/lib/routing",
  "lib/navigation": "@/lib/routing",
  "lib/legacyGameRouteRedirect": "@/lib/routing",
  "lib/editorPanels": "@/lib/routing",
  "lib/advancedEditorPanels": "@/lib/routing",
  "lib/advancedEditorFilters": "@/lib/editor",
  "lib/engineParams": "@/lib/editor",
  "lib/cvarHumanize": "@/lib/editor",
  "lib/paramValue": "@/lib/editor",
  "lib/paramValueEqual": "@/lib/editor",
  "lib/paramSelectOptions": "@/lib/editor",
  "lib/paramDependencies": "@/lib/editor",
  "lib/sgEngineConflicts": "@/lib/editor",
  "lib/buildCustomChanges": "@/lib/editor",
  "lib/lastPreset": "@/lib/editor",
  "lib/gameEngine": "@/lib/game",
  "lib/gameRunning": "@/lib/game",
  "lib/gameCover": "@/lib/game",
  "lib/prefetchGameWorkspace": "@/lib/game",
  "lib/gpuCompat": "@/lib/gpu",
};

function walk(dir, out = []) {
  for (const ent of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, ent.name);
    if (ent.isDirectory()) walk(p, out);
    else if (/\.(ts|tsx)$/.test(ent.name)) out.push(p);
  }
  return out;
}

for (const file of walk(root)) {
  if (file.includes(`${path.sep}lib${path.sep}`) && !file.includes(`${path.sep}lib${path.sep}api${path.sep}`)) {
    // still migrate imports inside lib subfolders if any old paths remain
  }
  let text = fs.readFileSync(file, "utf8");
  const orig = text;
  for (const [from, to] of Object.entries(map)) {
    const re = new RegExp(`from (['"])(?:\\.\\./)+${from.replace("/", "\\/")}\\1`, "g");
    text = text.replace(re, `from $1${to}$1`);
    const re2 = new RegExp(`from (['"])\\./${from.replace("/", "\\/")}\\1`, "g");
    text = text.replace(re2, `from $1${to}$1`);
  }
  if (text !== orig) fs.writeFileSync(file, text);
}

console.log("import migration done");
