import { readFileSync } from "node:fs";

const packageLock = JSON.parse(readFileSync(new URL("../package-lock.json", import.meta.url), "utf8"));
const cargoLock = readFileSync(new URL("../src-tauri/Cargo.lock", import.meta.url), "utf8");

const npmPackages = packageLock.packages ?? {};
const cargoVersions = new Map();

for (const block of cargoLock.split(/\r?\n\r?\n/)) {
  const name = block.match(/^name = "([^"]+)"$/m)?.[1];
  const version = block.match(/^version = "([^"]+)"$/m)?.[1];
  if (name && version && (name === "tauri" || name.startsWith("tauri-plugin-"))) {
    cargoVersions.set(name, version);
  }
}

const mismatches = [];
for (const [path, metadata] of Object.entries(npmPackages)) {
  const match = path.match(/^node_modules\/@tauri-apps\/(api|plugin-.+)$/);
  if (!match || typeof metadata?.version !== "string") continue;

  const rustName = match[1] === "api" ? "tauri" : `tauri-${match[1]}`;
  const rustVersion = cargoVersions.get(rustName);
  if (!rustVersion) continue;

  const npmMinor = metadata.version.split(".").slice(0, 2).join(".");
  const rustMinor = rustVersion.split(".").slice(0, 2).join(".");
  if (npmMinor !== rustMinor) {
    mismatches.push(`${rustName} (${rustVersion}) != @tauri-apps/${match[1]} (${metadata.version})`);
  }
}

if (mismatches.length > 0) {
  console.error("Tauri JavaScript packages and Rust crates must use the same major/minor version:");
  for (const mismatch of mismatches) console.error(`- ${mismatch}`);
  process.exit(1);
}

console.log("Tauri JavaScript and Rust package versions are aligned.");
