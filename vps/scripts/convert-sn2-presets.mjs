import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REF = process.env.SN2_REF_PRESETS
  ?? "C:\\Users\\Mike\\Desktop\\Новая папка (2)\\Mods SB2\\GraphicsPresets\\presets";
const OUT = path.join(__dirname, "..", "source", "subnautica2-tiers", "presets");

const TIERS = [
  { id: "ultra-low", name: "Ultra Low", folder: "potato", description: "Минимум нагрузки — potato tier для SN2." },
  { id: "low", name: "Low", folder: "low", description: "Low: DLSS 60%, Lumen off, лёгкие тени." },
  { id: "medium", name: "Medium", folder: "medium", description: "Medium: частичный Lumen, DLSS 75%." },
  { id: "high", name: "High", folder: "high", description: "High: Lumen on, DLSS 90%, FG optional." },
  { id: "epic", name: "Epic", folder: "ultramax", description: "Epic: максимум меню без ultramax engine push." },
  { id: "ultra-high", name: "Ultra High", folder: "ultramax", description: "Ultra Max: полный ultramax профиль." },
];

function parseIni(text) {
  const sections = {};
  let section = null;
  for (const raw of text.split(/\r?\n/)) {
    const line = raw.trim();
    if (!line || line.startsWith(";") || line.startsWith("#")) continue;
    const sec = line.match(/^\[(.+)\]$/);
    if (sec) {
      section = sec[1];
      sections[section] = sections[section] ?? {};
      continue;
    }
    const eq = line.indexOf("=");
    if (eq === -1 || !section) continue;
    const key = line.slice(0, eq).trim();
    let val = line.slice(eq + 1).trim();
    val = val.replace(/^1920$/, "{{width}}").replace(/^1080$/, "{{height}}");
    sections[section][key] = val;
  }
  return sections;
}

function readIni(file) {
  if (!fs.existsSync(file)) return null;
  return parseIni(fs.readFileSync(file, "utf8"));
}

function epicOverrides(gus, engine) {
  const sg = gus["ScalabilityGroups"];
  if (sg) {
    for (const k of Object.keys(sg)) {
      if (k.startsWith("sg.") && k !== "sg.ResolutionQuality") sg[k] = "4";
    }
    sg["sg.ResolutionQuality"] = "100";
  }
  const local = gus["/Script/Subnautica2.SN2SettingsLocal"];
  if (local) {
    local["UpscalingFrameGeneration"] = "0";
    local["bUseVSync"] = "False";
  }
  if (engine?.["SystemSettings"]) {
    engine["SystemSettings"]["sg.DefaultScalabilityLevel"] = "4";
    engine["SystemSettings"]["r.ViewDistanceScale"] = "1.75";
    engine["SystemSettings"]["r.Shadow.MaxResolution"] = "2048";
  }
}

for (const tier of TIERS) {
  const dir = path.join(REF, tier.folder);
  const gusIni = readIni(path.join(dir, "GameUserSettings.ini"));
  const engIni = readIni(path.join(dir, "Engine.ini"));
  if (!gusIni) {
    console.error(`Missing GUS for ${tier.id} in ${dir}`);
    process.exit(1);
  }

  const gus = structuredClone(gusIni);
  const engine = engIni ? structuredClone(engIni) : null;

  if (tier.id === "epic") {
    epicOverrides(gus, engine);
  }

  const files = { "GameUserSettings.ini": gus };
  if (engine) {
    files["Engine.ini"] = engine;
  }

  const preset = {
    id: tier.id,
    name: tier.name,
    description: tier.description,
    files,
  };

  fs.mkdirSync(OUT, { recursive: true });
  fs.writeFileSync(path.join(OUT, `${tier.id}.json`), JSON.stringify(preset, null, 2), "utf8");
  console.log(`Wrote ${tier.id}.json`);
}
