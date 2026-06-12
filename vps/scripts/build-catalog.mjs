import archiver from "archiver";
import crypto from "node:crypto";
import fsSync from "node:fs";
import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const VPS_ROOT = path.join(__dirname, "..");
const PUBLIC = path.join(VPS_ROOT, "public");
const SOURCE = path.join(VPS_ROOT, "source");

const UNITY_PRESET_IDS = [
  { id: "ultra-low", name: "Ultra Low", file: "ultra-low.json" },
  { id: "low", name: "Low", file: "low.json" },
  { id: "medium", name: "Medium", file: "medium.json" },
  { id: "high", name: "High", file: "high.json" },
  { id: "epic", name: "Epic", file: "epic.json" },
  { id: "ultra-high", name: "Ultra High", file: "ultra-high.json" },
];

const FORZA_PRESETS = [
  { id: "potato", name: "Potato", description: "Минимально играбельные настройки — максимум FPS на слабом железе.", profile_folder: "06_Potato" },
  { id: "minimum", name: "Minimum", description: "Производительность для 8 GB VRAM. Без RT, TAA выкл.", profile_folder: "01_Minimum" },
  { id: "low", name: "Low", description: "Производительность для 12 GB VRAM. Сбалансированный FPS.", profile_folder: "02_Low" },
  { id: "medium", name: "Medium", description: "Фотореализм без ray tracing — баланс картинки и FPS.", profile_folder: "03_Medium" },
  { id: "high", name: "High", description: "Высокое качество без RT. DLSS Performance на RTX, FSR на AMD.", profile_folder: "04_High" },
  { id: "ultramax", name: "Ultra Max", description: "RT отражения + RTGI. Нужно 12–16+ GB VRAM. Без DLAA/NVIDIATech.", profile_folder: "05_UltraMax" },
];

const CATALOG_VERSION = "1.5.0";

async function sha256File(filePath) {
  const data = await fs.readFile(filePath);
  return crypto.createHash("sha256").update(data).digest("hex");
}

async function zipDirectory(sourceDir, outZip) {
  await fs.mkdir(path.dirname(outZip), { recursive: true });
  await new Promise((resolve, reject) => {
    const output = fsSync.createWriteStream(outZip);
    const archive = archiver("zip", { zlib: { level: 9 } });
    output.on("close", resolve);
    archive.on("error", reject);
    archive.pipe(output);
    archive.directory(sourceDir, false);
    archive.finalize();
  });
}

async function buildPack({ packId, stagingFn, manifest }) {
  const packDir = path.join(PUBLIC, "packs", packId);
  const staging = path.join(packDir, "_staging");
  await fs.rm(packDir, { recursive: true, force: true });
  await fs.mkdir(staging, { recursive: true });
  try {
    await stagingFn(staging);

    const zipPath = path.join(packDir, "pack.zip");
    await zipDirectory(staging, zipPath);
    const sha256 = await sha256File(zipPath);
    manifest.bundle = { file: "pack.zip", sha256 };
    manifest.revision = CATALOG_VERSION;
    manifest.updated_at = new Date().toISOString();
    await fs.writeFile(
      path.join(packDir, "manifest.json"),
      JSON.stringify(manifest, null, 2),
      "utf8",
    );
    return { packId, manifest_url: `packs/${packId}/manifest.json`, sha256 };
  } finally {
    await fs.rm(staging, { recursive: true, force: true });
  }
}

async function buildForzaPack() {
  return buildPack({
    packId: "forza-fh6",
    stagingFn: async (staging) => {
      await fs.cp(path.join(SOURCE, "forza-fh6", "presets"), path.join(staging, "presets"), { recursive: true });
      await fs.copyFile(path.join(SOURCE, "forza-fh6", "parameter-catalog.json"), path.join(staging, "parameter-catalog.json"));
      await fs.copyFile(path.join(SOURCE, "forza-fh6", "policy.json"), path.join(staging, "policy.json"));
    },
    manifest: {
      schema_version: 1,
      pack_id: "forza-fh6",
      title: "Forza Horizon 6 — авторские пресеты",
      match: { steam_app_ids: ["2483190"], game_ids: ["steam-2483190"], engine_families: ["forza"] },
      apply: {
        kind: "forza",
        presets_root: "presets",
        user_config_patch: "Preset.xml",
        media_dir: "media",
        parameter_catalog: "parameter-catalog.json",
      },
      presets: FORZA_PRESETS,
    },
  });
}

async function buildSubnauticaOverlayPack() {
  return buildPack({
    packId: "subnautica2-overlay",
    stagingFn: async (staging) => {
      await fs.copyFile(path.join(SOURCE, "subnautica2-overlay", "overlay.json"), path.join(staging, "overlay.json"));
    },
    manifest: {
      schema_version: 1,
      pack_id: "subnautica2-overlay",
      title: "Subnautica 2 — UE overlay",
      match: { steam_app_ids: ["1962700"], game_ids: ["steam-1962700"], engine_families: ["ue5"], overlay_ids: ["subnautica2"] },
      apply: { kind: "ue_overlay", overlay_id: "subnautica2", overlay_file: "overlay.json" },
    },
  });
}

async function buildUnityTiersPack() {
  return buildPack({
    packId: "unity-tiers",
    stagingFn: async (staging) => {
      await fs.cp(path.join(SOURCE, "unity-tiers", "presets"), path.join(staging, "presets"), { recursive: true });
    },
    manifest: {
      schema_version: 1,
      pack_id: "unity-tiers",
      title: "Unity tier presets",
      match: { engine_families: ["unity"] },
      apply: { kind: "unity", presets_root: "presets" },
      presets: UNITY_PRESET_IDS.map((p) => ({ id: p.id, name: p.name, description: "", definition_file: p.file })),
    },
  });
}

const SN2_RESHADE_PRESETS = [
  {
    id: "sn2-underwater-clarity",
    name: "Underwater Clarity",
    description: "Чёткость под водой для Subnautica 2.",
    ini_file: "sn2-underwater-clarity.ini",
  },
];

async function buildSubnautica2ReShadePack() {
  return buildPack({
    packId: "subnautica2-reshade",
    stagingFn: async (staging) => {
      await fs.cp(
        path.join(SOURCE, "subnautica2-reshade", "presets"),
        path.join(staging, "presets"),
        { recursive: true },
      );
      await fs.copyFile(
        path.join(SOURCE, "subnautica2-reshade", "presets", "sn2-underwater-clarity.ini"),
        path.join(staging, "manifest-presets", "sn2-underwater-clarity.ini"),
      ).catch(() => {});
    },
    manifest: {
      schema_version: 1,
      pack_id: "subnautica2-reshade",
      title: "Subnautica 2 — ReShade presets",
      match: {
        steam_app_ids: ["1962700"],
        game_ids: ["steam-1962700"],
        engine_families: ["ue5"],
      },
      apply: { kind: "reshade_ini", presets_root: "presets" },
      presets: SN2_RESHADE_PRESETS,
    },
  });
}

async function buildUeCatalogPack() {
  return buildPack({
    packId: "ue-catalog",
    stagingFn: async (staging) => {
      await fs.cp(path.join(SOURCE, "ue-catalog"), path.join(staging, "catalog"), { recursive: true });
    },
    manifest: {
      schema_version: 1,
      pack_id: "ue-catalog",
      title: "UE parameter catalog",
      match: { engine_families: ["ue4", "ue5"] },
      apply: { kind: "catalog", catalog_root: "catalog" },
    },
  });
}

async function main() {
  await fs.mkdir(PUBLIC, { recursive: true });
  const packs = [];
  packs.push(await buildForzaPack());
  packs.push(await buildSubnauticaOverlayPack());
  packs.push(await buildSubnautica2ReShadePack());
  packs.push(await buildUnityTiersPack());
  packs.push(await buildUeCatalogPack());

  const catalog = {
    schema_version: 1,
    catalog_id: "gsm-author-presets",
    version: CATALOG_VERSION,
    updated_at: new Date().toISOString(),
    base_url: ".",
    packs: packs.map((p) => ({ id: p.packId, manifest_url: p.manifest_url })),
  };

  await fs.writeFile(path.join(PUBLIC, "catalog.json"), JSON.stringify(catalog, null, 2), "utf8");
  console.log("Catalog built:", path.join(PUBLIC, "catalog.json"));
  console.log("Packs:", packs.map((p) => p.packId).join(", "));
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
