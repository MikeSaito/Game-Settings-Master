import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const source = path.join(root, "src-tauri/catalog/ue_reference_index.json");
const target = path.join(root, "shared/ue-validation-index.json");

const data = JSON.parse(fs.readFileSync(source, "utf8"));
const slim = {};
for (const entry of data.entries ?? []) {
  slim[entry.key.toLowerCase()] = {
    intro: entry.introduced_in ?? null,
    removed: entry.removed_in ?? null,
    ue4: !!entry.ue4,
    ue5: !!entry.ue5,
  };
}
fs.writeFileSync(target, JSON.stringify(slim));
console.log(`Wrote ${Object.keys(slim).length} keys to ${path.relative(root, target)}`);
