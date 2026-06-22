import fs from "node:fs";
import path from "node:path";

const root = path.resolve("src");
const replacements = [
  [/from (['"])\.\.\/hooks\/useAppSettings\1/g, "from $1@/hooks/app/useAppSettings$1"],
  [/from (['"])\.\.\/hooks\/useAppUpdater\1/g, "from $1@/hooks/app/useAppUpdater$1"],
  [/from (['"])\.\.\/hooks\/useBackgroundSafeEnabled\1/g, "from $1@/hooks/app/useBackgroundSafeEnabled$1"],
  [/from (['"])\.\.\/hooks\/useDebouncedValue\1/g, "from $1@/hooks/app/useDebouncedValue$1"],
  [/from (['"])\.\.\/hooks\/useDebouncedCallback\1/g, "from $1@/hooks/app/useDebouncedCallback$1"],
  [/from (['"])\.\.\/hooks\/useAppWindowFocused\1/g, "from $1@/hooks/app/useAppWindowFocused$1"],
  [/from (['"])\.\.\/hooks\/useGameRunning\1/g, "from $1@/hooks/game/useGameRunning$1"],
  [/from (['"])\.\.\/hooks\/useRunningExeName\1/g, "from $1@/hooks/game/useRunningExeName$1"],
  [/from (['"])\.\.\/hooks\/useAdvancedEditorState\1/g, "from $1@/hooks/editor/useAdvancedEditorState$1"],
  [/from (['"])\.\/hooks\/useAppSettings\1/g, "from $1@/hooks/app/useAppSettings$1"],
  [/from (['"])\.\/hooks\/useBackgroundSafeEnabled\1/g, "from $1@/hooks/app/useBackgroundSafeEnabled$1"],
  [/from (['"])\.\.\/\.\.\/hooks\/useAppSettings\1/g, "from $1@/hooks/app/useAppSettings$1"],
  [/from (['"])\.\.\/\.\.\/hooks\/useBackgroundSafeEnabled\1/g, "from $1@/hooks/app/useBackgroundSafeEnabled$1"],
  [/from (['"])\.\.\/\.\.\/hooks\/useGameRunning\1/g, "from $1@/hooks/game/useGameRunning$1"],
  [/from (['"])\.\.\/\.\.\/hooks\/useAdvancedEditorState\1/g, "from $1@/hooks/editor/useAdvancedEditorState$1"],
  [/from (['"])\.\/components\/UpdateGate\1/g, "from $1@/components/app/UpdateGate$1"],
  [/from (['"])\.\/components\/ErrorBoundary\1/g, "from $1@/components/app/ErrorBoundary$1"],
  [/from (['"])\.\.\/components\/UpdateGate\1/g, "from $1@/components/app/UpdateGate$1"],
  [/from (['"])\.\.\/components\/GameCover\1/g, "from $1@/components/game/GameCover$1"],
  [/from (['"])\.\.\/\.\.\/components\/GameCover\1/g, "from $1@/components/game/GameCover$1"],
  [/from (['"])\.\/useBackgroundSafeEnabled\1/g, "from $1@/hooks/app/useBackgroundSafeEnabled$1"],
  [/from (['"])\.\/useAppSettings\1/g, "from $1@/hooks/app/useAppSettings$1"],
  [/from (['"])\.\/useGameRunning\1/g, "from $1@/hooks/game/useGameRunning$1"],
  [/from (['"])\.\/useRunningExeName\1/g, "from $1@/hooks/game/useRunningExeName$1"],
  [/from (['"])\.\.\/hooks\/useAppUpdater\1/g, "from $1@/hooks/app/useAppUpdater$1"],
  [/from (['"])\.\.\/hooks\/useAppUpdater\1/g, "from $1@/hooks/app/useAppUpdater$1"],
];

function walk(dir, out = []) {
  for (const ent of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, ent.name);
    if (ent.isDirectory()) walk(p, out);
    else if (/\.(ts|tsx)$/.test(ent.name)) out.push(p);
  }
  return out;
}

for (const file of walk(root)) {
  let text = fs.readFileSync(file, "utf8");
  const orig = text;
  for (const [re, to] of replacements) text = text.replace(re, to);
  if (text !== orig) fs.writeFileSync(file, text);
}

console.log("hooks/components import migration done");
