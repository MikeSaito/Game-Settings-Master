import Ajv from "ajv";
import addFormats from "ajv-formats";
import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const VPS_ROOT = path.join(__dirname, "..");
const PUBLIC = path.join(VPS_ROOT, "public");
const SCHEMA_PATH = path.join(VPS_ROOT, "schema", "preset-catalog.v1.schema.json");

const ajv = new Ajv({ allErrors: true, strict: false });
addFormats(ajv);

const schema = JSON.parse(await fs.readFile(SCHEMA_PATH, "utf8"));
delete schema.$schema;

const validateCatalog = ajv.compile(schema);
const validateManifest = ajv.compile({
  ...schema.$defs.pack_manifest,
  $defs: schema.$defs,
});

async function readJson(filePath) {
  return JSON.parse(await fs.readFile(filePath, "utf8"));
}

function reportErrors(label, validate, data) {
  if (validate(data)) return true;
  console.error(`\n${label}:`);
  for (const err of validate.errors ?? []) {
    console.error(`  - ${err.instancePath || "/"} ${err.message}`);
  }
  return false;
}

async function main() {
  const catalogPath = path.join(PUBLIC, "catalog.json");
  const catalog = await readJson(catalogPath);
  let ok = reportErrors("catalog.json", validateCatalog, catalog);

  for (const pack of catalog.packs ?? []) {
    const manifestPath = path.join(PUBLIC, pack.manifest_url.replace(/^\//, ""));
    const manifest = await readJson(manifestPath);
    ok = reportErrors(pack.id, validateManifest, manifest) && ok;

    if (manifest.bundle?.file) {
      const zipPath = path.join(path.dirname(manifestPath), manifest.bundle.file);
      try {
        await fs.access(zipPath);
      } catch {
        console.error(`\n${pack.id}: missing bundle ${manifest.bundle.file}`);
        ok = false;
      }
    }
  }

  if (!ok) {
    process.exit(1);
  }
  console.log("Schema validation OK:", catalog.packs?.length ?? 0, "packs");
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
