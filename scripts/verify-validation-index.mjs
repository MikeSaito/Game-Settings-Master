import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { execSync } from "node:child_process";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const target = path.join(root, "shared/ue-validation-index.json");
const backup = fs.existsSync(target) ? fs.readFileSync(target, "utf8") : null;

try {
  execSync("npm run validation:index", { cwd: root, stdio: "pipe" });
} catch (error) {
  console.error(error.stdout?.toString() ?? error.message);
  process.exit(1);
}

const current = fs.readFileSync(target, "utf8");
if (backup !== null && current !== backup) {
  fs.writeFileSync(target, backup);
  console.error(
    "shared/ue-validation-index.json is out of sync with src-tauri/catalog/ue_reference_index.json.\n" +
      "Run: npm run validation:index\n" +
      "Then commit the updated shared/ue-validation-index.json",
  );
  process.exit(1);
}

console.log("ue-validation-index.json is in sync with ue_reference_index.json.");
