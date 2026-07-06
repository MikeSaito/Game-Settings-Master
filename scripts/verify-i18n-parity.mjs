import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const localesDir = path.join(root, "src/i18n/locales");

function collectKeys(value, prefix = "") {
  const keys = new Set();
  if (value == null || typeof value !== "object" || Array.isArray(value)) {
    return keys;
  }
  for (const [key, nested] of Object.entries(value)) {
    const full = prefix ? `${prefix}.${key}` : key;
    keys.add(full);
    for (const child of collectKeys(nested, full)) {
      keys.add(child);
    }
  }
  return keys;
}

function logicalKeys(keys) {
  return new Set(
    [...keys].map((key) => key.replace(/_(one|other|few|many)$/, "")),
  );
}

function loadNamespace(lang, file) {
  const filePath = path.join(localesDir, lang, file);
  return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

const namespaces = fs
  .readdirSync(path.join(localesDir, "en"))
  .filter((name) => name.endsWith(".json"));

let failed = false;

for (const ns of namespaces) {
  const enKeys = logicalKeys(collectKeys(loadNamespace("en", ns)));
  const ruKeys = logicalKeys(collectKeys(loadNamespace("ru", ns)));
  const missingInRu = [...enKeys].filter((key) => !ruKeys.has(key)).sort();
  const missingInEn = [...ruKeys].filter((key) => !enKeys.has(key)).sort();

  if (missingInRu.length > 0 || missingInEn.length > 0) {
    failed = true;
    console.error(`i18n parity failed for ${ns}:`);
    if (missingInRu.length > 0) {
      console.error(`  missing in ru: ${missingInRu.join(", ")}`);
    }
    if (missingInEn.length > 0) {
      console.error(`  missing in en: ${missingInEn.join(", ")}`);
    }
  }
}

if (failed) {
  process.exit(1);
}

console.log(`i18n parity OK (${namespaces.length} namespaces, en / ru)`);
